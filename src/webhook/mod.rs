//! Outbound webhook alerts for hits.
//!
//! HoneyPrompt POSTs a JSON body to an operator-configured URL. Authenticity is
//! proven with an HMAC-SHA256 signature (RustCrypto `hmac` + `sha2`) over
//! `"{timestamp}.{raw body}"`, sent as `X-HoneyPrompt-Signature: t=..,v1=..`.
//! Slack and Discord formats exist because those services reject generic JSON;
//! agent-supplied text is escaped so it cannot inject links or mentions.
//!
//! Secrets (the secret, the auth header and the URL's path/query) are never
//! printed or logged; only the host is.

use std::sync::Arc;
use std::time::{Duration, Instant};

use hmac::{Hmac, Mac};
use reqwest::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::store::HitTiming;

type HmacSha256 = Hmac<Sha256>;

pub const ENV_URL: &str = "HONEYPROMPT_WEBHOOK_URL";
pub const ENV_SECRET: &str = "HONEYPROMPT_WEBHOOK_SECRET";
pub const ENV_AUTH_HEADER: &str = "HONEYPROMPT_WEBHOOK_AUTH_HEADER";

/// Shortest accepted signing secret.
pub const MIN_SECRET_CHARS: usize = 32;
/// Longest user-agent included in an alert.
pub const MAX_ALERT_UA_CHARS: usize = 256;
/// Per-attempt timeout.
pub const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(5);
/// Delays before the 2nd and 3rd attempts.
pub const RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(1), Duration::from_secs(5)];
/// At most this many alerts are sent per `RATE_WINDOW`; the rest are counted
/// and reported in the next alert's `suppressed` field.
pub const RATE_LIMIT: u32 = 30;
pub const RATE_WINDOW: Duration = Duration::from_secs(60);
/// Alerts waiting to be sent; when full, new alerts are dropped (and counted).
pub const QUEUE_CAPACITY: usize = 64;

/// Body format.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookFormat {
    #[default]
    Json,
    Slack,
    Discord,
}

/// Which hits trigger an alert.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookSend {
    /// Delayed and out-of-band hits only.
    #[default]
    Notable,
    /// Every hit.
    All,
}

/// `[webhook]` section of honeyprompt.toml. Every field can be overridden by an
/// environment variable (`HONEYPROMPT_WEBHOOK_*`), which keeps secrets out of files.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebhookConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// A full header line, e.g. `Authorization: Bearer abc`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    pub format: WebhookFormat,
    pub send: WebhookSend,
}

impl WebhookConfig {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

/// A validated, ready-to-use webhook destination.
#[derive(Clone)]
pub struct Webhook {
    url: Url,
    secret: Option<String>,
    auth_header: Option<(HeaderName, HeaderValue)>,
    pub format: WebhookFormat,
    pub send: WebhookSend,
    /// Host of `callback_base_url`, so one receiver can tell honeypots apart.
    deployment: String,
    retry_delays: Vec<Duration>,
    rate_limit: u32,
    rate_window: Duration,
}

impl std::fmt::Debug for Webhook {
    // Never print the URL path, secret or auth header.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.describe())
    }
}

/// One hit, as sent in an alert.
#[derive(Debug, Clone, PartialEq)]
pub struct HitAlert {
    pub received_at: i64,
    pub tier: u8,
    pub nonce: String,
    pub payload_id: String,
    pub timing: HitTiming,
    pub since_page_fetch_secs: Option<i64>,
    pub since_seeding_secs: Option<i64>,
    pub source_ip: String,
    pub user_agent: String,
    pub classification: String,
    pub t5_proof_valid: Option<bool>,
}

