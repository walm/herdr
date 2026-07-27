use std::collections::HashMap;
use std::time::Instant;

use crate::detect::{Agent, AgentState};
use crate::layout::PaneId;
use crate::terminal::{TerminalId, TerminalState};

use super::{Tab, Workspace};

/// Detail info for a single pane, used by the agent detail panel.
pub struct PaneDetail {
    pub pane_id: PaneId,
    pub tab_idx: usize,
    pub tab_label: String,
    pub label: String,
    pub pane_label: Option<String>,
    pub terminal_title: Option<String>,
    pub terminal_title_stripped: Option<String>,
    pub agent_label: String,
    pub agent_kind_label: Option<String>,
    pub agent: Option<Agent>,
    pub state: AgentState,
    pub seen: bool,
    pub last_agent_state_change_seq: Option<u64>,
    pub state_labels: HashMap<String, String>,
    pub tokens: HashMap<String, String>,
}

impl Tab {
    pub fn has_working_pane(&self, terminals: &HashMap<TerminalId, TerminalState>) -> bool {
        self.panes.values().any(|pane| {
            terminals
                .get(&pane.attached_terminal_id)
                .is_some_and(|terminal| terminal.state == AgentState::Working)
        })
    }

    /// Highest-attention agent `(state, seen)` across this tab's panes, using the
    /// same precedence as workspaces (blocked > done > working > idle > unknown).
    /// Defaults to `(Unknown, true)` when the tab has no agent panes.
    pub fn aggregate_state(
        &self,
        terminals: &HashMap<TerminalId, TerminalState>,
    ) -> (AgentState, bool) {
        self.panes
            .values()
            .filter_map(|pane| {
                terminals
                    .get(&pane.attached_terminal_id)
                    .map(|terminal| (terminal.state, pane.seen))
            })
            .max_by_key(|(state, seen)| pane_attention_priority(*state, *seen))
            .unwrap_or((AgentState::Unknown, true))
    }

    /// Marker of the most recently marked pane in this tab, with its report
    /// time. The timestamp travels up so a workspace can rank across its tabs.
    pub fn aggregate_marker(
        &self,
        terminals: &HashMap<TerminalId, TerminalState>,
    ) -> Option<(String, Instant)> {
        self.panes
            .values()
            .filter_map(|pane| {
                terminals
                    .get(&pane.attached_terminal_id)
                    .and_then(|terminal| terminal.effective_marker_at())
            })
            .max_by_key(|(_, reported_at)| *reported_at)
    }

    fn pane_details(
        &self,
        terminals: &HashMap<TerminalId, TerminalState>,
        tab_idx: usize,
        tab_label: &str,
    ) -> Vec<PaneDetail> {
        self.layout
            .pane_ids()
            .iter()
            .filter_map(|id| {
                let pane = self.panes.get(id)?;
                let terminal = terminals.get(&pane.attached_terminal_id)?;
                let agent_kind_label = terminal.effective_agent_label().map(str::to_string);
                let fallback_agent_label = terminal
                    .agent_name
                    .as_deref()
                    .or(agent_kind_label.as_deref())?
                    .to_string();
                let agent_label = terminal
                    .effective_display_agent()
                    .unwrap_or_else(|| fallback_agent_label.clone());
                let presentation = terminal.effective_presentation();
                Some(PaneDetail {
                    pane_id: *id,
                    tab_idx,
                    tab_label: tab_label.to_string(),
                    label: agent_label.clone(),
                    pane_label: terminal
                        .effective_title()
                        .or_else(|| terminal.manual_label.clone()),
                    terminal_title: terminal.terminal_title.clone(),
                    terminal_title_stripped: terminal.terminal_title_stripped(),
                    agent_label,
                    agent_kind_label,
                    agent: terminal.effective_known_agent(),
                    state: terminal.state,
                    seen: pane.seen,
                    last_agent_state_change_seq: terminal.last_agent_state_change_seq,
                    state_labels: presentation.state_labels,
                    tokens: terminal.metadata_tokens.values(),
                })
            })
            .collect()
    }
}

fn pane_attention_priority(state: AgentState, seen: bool) -> u8 {
    match (state, seen) {
        (AgentState::Blocked, _) => 4,
        (AgentState::Idle, false) => 3,
        (AgentState::Working, _) => 2,
        (AgentState::Idle, true) => 1,
        (AgentState::Unknown, _) => 0,
    }
}

impl Workspace {
    pub fn aggregate_state(
        &self,
        terminals: &HashMap<TerminalId, TerminalState>,
    ) -> (AgentState, bool) {
        self.tabs
            .iter()
            .map(|tab| tab.aggregate_state(terminals))
            .max_by_key(|(state, seen)| pane_attention_priority(*state, *seen))
            .unwrap_or((AgentState::Unknown, true))
    }

    pub fn has_working_pane(&self, terminals: &HashMap<TerminalId, TerminalState>) -> bool {
        self.tabs.iter().any(|tab| tab.has_working_pane(terminals))
    }

