use crate::types::AppEvent;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TierFilter {
    All,
    T1,
    T2,
    T3,
}

impl TierFilter {
    pub fn next(self) -> Self {
        match self {
            TierFilter::All => TierFilter::T1,
            TierFilter::T1 => TierFilter::T2,
            TierFilter::T2 => TierFilter::T3,
            TierFilter::T3 => TierFilter::All,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortField {
    Time,
    Tier,
    Source,
}

impl SortField {
    pub fn next(self) -> Self {
        match self {
            SortField::Time => SortField::Tier,
            SortField::Tier => SortField::Source,
            SortField::Source => SortField::Time,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiMode {
    Normal,
    Command,
    Help,
}

pub struct AppState {
    pub events: Vec<AppEvent>,
    pub filter: TierFilter,
    pub sort: SortField,
    pub show_replays: bool,
    pub table_state: ratatui::widgets::TableState,
    pub at_bottom: bool,
    pub new_events_count: usize,
    pub mode: UiMode,
    pub command_input: String,
    pub command_error: Option<(String, std::time::Instant)>,
    pub status_line: String,
}

impl AppState {
    pub fn new() -> Self {
        let mut table_state = ratatui::widgets::TableState::default();
        table_state.select(Some(0));
        AppState {
            events: Vec::new(),
            filter: TierFilter::All,
            sort: SortField::Time,
            show_replays: false,
            table_state,
            at_bottom: true,
            new_events_count: 0,
            mode: UiMode::Normal,
            command_input: String::new(),
            command_error: None,
            status_line: String::new(),
        }
    }

    pub fn push_event(&mut self, event: AppEvent) {
        self.events.push(event);
        if !self.at_bottom {
            self.new_events_count += 1;
        }
    }

    pub fn visible_events(&self) -> Vec<&AppEvent> {
        let mut result: Vec<&AppEvent> = self
            .events
            .iter()
            .filter(|e| {
                // Replay filter
                if !self.show_replays && e.is_replay {
                    return false;
                }
                // Tier filter
                match self.filter {
                    TierFilter::All => true,
                    TierFilter::T1 => e.tier == 1,
                    TierFilter::T2 => e.tier == 2,
                    TierFilter::T3 => e.tier == 3,
                }
            })
            .collect();

        match self.sort {
            SortField::Time => {
                result.sort_by(|a, b| b.received_at.cmp(&a.received_at));
            }
            SortField::Tier => {
                result.sort_by(|a, b| {
                    a.tier
                        .cmp(&b.tier)
                        .then_with(|| b.received_at.cmp(&a.received_at))
                });
            }
            SortField::Source => {
                result.sort_by(|a, b| {
                    a.fingerprint
                        .source_ip
                        .to_string()
                        .cmp(&b.fingerprint.source_ip.to_string())
                        .then_with(|| b.received_at.cmp(&a.received_at))
                });
            }
        }

        result
    }

    pub fn detection_count(&self) -> usize {
        self.events.iter().filter(|e| !e.is_replay).count()
    }

    pub fn session_count(&self) -> usize {
        let unique: HashSet<&str> = self
            .events
            .iter()
            .filter(|e| !e.is_replay)
            .map(|e| e.session_id.as_str())
            .collect();
        unique.len()
    }

    pub fn tier_counts(&self) -> (usize, usize, usize) {
        let non_replays: Vec<&AppEvent> = self.events.iter().filter(|e| !e.is_replay).collect();
        let t1 = non_replays.iter().filter(|e| e.tier == 1).count();
        let t2 = non_replays.iter().filter(|e| e.tier == 2).count();
        let t3 = non_replays.iter().filter(|e| e.tier == 3).count();
        (t1, t2, t3)
    }

    pub fn replay_count(&self) -> usize {
        self.events.iter().filter(|e| e.is_replay).count()
    }

    pub fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
        self.table_state.select(Some(0));
    }

    pub fn cycle_sort(&mut self) {
        self.sort = self.sort.next();
    }

    pub fn toggle_replays(&mut self) {
        self.show_replays = !self.show_replays;
        self.table_state.select(Some(0));
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::net::IpAddr;
    use crate::types::{AgentFingerprint, AgentClass};

    fn make_test_event(
        tier: u8,
        is_replay: bool,
        ip: &str,
        session_id: &str,
        received_at: u64,
    ) -> AppEvent {
        let source_ip: IpAddr = ip.parse().unwrap();
        AppEvent {
            nonce: "nonce123".to_string(),
            tier,
            payload_id: format!("t{}-test", tier),
            embedding_loc: "html_comment".to_string(),
            fingerprint: AgentFingerprint {
                source_ip,
                user_agent: "TestAgent/1.0".to_string(),
                headers: HashMap::new(),
                received_at,
            },
            classification: AgentClass::Unknown,
            session_id: session_id.to_string(),
            is_replay,
            fire_count: 1,
            received_at,
        }
    }

    #[test]
    fn test_push_event_appends() {
        let mut state = AppState::new();
        assert_eq!(state.events.len(), 0);
        let ev = make_test_event(1, false, "1.2.3.4", "sess1", 1000);
        state.push_event(ev);
        assert_eq!(state.events.len(), 1);
    }

    #[test]
    fn test_visible_events_filter_all_excludes_replays_by_default() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, true, "2.3.4.5", "sess2", 2000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert!(!visible[0].is_replay);
    }

    #[test]
    fn test_visible_events_filter_t1_returns_only_tier1() {
        let mut state = AppState::new();
        state.filter = TierFilter::T1;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(2, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(3, false, "3.4.5.6", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].tier, 1);
    }

    #[test]
    fn test_visible_events_filter_t2_returns_only_tier2() {
        let mut state = AppState::new();
        state.filter = TierFilter::T2;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(2, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(3, false, "3.4.5.6", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].tier, 2);
    }

    #[test]
    fn test_visible_events_filter_t3_returns_only_tier3() {
        let mut state = AppState::new();
        state.filter = TierFilter::T3;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(2, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(3, false, "3.4.5.6", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].tier, 3);
    }

    #[test]
    fn test_visible_events_show_replays_false_excludes_replays() {
        let mut state = AppState::new();
        state.show_replays = false;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, true, "2.3.4.5", "sess2", 2000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].is_replay, false);
    }

    #[test]
    fn test_visible_events_show_replays_true_includes_replays() {
        let mut state = AppState::new();
        state.show_replays = true;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, true, "2.3.4.5", "sess2", 2000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn test_visible_events_sort_time_newest_first() {
        let mut state = AppState::new();
        state.sort = SortField::Time;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess2", 3000));
        state.push_event(make_test_event(1, false, "3.4.5.6", "sess3", 2000));
        let visible = state.visible_events();
        assert_eq!(visible[0].received_at, 3000);
        assert_eq!(visible[1].received_at, 2000);
        assert_eq!(visible[2].received_at, 1000);
    }

    #[test]
    fn test_visible_events_sort_tier_ascending() {
        let mut state = AppState::new();
        state.sort = SortField::Tier;
        state.push_event(make_test_event(3, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(2, false, "3.4.5.6", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible[0].tier, 1);
        assert_eq!(visible[1].tier, 2);
        assert_eq!(visible[2].tier, 3);
    }

    #[test]
    fn test_visible_events_sort_source_ascending() {
        let mut state = AppState::new();
        state.sort = SortField::Source;
        state.push_event(make_test_event(1, false, "3.4.5.6", "sess1", 1000));
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess2", 2000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible[0].fingerprint.source_ip.to_string(), "1.2.3.4");
        assert_eq!(visible[1].fingerprint.source_ip.to_string(), "2.3.4.5");
        assert_eq!(visible[2].fingerprint.source_ip.to_string(), "3.4.5.6");
    }

    #[test]
    fn test_detection_count_excludes_replays() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(1, true, "3.4.5.6", "sess3", 3000));
        assert_eq!(state.detection_count(), 2);
    }

    #[test]
    fn test_session_count_unique_sessions() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess1", 2000)); // same session
        state.push_event(make_test_event(1, false, "3.4.5.6", "sess2", 3000));
        state.push_event(make_test_event(1, true, "4.5.6.7", "sess3", 4000)); // replay excluded
        assert_eq!(state.session_count(), 2);
    }

    #[test]
    fn test_tier_counts_excludes_replays() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(2, false, "3.4.5.6", "sess3", 3000));
        state.push_event(make_test_event(3, false, "4.5.6.7", "sess4", 4000));
        state.push_event(make_test_event(1, true, "5.6.7.8", "sess5", 5000)); // replay
        let (t1, t2, t3) = state.tier_counts();
        assert_eq!(t1, 2);
        assert_eq!(t2, 1);
        assert_eq!(t3, 1);
    }

    #[test]
    fn test_replay_count() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, true, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(1, true, "3.4.5.6", "sess3", 3000));
        assert_eq!(state.replay_count(), 2);
    }

    #[test]
    fn test_handle_filter_cycle() {
        let mut state = AppState::new();
        assert_eq!(state.filter, TierFilter::All);
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::T1);
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::T2);
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::T3);
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::All);
    }

    #[test]
    fn test_handle_sort_cycle() {
        let mut state = AppState::new();
        assert_eq!(state.sort, SortField::Time);
        state.cycle_sort();
        assert_eq!(state.sort, SortField::Tier);
        state.cycle_sort();
        assert_eq!(state.sort, SortField::Source);
        state.cycle_sort();
        assert_eq!(state.sort, SortField::Time);
    }

    #[test]
    fn test_handle_replay_toggle() {
        let mut state = AppState::new();
        assert!(!state.show_replays);
        state.toggle_replays();
        assert!(state.show_replays);
        state.toggle_replays();
        assert!(!state.show_replays);
    }
}