fn non_empty(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

impl Webhook {
    /// Build from config plus environment overrides. `Ok(None)` when no URL is set.
    pub fn from_config(
        config: &Config,
        env: impl Fn(&str) -> Option<String>,
    ) -> anyhow::Result<Option<Self>> {
        let wc = &config.webhook;
        let url = match non_empty(env(ENV_URL)).or_else(|| non_empty(wc.url.clone())) {
            Some(u) => u,
            None => return Ok(None),
        };
        let secret = non_empty(env(ENV_SECRET)).or_else(|| non_empty(wc.secret.clone()));
        let auth_header =
            non_empty(env(ENV_AUTH_HEADER)).or_else(|| non_empty(wc.auth_header.clone()));

        // Errors must not echo the URL: it may itself be the credential.
        let url =
            Url::parse(&url).map_err(|_| anyhow::anyhow!("webhook url is not a valid URL"))?;
        let host = url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("webhook url has no host"))?;
        let is_local = matches!(host, "localhost" | "127.0.0.1" | "[::1]");
        match url.scheme() {
            "https" => {}
            "http" if is_local => {}
            _ => anyhow::bail!(
                "webhook url must use https:// (http:// is only allowed for localhost)"
            ),
        }
        if let Some(s) = &secret {
            if s.chars().count() < MIN_SECRET_CHARS {
                anyhow::bail!(
                    "webhook secret must be at least {MIN_SECRET_CHARS} characters (try `openssl rand -hex 32`)"
                );
            }
        }
        let auth_header = match auth_header {
            None => None,
            Some(line) => {
                let (name, value) = line.split_once(':').ok_or_else(|| {
                    anyhow::anyhow!("webhook auth_header must look like `Name: value`")
                })?;
                let name = HeaderName::from_bytes(name.trim().as_bytes()).map_err(|_| {
                    anyhow::anyhow!("webhook auth_header has an invalid header name")
                })?;
                let mut value = HeaderValue::from_str(value.trim())
                    .map_err(|_| anyhow::anyhow!("webhook auth_header has an invalid value"))?;
                value.set_sensitive(true);
                Some((name, value))
            }
        };
        let deployment = Url::parse(&config.callback_base_url)
            .ok()
            .and_then(|u| u.host_str().map(str::to_string))
            .unwrap_or_default();

        Ok(Some(Self {
            url,
            secret,
            auth_header,
            format: wc.format,
            send: wc.send,
            deployment,
            retry_delays: RETRY_DELAYS.to_vec(),
            rate_limit: RATE_LIMIT,
            rate_window: RATE_WINDOW,
        }))
    }

    /// Like `from_config`, reading the process environment.
    pub fn from_config_and_env(config: &Config) -> anyhow::Result<Option<Self>> {
        Self::from_config(config, |k| std::env::var(k).ok())
    }

    /// The destination host only.
    pub fn host(&self) -> &str {
        self.url.host_str().unwrap_or("?")
    }

    /// Safe one-line summary for startup output.
    pub fn describe(&self) -> String {
        format!(
            "{} (signed: {}, format {}, send {})",
            self.host(),
            if self.secret.is_some() { "yes" } else { "no" },
            match self.format {
                WebhookFormat::Json => "json",
                WebhookFormat::Slack => "slack",
                WebhookFormat::Discord => "discord",
            },
            match self.send {
                WebhookSend::Notable => "notable",
                WebhookSend::All => "all",
            },
        )
    }

    /// Whether a hit with this timing should be sent.
    pub fn wants(&self, timing: HitTiming) -> bool {
        wanted(self.send, timing)
    }

    #[cfg(test)]
    fn for_tests(mut self, retry_delays: Vec<Duration>, rate_limit: u32) -> Self {
        self.retry_delays = retry_delays;
        self.rate_limit = rate_limit;
        self
    }

    /// Serialize the request body for a tier hit in the configured format.
    pub fn build_body(
        &self,
        alert: &HitAlert,
        delivery_id: &str,
        suppressed: u64,
        test: bool,
    ) -> Vec<u8> {
        self.build_alert_body(&Alert::Hit(alert.clone()), delivery_id, suppressed, test)
    }

    /// Serialize the request body for any alert in the configured format.
    pub fn build_alert_body(
        &self,
        alert: &Alert,
        delivery_id: &str,
        suppressed: u64,
        test: bool,
    ) -> Vec<u8> {
        let value = match (self.format, alert) {
            (WebhookFormat::Json, Alert::Hit(hit)) => serde_json::to_value(JsonBody {
                version: 1,
                event: "hit",
                test,
                delivery_id,
                deployment: &self.deployment,
                hit: JsonHit::from(hit),
                suppressed,
            })
            .expect("alert body serializes"),
            (WebhookFormat::Json, Alert::Seed(s)) => serde_json::json!({
                "version": 1,
                "event": "seed_hit",
                "test": test,
                "delivery_id": delivery_id,
                "deployment": &self.deployment,
                "seed": {
                    "label": &s.label,
                    "note": &s.note,
                    "nonce": &s.nonce,
                    "created_at": s.created_at,
                    "since_seeding_secs": s.received_at - s.created_at,
                },
                "hit": {
                    "received_at": s.received_at,
                    "source_ip": &s.source_ip,
                    "user_agent": truncate(&s.user_agent),
                    "classification": &s.classification,
                },
                "suppressed": suppressed,
            }),
            (WebhookFormat::Slack, a) => serde_json::json!({
                "text": slack_text(&self.deployment, a, suppressed, test),
            }),
            (WebhookFormat::Discord, a) => serde_json::json!({
                "content": discord_text(&self.deployment, a, suppressed, test),
                "allowed_mentions": { "parse": [] },
            }),
        };
        serde_json::to_vec(&value).expect("alert body serializes")
    }
}

/// A fetch of a seeded canary URL (`/r/{nonce}`).
#[derive(Debug, Clone, PartialEq)]
pub struct SeedAlert {
    pub received_at: i64,
    pub label: String,
    pub note: Option<String>,
    pub nonce: String,
    pub created_at: i64,
    pub source_ip: String,
    pub user_agent: String,
    pub classification: String,
}

/// Anything the webhook can report.
#[derive(Debug, Clone, PartialEq)]
pub enum Alert {
    Hit(HitAlert),
    Seed(SeedAlert),
}

#[derive(Serialize)]
struct JsonBody<'a> {
    version: u32,
    event: &'static str,
    test: bool,
    delivery_id: &'a str,
    deployment: &'a str,
    hit: JsonHit<'a>,
    suppressed: u64,
}

#[derive(Serialize)]
struct JsonHit<'a> {
    received_at: i64,
    tier: u8,
    tier_name: &'static str,
    nonce: &'a str,
    payload_id: &'a str,
    timing: &'static str,
    since_page_fetch_secs: Option<i64>,
    since_seeding_secs: Option<i64>,
    source_ip: &'a str,
    user_agent: String,
    classification: &'a str,
    t5_proof_valid: Option<bool>,
}

