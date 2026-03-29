use rust_embed::RustEmbed;
use serde::Deserialize;
use anyhow::Context;

use crate::types::AgentClass;

#[derive(RustEmbed)]
#[folder = "assets/crawlers/"]
struct CrawlerCatalogAssets;

/// Internal deserialization struct matching the TOML schema entry.
#[derive(Debug, Deserialize)]
struct CrawlerEntry {
    ua_fragment: String,
    provider: String,
    class: String,
}

/// Intermediate deserialization struct for the catalog file.
#[derive(Debug, Deserialize)]
struct CrawlerCatalogFile {
    crawlers: Vec<CrawlerEntry>,
}

/// Loaded catalog of known AI crawlers and agents.
pub struct CrawlerCatalog {
    entries: Vec<CrawlerEntry>,
}

impl CrawlerCatalog {
    /// Load the embedded TOML crawler catalog.
    pub fn load() -> anyhow::Result<Self> {
        let file = CrawlerCatalogAssets::get("known_crawlers.toml")
            .context("Embedded crawler catalog not found: known_crawlers.toml")?;
        let content = std::str::from_utf8(file.data.as_ref())
            .context("Crawler catalog asset is not valid UTF-8")?;
        let catalog: CrawlerCatalogFile =
            toml::from_str(content).context("Failed to parse crawler catalog TOML")?;
        Ok(CrawlerCatalog {
            entries: catalog.crawlers,
        })
    }

    /// Classify a user-agent string per D-05 (UA-primary matching).
    ///
    /// Iterates entries and returns the first match. If the entry class is
    /// "crawler" returns KnownCrawler; if "agent" returns KnownAgent. An empty
    /// or unmatched user_agent returns Unknown.
    pub fn classify(&self, user_agent: &str) -> AgentClass {
        if user_agent.is_empty() {
            return AgentClass::Unknown;
        }
        for entry in &self.entries {
            if user_agent.contains(entry.ua_fragment.as_str()) {
                let provider = entry.provider.clone();
                return match entry.class.as_str() {
                    "crawler" => AgentClass::KnownCrawler { provider },
                    "agent" => AgentClass::KnownAgent { provider },
                    _ => AgentClass::Unknown,
                };
            }
        }
        AgentClass::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> CrawlerCatalog {
        CrawlerCatalog::load().expect("catalog must load")
    }

    #[test]
    fn test_classify_gptbot_is_known_crawler_openai() {
        let c = catalog();
        let result = c.classify("Mozilla/5.0 (compatible; GPTBot/1.0; +https://openai.com/gptbot)");
        assert_eq!(
            result,
            AgentClass::KnownCrawler { provider: "OpenAI".to_string() }
        );
    }

    #[test]
    fn test_classify_claudebot_is_known_crawler_anthropic() {
        let c = catalog();
        let result = c.classify("Mozilla/5.0 (compatible; ClaudeBot/1.0; +https://claude.ai/bot)");
        assert_eq!(
            result,
            AgentClass::KnownCrawler { provider: "Anthropic".to_string() }
        );
    }

    #[test]
    fn test_classify_googlebot_is_known_crawler_google() {
        let c = catalog();
        let result = c.classify("Mozilla/5.0 (compatible; Googlebot/2.1)");
        assert_eq!(
            result,
            AgentClass::KnownCrawler { provider: "Google".to_string() }
        );
    }

    #[test]
    fn test_classify_unknown_browser_ua() {
        let c = catalog();
        let result = c.classify("Mozilla/5.0 (X11; Linux x86_64)");
        assert_eq!(result, AgentClass::Unknown);
    }

    #[test]
    fn test_classify_empty_ua_returns_unknown() {
        let c = catalog();
        let result = c.classify("");
        assert_eq!(result, AgentClass::Unknown);
    }

    #[test]
    fn test_classify_chatgpt_user_is_known_agent() {
        let c = catalog();
        let result = c.classify("Mozilla/5.0 (compatible; ChatGPT-User/1.0)");
        assert_eq!(
            result,
            AgentClass::KnownAgent { provider: "OpenAI".to_string() }
        );
    }
}