    /// Marker of the most recently marked tab in this workspace.
    pub fn aggregate_marker(
        &self,
        terminals: &HashMap<TerminalId, TerminalState>,
    ) -> Option<(String, Instant)> {
        self.tabs
            .iter()
            .filter_map(|tab| tab.aggregate_marker(terminals))
            .max_by_key(|(_, reported_at)| *reported_at)
    }

    pub fn pane_details(&self, terminals: &HashMap<TerminalId, TerminalState>) -> Vec<PaneDetail> {
        let multi_tab = self.tabs.len() > 1;
        self.tabs
            .iter()
            .enumerate()
            .flat_map(|(tab_idx, tab)| {
                let tab_label = self
                    .tab_display_name(tab_idx)
                    .unwrap_or_else(|| (tab_idx + 1).to_string());
                tab.pane_details(terminals, tab_idx, &tab_label).into_iter()
            })
            .map(|mut detail| {
                if multi_tab {
                    detail.label = format!("{}·{}", detail.tab_label, detail.agent_label);
                }
                detail
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Direction;

    use super::*;
    use crate::detect::Agent;

    fn terminal_for_pane(ws: &Workspace, pane_id: PaneId) -> TerminalState {
        TerminalState::new(ws.terminal_id(pane_id).unwrap().clone(), "/tmp".into())
    }

    fn set_marker(terminal: &mut TerminalState, source: &str, marker: &str) {
        terminal.set_agent_metadata(crate::terminal::AgentMetadataReport {
            source: source.into(),
            agent_label: None,
            applies_to_source: None,
            title: None,
            display_agent: None,
            marker: Some(marker.into()),
            state_labels: HashMap::new(),
            clear_title: false,
            clear_display_agent: false,
            clear_marker: false,
            clear_state_labels: false,
            ttl: None,
            seq: None,
        });
    }

    #[test]
    fn tab_aggregate_marker_is_none_without_markers() {
        let ws = Workspace::test_new("test");
        let mut terminals = HashMap::new();
        let root = ws.tabs[0].root_pane;
        let terminal = terminal_for_pane(&ws, root);
        terminals.insert(terminal.id.clone(), terminal);

        assert!(ws.tabs[0].aggregate_marker(&terminals).is_none());
        assert!(ws.aggregate_marker(&terminals).is_none());
    }

    #[test]
    fn tab_aggregate_marker_prefers_most_recently_marked_pane() {
        let mut ws = Workspace::test_new("test");
        let id2 = ws.test_split(Direction::Horizontal);
        let root_id = ws.tabs[0]
            .panes
            .keys()
            .find(|id| **id != id2)
            .copied()
            .unwrap();
        let mut terminals = HashMap::new();

        let mut root_terminal = terminal_for_pane(&ws, root_id);
        set_marker(&mut root_terminal, "first", "🔨");
        terminals.insert(root_terminal.id.clone(), root_terminal);

        // Reported after the root pane, so this marker is the newer one.
        let mut second_terminal = terminal_for_pane(&ws, id2);
        set_marker(&mut second_terminal, "second", "✅");
        terminals.insert(second_terminal.id.clone(), second_terminal);

        let (marker, _) = ws.tabs[0].aggregate_marker(&terminals).unwrap();
        assert_eq!(marker, "✅");

        // The workspace ranks across tabs and sees the same winner here.
        let (ws_marker, _) = ws.aggregate_marker(&terminals).unwrap();
        assert_eq!(ws_marker, "✅");
    }

    #[test]
    fn workspace_aggregate_marker_prefers_most_recently_marked_tab() {
        let mut ws = Workspace::test_new("test");
        ws.test_add_tab(None);
        let first_pane = ws.tabs[0].root_pane;
        let second_pane = ws.tabs[1].root_pane;
        let mut terminals = HashMap::new();

        let mut first_terminal = terminal_for_pane(&ws, first_pane);
        set_marker(&mut first_terminal, "first", "🔨");
        terminals.insert(first_terminal.id.clone(), first_terminal);

        let mut second_terminal = terminal_for_pane(&ws, second_pane);
        set_marker(&mut second_terminal, "second", "⚠");
        terminals.insert(second_terminal.id.clone(), second_terminal);

        let (marker, _) = ws.aggregate_marker(&terminals).unwrap();
        assert_eq!(marker, "⚠");
    }

    #[test]
    fn aggregate_state_all_unknown() {
        let ws = Workspace::test_new("test");
        let mut terminals = HashMap::new();
        let root = ws.tabs[0].root_pane;
        let terminal = terminal_for_pane(&ws, root);
        terminals.insert(terminal.id.clone(), terminal);
        let (state, seen) = ws.aggregate_state(&terminals);
        assert_eq!(state, AgentState::Unknown);
        assert!(seen);
    }

    #[test]
    fn aggregate_state_priority() {
        let mut ws = Workspace::test_new("test");
        let id2 = ws.test_split(Direction::Horizontal);
        let root_id = ws.tabs[0]
            .panes
            .keys()
            .find(|id| **id != id2)
            .copied()
            .unwrap();
        let mut terminals = HashMap::new();
        let mut root_terminal = terminal_for_pane(&ws, root_id);
        root_terminal.state = AgentState::Idle;
        terminals.insert(root_terminal.id.clone(), root_terminal);
        let mut second_terminal = terminal_for_pane(&ws, id2);
        second_terminal.state = AgentState::Working;
        terminals.insert(second_terminal.id.clone(), second_terminal);

        let (state, seen) = ws.aggregate_state(&terminals);

        assert_eq!(state, AgentState::Working);
        assert!(seen);
    }

    #[test]
    fn aggregate_state_done_unseen_beats_working() {
        let mut ws = Workspace::test_new("test");
        let id2 = ws.test_split(Direction::Horizontal);
        let root_id = ws.tabs[0]
            .panes
            .keys()
            .find(|id| **id != id2)
            .copied()
            .unwrap();
        let mut terminals = HashMap::new();
        let mut root_terminal = terminal_for_pane(&ws, root_id);
        root_terminal.state = AgentState::Idle;
        terminals.insert(root_terminal.id.clone(), root_terminal);
        let mut second_terminal = terminal_for_pane(&ws, id2);
        second_terminal.state = AgentState::Working;
        terminals.insert(second_terminal.id.clone(), second_terminal);
        let root = ws.tabs[0].panes.get_mut(&root_id).unwrap();
        root.seen = false;

        let (state, seen) = ws.aggregate_state(&terminals);

        assert_eq!(state, AgentState::Idle);
        assert!(!seen);
    }

    #[test]
    fn pane_details_prefers_agent_name_over_detected_agent_label() {
        let ws = Workspace::test_new("test");
        let root_pane = ws.tabs[0].root_pane;
        let mut terminals = HashMap::new();
        let mut terminal = terminal_for_pane(&ws, root_pane);
        terminal.set_detected_state(Some(Agent::Pi), AgentState::Working);
        terminal.set_agent_name("planner".into());
        terminals.insert(terminal.id.clone(), terminal);

        let labels: Vec<_> = ws
            .pane_details(&terminals)
            .into_iter()
            .map(|detail| (detail.label, detail.agent_label, detail.agent))
            .collect();

        assert_eq!(
            labels,
            vec![("planner".into(), "planner".into(), Some(Agent::Pi))]
        );
    }

    #[test]
    fn pane_details_includes_tab_context_for_multi_tab_workspace() {
        let mut ws = Workspace::test_new("test");
        ws.tabs[0].custom_name = Some("main".into());
        let root_pane = ws.tabs[0].root_pane;
        let second_tab = ws.test_add_tab(Some("review"));
        let review_pane = ws.tabs[second_tab].root_pane;
        let mut terminals = HashMap::new();
        let mut root_terminal = terminal_for_pane(&ws, root_pane);
        root_terminal.set_hook_authority(
            "test".into(),
            "pi".into(),
            AgentState::Working,
            None,
            None,
        );
        terminals.insert(root_terminal.id.clone(), root_terminal);
        let mut review_terminal = terminal_for_pane(&ws, review_pane);
        review_terminal.set_hook_authority(
            "test".into(),
            "claude".into(),
            AgentState::Idle,
            None,
            None,
        );
        terminals.insert(review_terminal.id.clone(), review_terminal);

        let labels: Vec<_> = ws
            .pane_details(&terminals)
            .into_iter()
            .map(|detail| (detail.label, detail.agent_label, detail.agent))
            .collect();

        assert_eq!(
            labels,
            vec![
                ("main·pi".into(), "pi".into(), Some(Agent::Pi)),
                ("review·claude".into(), "claude".into(), Some(Agent::Claude)),
            ]
        );
    }

    #[test]
    fn pane_details_use_tab_vector_index_not_stable_public_tab_number() {
        let mut ws = Workspace::test_new("test");
        let removed_tab = ws.test_add_tab(Some("removed"));
        let survivor_tab = ws.test_add_tab(Some("survivor"));
        let survivor_pane = ws.tabs[survivor_tab].root_pane;
        assert!(ws.close_tab(removed_tab));

        let mut terminals = HashMap::new();
        let mut terminal = terminal_for_pane(&ws, survivor_pane);
        terminal.detected_agent = Some(Agent::Codex);
        terminals.insert(terminal.id.clone(), terminal);

        let details = ws.pane_details(&terminals);
        let survivor = details
            .iter()
            .find(|detail| detail.pane_id == survivor_pane)
            .expect("surviving tab agent should be listed");

        assert_eq!(ws.tabs[1].number, 3);
        assert_eq!(survivor.tab_idx, 1);
    }
}