impl<'a> From<&'a HitAlert> for JsonHit<'a> {
    fn from(a: &'a HitAlert) -> Self {
        JsonHit {
            received_at: a.received_at,
            tier: a.tier,
            tier_name: crate::report::proof_level(a.tier),
            nonce: &a.nonce,
            payload_id: &a.payload_id,
            timing: a.timing.as_str(),
            since_page_fetch_secs: a.since_page_fetch_secs,
            since_seeding_secs: a.since_seeding_secs,
            source_ip: &a.source_ip,
            user_agent: truncate(&a.user_agent),
            classification: &a.classification,
            t5_proof_valid: a.t5_proof_valid,
        }
    }
}

fn wanted(send: WebhookSend, timing: HitTiming) -> bool {
    match send {
        WebhookSend::All => true,
        WebhookSend::Notable => matches!(timing, HitTiming::Delayed | HitTiming::OutOfBand),
    }
}

fn truncate(s: &str) -> String {
    s.chars().take(MAX_ALERT_UA_CHARS).collect()
}

/// Text for an inline code span: backticks would close it, newlines break it.
fn code_safe(s: &str) -> String {
    truncate(s)
        .chars()
        .map(|c| match c {
            '`' => '\'',
            '\n' | '\r' => ' ',
            c => c,
        })
        .collect()
}

/// Slack treats `&`, `<` and `>` as control characters (links, `<!channel>`).
fn slack_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Backslash-escape Discord markdown; mentions are disabled via `allowed_mentions`.
fn discord_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if "\\*_~`|>[]()#-:@<".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn summary_line(deployment: &str, a: &Alert, suppressed: u64, test: bool) -> String {
    let mut s = String::from(if test { "[TEST] " } else { "" });
    match a {
        Alert::Hit(h) => s.push_str(&format!(
            "HoneyPrompt: tier {} ({}) {} hit on {} from {}",
            h.tier,
            crate::report::proof_level(h.tier),
            h.timing.as_str(),
            deployment,
            h.source_ip,
        )),
        Alert::Seed(sd) => s.push_str(&format!(
            "HoneyPrompt: seeded canary fetched on {} from {}, {} after seeding",
            deployment,
            sd.source_ip,
            crate::report::format_duration(sd.received_at - sd.created_at),
        )),
    }
    if suppressed > 0 {
        s.push_str(&format!(" ({suppressed} earlier alerts suppressed)"));
    }
    s
}

/// (user agent, classification, seed label, seed note) for the chat formats.
fn chat_fields(a: &Alert) -> (&str, &str, Option<&str>, Option<&str>) {
    match a {
        Alert::Hit(h) => (&h.user_agent, &h.classification, None, None),
        Alert::Seed(s) => (
            &s.user_agent,
            &s.classification,
            Some(s.label.as_str()),
            s.note.as_deref(),
        ),
    }
}

fn slack_text(deployment: &str, a: &Alert, suppressed: u64, test: bool) -> String {
    let (ua, class, label, note) = chat_fields(a);
    let mut out = slack_escape(&summary_line(deployment, a, suppressed, test));
    if let Some(l) = label {
        out.push_str(&format!(" | seed `{}`", slack_escape(&code_safe(l))));
    }
    if let Some(n) = note {
        out.push_str(&format!(" | note: {}", slack_escape(&truncate(n))));
    }
    out.push_str(&format!(
        " | UA `{}` | {}",
        slack_escape(&code_safe(ua)),
        slack_escape(class)
    ));
    out
}

fn discord_text(deployment: &str, a: &Alert, suppressed: u64, test: bool) -> String {
    let (ua, class, label, note) = chat_fields(a);
    let mut out = discord_escape(&summary_line(deployment, a, suppressed, test));
    if let Some(l) = label {
        out.push_str(&format!(" | seed `{}`", code_safe(l)));
    }
    if let Some(n) = note {
        out.push_str(&format!(" | note: {}", discord_escape(&truncate(n))));
    }
    out.push_str(&format!(
        " | UA `{}` | {}",
        code_safe(ua),
        discord_escape(class)
    ));
    out
}

/// Hex HMAC-SHA256 over `"{timestamp}.{body}"`.
pub fn sign(secret: &str, timestamp: i64, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(body);
    hex::encode(mac.finalize().into_bytes())
}

/// Value of the `X-HoneyPrompt-Signature` header.
pub fn signature_header(secret: &str, timestamp: i64, body: &[u8]) -> String {
    format!("t={timestamp},v1={}", sign(secret, timestamp, body))
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn delivery_id() -> String {
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).expect("OS CSPRNG unavailable");
    hex::encode(buf)
}

/// HTTPS client: rustls with the `ring` provider and bundled Mozilla roots
/// (webpki-roots), so it works the same in a distroless container. Redirects are
/// not followed, so the auth header can never be sent to another host.
pub fn build_client() -> anyhow::Result<reqwest::Client> {
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_root_certificates(roots)
    .with_no_client_auth();
    Ok(reqwest::Client::builder()
        .use_preconfigured_tls(tls)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(ATTEMPT_TIMEOUT)
        .user_agent(concat!("honeyprompt/", env!("CARGO_PKG_VERSION")))
        .build()?)
}

