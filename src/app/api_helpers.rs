pub(super) fn tab_attention_priority(state: crate::detect::AgentState, seen: bool) -> u8 {
    match (state, seen) {
        (crate::detect::AgentState::Blocked, _) => 4,
        (crate::detect::AgentState::Idle, false) => 3,
        (crate::detect::AgentState::Working, _) => 2,
        (crate::detect::AgentState::Idle, true) => 1,
        (crate::detect::AgentState::Unknown, _) => 0,
    }
}

fn parse_api_key(key: &str) -> Option<crossterm::event::KeyEvent> {
    let normalized = normalize_api_key_alias(key.trim());
    let (code, modifiers) = crate::config::parse_key_combo(normalized)?;
    Some(crossterm::event::KeyEvent::new(code, modifiers))
}

fn normalize_api_key_alias(key: &str) -> &str {
    match key {
        "C-c" | "c-c" => "ctrl+c",
        "+" => "plus",
        _ => key,
    }
}

pub(super) fn encode_api_text(runtime: &crate::terminal::TerminalRuntime, text: &str) -> Vec<u8> {
    let bracketed = runtime
        .input_state()
        .map(|state| state.bracketed_paste)
        .unwrap_or(false);
    if bracketed {
        format!("\x1b[200~{text}\x1b[201~").into_bytes()
    } else {
        text.as_bytes().to_vec()
    }
}

pub(super) fn encode_api_keys(
    runtime: &crate::terminal::TerminalRuntime,
    keys: &[String],
) -> Result<Vec<Vec<u8>>, String> {
    let mut encoded_keys = Vec::with_capacity(keys.len());
    for key in keys {
        let Some(key_event) = parse_api_key(key) else {
            return Err(key.clone());
        };
        encoded_keys.push(runtime.encode_terminal_key(key_event.into()));
    }
    Ok(encoded_keys)
}

pub(super) fn detect_state_from_api(
    state: crate::api::schema::PaneAgentState,
) -> crate::detect::AgentState {
    match state {
        crate::api::schema::PaneAgentState::Idle => crate::detect::AgentState::Idle,
        crate::api::schema::PaneAgentState::Working => crate::detect::AgentState::Working,
        crate::api::schema::PaneAgentState::Blocked => crate::detect::AgentState::Blocked,
        crate::api::schema::PaneAgentState::Unknown => crate::detect::AgentState::Unknown,
    }
}

pub(super) fn pane_agent_status(
    state: crate::detect::AgentState,
    seen: bool,
) -> crate::api::schema::AgentStatus {
    match (state, seen) {
        (crate::detect::AgentState::Idle, false) => crate::api::schema::AgentStatus::Done,
        (crate::detect::AgentState::Idle, true) => crate::api::schema::AgentStatus::Idle,
        (crate::detect::AgentState::Working, _) => crate::api::schema::AgentStatus::Working,
        (crate::detect::AgentState::Blocked, _) => crate::api::schema::AgentStatus::Blocked,
        (crate::detect::AgentState::Unknown, _) => crate::api::schema::AgentStatus::Unknown,
    }
}

pub(super) fn normalize_reported_agent_label(agent: &str) -> Option<String> {
    let trimmed = agent.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(agent) = crate::detect::parse_agent_label(trimmed) {
        return Some(crate::detect::agent_label(agent).to_string());
    }
    Some(trimmed.to_string())
}

/// Markers render into a single fixed 2-column slot in the tab bar and sidebar,
/// so they are capped server-side: every client gets the same guarantee and the
/// UI never has to defend against an over-wide glyph shifting its geometry.
pub(super) fn normalize_marker(marker: Option<String>) -> Option<String> {
    const MAX_MARKER_WIDTH: usize = 2;
    use unicode_width::UnicodeWidthStr;

    let trimmed = marker?.trim().to_string();
    let mut normalized = String::new();
    // Width is measured on the accumulated string, not summed per character: a
    // variation selector promotes its base character from one column to two, so
    // per-character widths would under-count emoji presentation sequences.
    for ch in trimmed.chars().filter(|ch| !ch.is_control()).take(32) {
        let fits_len = normalized.len();
        normalized.push(ch);
        if UnicodeWidthStr::width(normalized.as_str()) > MAX_MARKER_WIDTH {
            normalized.truncate(fits_len);
            break;
        }
    }
    // A zero-width joiner left dangling by truncation would render as a stray
    // glyph. Variation selectors are kept: they carry emoji presentation.
    while normalized.ends_with('\u{200d}') {
        normalized.pop();
    }
    (!normalized.is_empty()).then_some(normalized)
}

pub(super) fn normalize_custom_status(status: Option<String>) -> Option<String> {
    let trimmed = status?.trim().to_string();
    let mut normalized = String::new();
    for ch in trimmed.chars().filter(|ch| !ch.is_control()).take(32) {
        normalized.push(ch);
    }
    (!normalized.trim().is_empty()).then(|| normalized.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_marker_keeps_two_column_emoji() {
        assert_eq!(normalize_marker(Some("🔨".into())), Some("🔨".into()));
        // Variation selectors carry emoji presentation and must survive.
        assert_eq!(normalize_marker(Some("⚠️".into())), Some("⚠️".into()));
    }

    #[test]
    fn normalize_marker_trims_and_rejects_empty() {
        assert_eq!(normalize_marker(Some("  ✅  ".into())), Some("✅".into()));
        assert_eq!(normalize_marker(Some("   ".into())), None);
        assert_eq!(normalize_marker(None), None);
    }

    #[test]
    fn normalize_marker_strips_control_chars() {
        assert_eq!(normalize_marker(Some("\u{7}✅".into())), Some("✅".into()));
        assert_eq!(normalize_marker(Some("a\nb".into())), Some("ab".into()));
    }

    #[test]
    fn normalize_marker_caps_at_two_display_columns() {
        use unicode_width::UnicodeWidthStr;

        // Two 2-column emoji: only the first fits the slot.
        assert_eq!(normalize_marker(Some("🔨✅".into())), Some("🔨".into()));
        // Four 1-column chars truncate to two.
        assert_eq!(normalize_marker(Some("abcd".into())), Some("ab".into()));

        for input in ["🔨✅", "abcd", "🔨a", "⚠️⚠️"] {
            let normalized = normalize_marker(Some(input.into())).unwrap();
            assert!(
                UnicodeWidthStr::width(normalized.as_str()) <= 2,
                "{input:?} normalized to {normalized:?} which is wider than the slot"
            );
        }
    }

    #[test]
    fn normalize_marker_drops_dangling_zero_width_joiner() {
        // A ZWJ sequence truncated mid-cluster must not keep a trailing joiner.
        let normalized = normalize_marker(Some("👨\u{200d}👩\u{200d}👧".into())).unwrap();
        assert!(!normalized.ends_with('\u{200d}'), "{normalized:?}");
    }
}