/// Why a delivery ultimately failed. Never contains the URL.
#[derive(Debug, PartialEq, Eq)]
pub enum DeliveryError {
    /// Receiver answered with a non-success status (not retried, or retries exhausted).
    Status(StatusCode),
    /// Connection / TLS / timeout error after all attempts.
    Transport(String),
}

impl std::fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryError::Status(s) if s.is_redirection() => {
                write!(f, "receiver answered {s} (redirects are not followed)")
            }
            DeliveryError::Status(s) => write!(f, "receiver answered {s}"),
            DeliveryError::Transport(e) => write!(f, "{e}"),
        }
    }
}

/// POST `body`, retrying network errors, 5xx and 429. Each attempt is signed
/// with a fresh timestamp; the body (and its delivery id) stays the same.
pub async fn deliver(
    client: &reqwest::Client,
    webhook: &Webhook,
    delivery_id: &str,
    body: &[u8],
    retry: bool,
) -> Result<StatusCode, DeliveryError> {
    let delays: &[Duration] = if retry { &webhook.retry_delays } else { &[] };
    let mut last = DeliveryError::Transport("not attempted".into());
    for attempt in 0..=delays.len() {
        if attempt > 0 {
            tokio::time::sleep(delays[attempt - 1]).await;
        }
        let mut req = client
            .post(webhook.url.clone())
            .header(CONTENT_TYPE, "application/json")
            .header("x-honeyprompt-delivery", delivery_id)
            .body(body.to_vec());
        if let Some(secret) = &webhook.secret {
            req = req.header(
                "x-honeyprompt-signature",
                signature_header(secret, now_secs(), body),
            );
        }
        if let Some((name, value)) = &webhook.auth_header {
            req = req.header(name.clone(), value.clone());
        }
        match req.send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return Ok(status);
                }
                last = DeliveryError::Status(status);
                if !(status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS) {
                    return Err(last);
                }
            }
            // `without_url` keeps the URL (possibly the credential) out of logs.
            Err(e) => last = DeliveryError::Transport(e.without_url().to_string()),
        }
    }
    Err(last)
}

/// Send one synthetic alert (`"test": true`) without retries; for `webhook-test`.
pub async fn send_test(webhook: &Webhook) -> Result<StatusCode, DeliveryError> {
    let client = build_client().map_err(|e| DeliveryError::Transport(e.to_string()))?;
    let alert = HitAlert {
        received_at: now_secs(),
        tier: 1,
        nonce: "0000000000000000".into(),
        payload_id: "webhook-test".into(),
        timing: HitTiming::OutOfBand,
        since_page_fetch_secs: None,
        since_seeding_secs: None,
        source_ip: "192.0.2.1".into(),
        user_agent: "honeyprompt-webhook-test".into(),
        classification: "Unknown".into(),
        t5_proof_valid: None,
    };
    let id = delivery_id();
    let body = webhook.build_body(&alert, &id, 0, true);
    deliver(&client, webhook, &id, &body, false).await
}

/// Queue handle used by the DB writer. Alerts the webhook doesn't want are
/// filtered here; a full queue drops the alert and counts it as suppressed.
#[derive(Clone)]
pub struct AlertSender {
    tx: mpsc::Sender<Alert>,
    send: WebhookSend,
    dropped: Arc<std::sync::atomic::AtomicU64>,
}

impl AlertSender {
    pub fn offer(&self, alert: HitAlert) {
        if wanted(self.send, alert.timing) {
            self.push(Alert::Hit(alert));
        }
    }

    /// Seed hits are always sent, whatever `send` is.
    pub fn offer_seed(&self, alert: SeedAlert) {
        self.push(Alert::Seed(alert));
    }

    fn push(&self, alert: Alert) {
        if self.tx.try_send(alert).is_err() {
            self.dropped
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Start the delivery task and return the queue handle.
pub fn spawn(webhook: Webhook) -> anyhow::Result<AlertSender> {
    let client = build_client()?;
    let (tx, rx) = mpsc::channel(QUEUE_CAPACITY);
    let dropped = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let sender = AlertSender {
        tx,
        send: webhook.send,
        dropped: dropped.clone(),
    };
    tokio::spawn(delivery_task(rx, client, webhook, dropped));
    Ok(sender)
}

async fn delivery_task(
    mut rx: mpsc::Receiver<Alert>,
    client: reqwest::Client,
    webhook: Webhook,
    dropped: Arc<std::sync::atomic::AtomicU64>,
) {
    let mut window_start = Instant::now();
    let mut sent_in_window = 0u32;
    let mut suppressed = 0u64;
    while let Some(alert) = rx.recv().await {
        if window_start.elapsed() >= webhook.rate_window {
            window_start = Instant::now();
            sent_in_window = 0;
        }
        if sent_in_window >= webhook.rate_limit {
            suppressed += 1;
            continue;
        }
        sent_in_window += 1;
        let total_suppressed = suppressed + dropped.swap(0, std::sync::atomic::Ordering::Relaxed);
        let id = delivery_id();
        let body = webhook.build_alert_body(&alert, &id, total_suppressed, false);
        match deliver(&client, &webhook, &id, &body, true).await {
            Ok(_) => suppressed = 0,
            Err(e) => {
                // Keep the count so the next successful alert reports it.
                suppressed = total_suppressed;
                eprintln!("warning: webhook to {} failed: {}", webhook.host(), e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use std::sync::Mutex;

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    fn config_with(webhook: WebhookConfig) -> Config {
        Config {
            callback_base_url: "https://honeyprompt.sh".into(),
            webhook,
            ..Config::default()
        }
    }

    fn no_env(_: &str) -> Option<String> {
        None
    }

    fn webhook(
        url: &str,
        secret: Option<&str>,
        format: WebhookFormat,
        send: WebhookSend,
    ) -> Webhook {
        Webhook::from_config(
            &config_with(WebhookConfig {
                url: Some(url.into()),
                secret: secret.map(str::to_string),
                format,
                send,
                ..Default::default()
            }),
            no_env,
        )
        .unwrap()
        .unwrap()
    }

    fn alert(timing: HitTiming, ua: &str) -> HitAlert {
        HitAlert {
            received_at: 1795187274,
            tier: 3,
            nonce: "3a7dd59e33438bc8".into(),
            payload_id: "t3-test".into(),
            timing,
            since_page_fetch_secs: None,
            since_seeding_secs: Some(259200),
            source_ip: "198.51.100.9".into(),
            user_agent: ua.into(),
            classification: "Unknown".into(),
            t5_proof_valid: None,
        }
    }

    // --- signing ---

    #[test]
    fn signature_matches_independent_python_hmac() {
        // python3: hmac.new(SECRET, b'1795187274.{"hello":"world"}', sha256).hexdigest()
        assert_eq!(
            sign(SECRET, 1795187274, br#"{"hello":"world"}"#),
            "9c7d5f867eef15ec79bc7d16cd7f0f9b3500ae5e0e9f6cd289ac205f18bf4691"
        );
    }

    #[test]
    fn signature_verifies_with_crate_constant_time_check() {
        let body = br#"{"a":1}"#;
        let sig = hex::decode(sign(SECRET, 42, body)).unwrap();
        let mut mac = HmacSha256::new_from_slice(SECRET.as_bytes()).unwrap();
        mac.update(b"42.");
        mac.update(body);
        assert!(mac.verify_slice(&sig).is_ok());
        let mut wrong = HmacSha256::new_from_slice(b"another-secret-another-secret-xx").unwrap();
        wrong.update(b"42.");
        wrong.update(body);
        assert!(wrong.verify_slice(&sig).is_err());
    }

    // --- config validation ---

    #[test]
    fn no_url_means_no_webhook() {
        assert!(Webhook::from_config(&Config::default(), no_env)
            .unwrap()
            .is_none());
    }

    #[test]
    fn env_overrides_config() {
        let cfg = config_with(WebhookConfig {
            url: Some("https://file.example.com/x".into()),
            ..Default::default()
        });
        let w = Webhook::from_config(&cfg, |k| match k {
            ENV_URL => Some("https://env.example.com/secret-path".into()),
            ENV_SECRET => Some(SECRET.into()),
            _ => None,
        })
        .unwrap()
        .unwrap();
        assert_eq!(w.host(), "env.example.com");
        assert!(w.secret.is_some());
    }

    #[test]
    fn rejects_plain_http_except_localhost() {
        for (url, ok) in [
            ("http://alerts.example.com/x", false),
            ("ftp://alerts.example.com/x", false),
            ("http://localhost:9000/x", true),
            ("http://127.0.0.1:9000/x", true),
            ("http://[::1]:9000/x", true),
            ("https://alerts.example.com/x", true),
        ] {
            let cfg = config_with(WebhookConfig {
                url: Some(url.into()),
                ..Default::default()
            });
            assert_eq!(Webhook::from_config(&cfg, no_env).is_ok(), ok, "{url}");
        }
    }

    #[test]
    fn rejects_short_secret_and_bad_auth_header() {
        let short = config_with(WebhookConfig {
            url: Some("https://a.example.com".into()),
            secret: Some("short".into()),
            ..Default::default()
        });
        assert!(Webhook::from_config(&short, no_env).is_err());
        let bad = config_with(WebhookConfig {
            url: Some("https://a.example.com".into()),
            auth_header: Some("no colon here".into()),
            ..Default::default()
        });
        assert!(Webhook::from_config(&bad, no_env).is_err());
    }

    #[test]
    fn errors_and_description_never_contain_secrets() {
        let url = "https://hooks.example.com/services/T000/B000/SECRETPATH?token=QUERYSECRET";
        let cfg = config_with(WebhookConfig {
            url: Some(url.into()),
            secret: Some(SECRET.into()),
            auth_header: Some("Authorization: Bearer HEADERSECRET".into()),
            ..Default::default()
        });
        let w = Webhook::from_config(&cfg, no_env).unwrap().unwrap();
        let shown = format!("{} {:?}", w.describe(), w);
        for secret in ["SECRETPATH", "QUERYSECRET", "HEADERSECRET", SECRET] {
            assert!(!shown.contains(secret), "{shown}");
        }
        assert!(shown.contains("hooks.example.com (signed: yes"));

        let bad = config_with(WebhookConfig {
            url: Some("http://hooks.example.com/SECRETPATH".into()),
            ..Default::default()
        });
        let err = Webhook::from_config(&bad, no_env).unwrap_err().to_string();
        assert!(!err.contains("SECRETPATH"), "{err}");
    }

    #[test]
    fn config_round_trips_through_toml_and_default_is_omitted() {
        let cfg = config_with(WebhookConfig {
            url: Some("https://a.example.com".into()),
            format: WebhookFormat::Slack,
            send: WebhookSend::All,
            ..Default::default()
        });
        let text = toml::to_string_pretty(&cfg).unwrap();
        assert!(text.contains("[webhook]") && text.contains("format = \"slack\""));
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(back, cfg);
        let default_text = toml::to_string_pretty(&Config::default()).unwrap();
        assert!(!default_text.contains("webhook"), "{default_text}");
    }

    // --- bodies and escaping ---

    #[test]
    fn json_body_fields() {
        let w = webhook(
            "https://a.example.com",
            None,
            WebhookFormat::Json,
            WebhookSend::All,
        );
        let body: serde_json::Value = serde_json::from_slice(&w.build_body(
            &alert(HitTiming::OutOfBand, "UA/1"),
            "abc",
            2,
            false,
        ))
        .unwrap();
        assert_eq!(body["version"], 1);
        assert_eq!(body["delivery_id"], "abc");
        assert_eq!(body["deployment"], "honeyprompt.sh");
        assert_eq!(body["suppressed"], 2);
        assert_eq!(body["hit"]["timing"], "out-of-band");
        assert_eq!(body["hit"]["tier_name"], "Computed Callback");
        assert_eq!(body["hit"]["since_seeding_secs"], 259200);
        assert!(body["hit"]["since_page_fetch_secs"].is_null());
    }

    #[test]
    fn user_agent_is_truncated() {
        let w = webhook(
            "https://a.example.com",
            None,
            WebhookFormat::Json,
            WebhookSend::All,
        );
        let long = "x".repeat(1000);
        let body: serde_json::Value = serde_json::from_slice(&w.build_body(
            &alert(HitTiming::Delayed, &long),
            "id",
            0,
            false,
        ))
        .unwrap();
        assert_eq!(
            body["hit"]["user_agent"].as_str().unwrap().len(),
            MAX_ALERT_UA_CHARS
        );
    }

    #[test]
    fn slack_escapes_control_sequences() {
        let w = webhook(
            "https://a.example.com",
            None,
            WebhookFormat::Slack,
            WebhookSend::All,
        );
        let ua = "<!channel> <https://evil.example|click> `x` & more";
        let body: serde_json::Value =
            serde_json::from_slice(&w.build_body(&alert(HitTiming::Delayed, ua), "id", 0, false))
                .unwrap();
        let text = body["text"].as_str().unwrap();
        assert!(!text.contains('<') && !text.contains('>'), "{text}");
        assert!(text.contains("&lt;!channel&gt;"));
        assert!(
            !text.contains("`x`"),
            "backticks inside the UA are neutralised"
        );
        assert!(text.contains("delayed hit on honeyprompt.sh"));
    }

    #[test]
    fn discord_disables_mentions_and_escapes_markdown() {
        let w = webhook(
            "https://a.example.com",
            None,
            WebhookFormat::Discord,
            WebhookSend::All,
        );
        let mut a = alert(
            HitTiming::OutOfBand,
            "@everyone [x](https://evil.example) `y`",
        );
        a.classification = "**bold** @here".into();
        let body: serde_json::Value =
            serde_json::from_slice(&w.build_body(&a, "id", 0, true)).unwrap();
        assert_eq!(body["allowed_mentions"]["parse"], serde_json::json!([]));
        let content = body["content"].as_str().unwrap();
        assert!(content.starts_with("[TEST] ") || content.starts_with("\\[TEST\\] "));
        assert!(content.contains("\\*\\*bold\\*\\* \\@here"), "{content}");
        // The UA sits in a code span with its own backticks neutralised.
        assert!(
            content.contains("`@everyone [x](https://evil.example) 'y'`"),
            "{content}"
        );
    }

    #[test]
    fn notable_mode_filters_in_session() {
        let w = webhook(
            "https://a.example.com",
            None,
            WebhookFormat::Json,
            WebhookSend::Notable,
        );
        assert!(!w.wants(HitTiming::InSession));
        assert!(!w.wants(HitTiming::Unknown));
        assert!(w.wants(HitTiming::Delayed));
        assert!(w.wants(HitTiming::OutOfBand));
    }

    // --- delivery against a local receiver ---

    #[derive(Default)]
    struct Received {
        requests: Vec<(HeaderMap, Vec<u8>)>,
    }

    /// Start a receiver that answers with `statuses` in order (last one repeats).
    async fn receiver(statuses: Vec<u16>) -> (String, Arc<Mutex<Received>>) {
        use axum::routing::post;
        let state = Arc::new(Mutex::new(Received::default()));
        let st = state.clone();
        let app = axum::Router::new()
            .route(
                "/hook",
                post(move |headers: HeaderMap, body: axum::body::Bytes| {
                    let st = st.clone();
                    let statuses = statuses.clone();
                    async move {
                        let mut g = st.lock().unwrap();
                        g.requests.push((headers, body.to_vec()));
                        let i = (g.requests.len() - 1).min(statuses.len() - 1);
                        let code = axum::http::StatusCode::from_u16(statuses[i]).unwrap();
                        if code.is_redirection() {
                            (code, [("location", "http://127.0.0.1:1/elsewhere")]).into_response()
                        } else {
                            code.into_response()
                        }
                    }
                }),
            )
            .route("/elsewhere", post(|| async { axum::http::StatusCode::OK }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (format!("http://127.0.0.1:{}/hook", addr.port()), state)
    }

    use axum::response::IntoResponse;

    fn fast(w: Webhook) -> Webhook {
        w.for_tests(
            vec![Duration::from_millis(10), Duration::from_millis(10)],
            RATE_LIMIT,
        )
    }

    #[tokio::test]
    async fn delivered_request_is_signed_and_verifiable() {
        let (url, rx) = receiver(vec![200]).await;
        let mut w = fast(webhook(
            &url,
            Some(SECRET),
            WebhookFormat::Json,
            WebhookSend::All,
        ));
        w.auth_header = Some((
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Bearer t0ken"),
        ));
        let client = build_client().unwrap();
        let body = w.build_body(&alert(HitTiming::OutOfBand, "UA/1"), "id1", 0, false);
        assert_eq!(
            deliver(&client, &w, "id1", &body, true).await,
            Ok(StatusCode::OK)
        );

        let g = rx.lock().unwrap();
        let (headers, got) = &g.requests[0];
        assert_eq!(got, &body);
        assert_eq!(headers["x-honeyprompt-delivery"], "id1");
        assert_eq!(headers["authorization"], "Bearer t0ken");
        assert_eq!(headers["content-type"], "application/json");
        assert!(headers["user-agent"]
            .to_str()
            .unwrap()
            .starts_with("honeyprompt/"));
        // Verify exactly as a receiver would.
        let sig = headers["x-honeyprompt-signature"].to_str().unwrap();
        let (t, v1) = sig
            .strip_prefix("t=")
            .and_then(|s| s.split_once(",v1="))
            .unwrap();
        assert!((now_secs() - t.parse::<i64>().unwrap()).abs() < 300);
        let mut mac = HmacSha256::new_from_slice(SECRET.as_bytes()).unwrap();
        mac.update(format!("{t}.").as_bytes());
        mac.update(got);
        assert!(mac.verify_slice(&hex::decode(v1).unwrap()).is_ok());
    }

    #[tokio::test]
    async fn unsigned_when_no_secret() {
        let (url, rx) = receiver(vec![204]).await;
        let w = fast(webhook(&url, None, WebhookFormat::Json, WebhookSend::All));
        let client = build_client().unwrap();
        deliver(&client, &w, "id", b"{}", true).await.unwrap();
        assert!(!rx.lock().unwrap().requests[0]
            .0
            .contains_key("x-honeyprompt-signature"));
    }

    #[tokio::test]
    async fn retries_server_errors_and_429_then_succeeds() {
        let (url, rx) = receiver(vec![500, 429, 200]).await;
        let w = fast(webhook(
            &url,
            Some(SECRET),
            WebhookFormat::Json,
            WebhookSend::All,
        ));
        let client = build_client().unwrap();
        assert_eq!(
            deliver(&client, &w, "id", b"{}", true).await,
            Ok(StatusCode::OK)
        );
        let g = rx.lock().unwrap();
        assert_eq!(g.requests.len(), 3);
        assert!(g
            .requests
            .iter()
            .all(|(h, _)| h["x-honeyprompt-delivery"] == "id"));
    }

    #[tokio::test]
    async fn gives_up_after_three_attempts() {
        let (url, rx) = receiver(vec![503]).await;
        let w = fast(webhook(&url, None, WebhookFormat::Json, WebhookSend::All));
        let client = build_client().unwrap();
        assert_eq!(
            deliver(&client, &w, "id", b"{}", true).await,
            Err(DeliveryError::Status(StatusCode::SERVICE_UNAVAILABLE))
        );
        assert_eq!(rx.lock().unwrap().requests.len(), 3);
    }

    #[tokio::test]
    async fn client_errors_are_not_retried() {
        let (url, rx) = receiver(vec![400]).await;
        let w = fast(webhook(&url, None, WebhookFormat::Json, WebhookSend::All));
        let client = build_client().unwrap();
        assert_eq!(
            deliver(&client, &w, "id", b"{}", true).await,
            Err(DeliveryError::Status(StatusCode::BAD_REQUEST))
        );
        assert_eq!(rx.lock().unwrap().requests.len(), 1);
    }

    #[tokio::test]
    async fn redirects_are_not_followed() {
        let (url, rx) = receiver(vec![307]).await;
        let w = fast(webhook(&url, None, WebhookFormat::Json, WebhookSend::All));
        let client = build_client().unwrap();
        let err = deliver(&client, &w, "id", b"{}", true).await.unwrap_err();
        assert_eq!(err, DeliveryError::Status(StatusCode::TEMPORARY_REDIRECT));
        assert!(err.to_string().contains("redirects are not followed"));
        assert_eq!(rx.lock().unwrap().requests.len(), 1);
    }

    #[tokio::test]
    async fn transport_errors_do_not_leak_the_url() {
        // Nothing listens on port 1.
        let w = fast(webhook(
            "http://127.0.0.1:1/SECRETPATH?token=QUERYSECRET",
            None,
            WebhookFormat::Json,
            WebhookSend::All,
        ));
        let client = build_client().unwrap();
        let err = deliver(&client, &w, "id", b"{}", true).await.unwrap_err();
        let shown = err.to_string();
        assert!(matches!(err, DeliveryError::Transport(_)));
        assert!(
            !shown.contains("SECRETPATH") && !shown.contains("QUERYSECRET"),
            "{shown}"
        );
    }

    #[tokio::test]
    async fn task_filters_rate_limits_and_reports_suppressed() {
        let (url, rx) = receiver(vec![200]).await;
        let w = fast(webhook(
            &url,
            None,
            WebhookFormat::Json,
            WebhookSend::Notable,
        ))
        .for_tests(vec![Duration::from_millis(10)], 2);
        let sender = spawn(w).unwrap();
        sender.offer(alert(HitTiming::InSession, "filtered")); // not notable
        for _ in 0..4 {
            sender.offer(alert(HitTiming::OutOfBand, "sent-or-suppressed"));
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
        let g = rx.lock().unwrap();
        assert_eq!(g.requests.len(), 2, "rate limit of 2 per window");
        for (_, body) in &g.requests {
            let v: serde_json::Value = serde_json::from_slice(body).unwrap();
            assert_eq!(v["hit"]["user_agent"], "sent-or-suppressed");
        }
    }

    #[tokio::test]
    async fn suppressed_count_is_reported_in_next_alert() {
        let (url, rx) = receiver(vec![200]).await;
        let mut w =
            fast(webhook(&url, None, WebhookFormat::Json, WebhookSend::All)).for_tests(vec![], 1);
        w.rate_window = Duration::from_millis(150);
        let sender = spawn(w).unwrap();
        for _ in 0..3 {
            sender.offer(alert(HitTiming::Delayed, "a"));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
        sender.offer(alert(HitTiming::Delayed, "b"));
        tokio::time::sleep(Duration::from_millis(150)).await;
        let g = rx.lock().unwrap();
        assert_eq!(g.requests.len(), 2);
        let second: serde_json::Value = serde_json::from_slice(&g.requests[1].1).unwrap();
        assert_eq!(second["hit"]["user_agent"], "b");
        assert_eq!(second["suppressed"], 2);
    }

    fn seed_alert(label: &str, note: Option<&str>, ua: &str) -> SeedAlert {
        SeedAlert {
            received_at: 1_000_000 + 12 * 86400,
            label: label.into(),
            note: note.map(str::to_string),
            nonce: "3a7dd59e33438bc8".into(),
            created_at: 1_000_000,
            source_ip: "198.51.100.9".into(),
            user_agent: ua.into(),
            classification: "Unknown".into(),
        }
    }

    #[test]
    fn seed_json_body() {
        let w = webhook(
            "https://a.example.com",
            None,
            WebhookFormat::Json,
            WebhookSend::Notable,
        );
        let a = Alert::Seed(seed_alert(
            "claude-md-prod",
            Some("api repo"),
            "Harvester/3.0",
        ));
        let v: serde_json::Value =
            serde_json::from_slice(&w.build_alert_body(&a, "id9", 0, false)).unwrap();
        assert_eq!(v["event"], "seed_hit");
        assert_eq!(v["seed"]["label"], "claude-md-prod");
        assert_eq!(v["seed"]["note"], "api repo");
        assert_eq!(v["seed"]["since_seeding_secs"], 12 * 86400);
        assert_eq!(v["hit"]["source_ip"], "198.51.100.9");
        assert_eq!(v["deployment"], "honeyprompt.sh");
    }

    #[test]
    fn seed_chat_formats_escape_label_note_and_ua() {
        let a = Alert::Seed(seed_alert(
            "claude-md",
            Some("<!channel> @everyone **x**"),
            "<https://evil.example|x> `y`",
        ));
        let slack = webhook(
            "https://a.example.com",
            None,
            WebhookFormat::Slack,
            WebhookSend::All,
        );
        let v: serde_json::Value =
            serde_json::from_slice(&slack.build_alert_body(&a, "id", 0, false)).unwrap();
        let text = v["text"].as_str().unwrap();
        assert!(!text.contains('<') && !text.contains('>'), "{text}");
        assert!(text.contains("seed `claude-md`"), "{text}");
        assert!(text.contains("12d 0h after seeding"), "{text}");

        let discord = webhook(
            "https://a.example.com",
            None,
            WebhookFormat::Discord,
            WebhookSend::All,
        );
        let v: serde_json::Value =
            serde_json::from_slice(&discord.build_alert_body(&a, "id", 0, false)).unwrap();
        assert_eq!(v["allowed_mentions"]["parse"], serde_json::json!([]));
        let content = v["content"].as_str().unwrap();
        assert!(content.contains("\\@everyone \\*\\*x\\*\\*"), "{content}");
    }

    #[tokio::test]
    async fn seed_alerts_are_sent_even_in_notable_mode() {
        let (url, rx) = receiver(vec![200]).await;
        let w = fast(webhook(
            &url,
            None,
            WebhookFormat::Json,
            WebhookSend::Notable,
        ));
        let sender = spawn(w).unwrap();
        sender.offer(alert(HitTiming::InSession, "filtered"));
        sender.offer_seed(seed_alert("claude-md", None, "UA"));
        tokio::time::sleep(Duration::from_millis(300)).await;
        let g = rx.lock().unwrap();
        assert_eq!(g.requests.len(), 1);
        let v: serde_json::Value = serde_json::from_slice(&g.requests[0].1).unwrap();
        assert_eq!(v["event"], "seed_hit");
    }
}
