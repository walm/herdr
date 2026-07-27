use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::Paragraph,
    Frame,
};

use super::status::agent_icon;
use super::text::{display_width_u16, truncate_end};
use super::widgets::panel_contrast_fg;
use crate::app::state::WorkspaceTabLabel;
use crate::app::AppState;
use crate::detect::AgentState;

const MIN_TAB_WIDTH: u16 = 8;
/// Markers render into a fixed 2-column slot; the API caps reported markers to
/// the same width so a wide glyph can never shift tab geometry.
const MARKER_SLOT_WIDTH: u16 = 2;
const MAX_WORKSPACE_LABEL_WIDTH: usize = 24;

/// The workspace-name label shown at the right of the tab bar, if enabled.
/// `auto` shows it only when the sidebar is collapsed; `on` always; `off` never.
pub(crate) fn workspace_tab_label_text(app: &AppState) -> Option<String> {
    let show = match app.workspace_tab_label {
        WorkspaceTabLabel::Off => false,
        WorkspaceTabLabel::On => true,
        WorkspaceTabLabel::Auto => app.sidebar_collapsed,
    };
    if !show {
        return None;
    }
    let ws = app.active.and_then(|idx| app.workspaces.get(idx))?;
    let name = truncate_end(&ws.display_name(), MAX_WORKSPACE_LABEL_WIDTH);
    Some(format!(" {name} "))
}

/// Columns reserved on the right of the tab bar for the workspace label (0 if none).
pub(crate) fn workspace_tab_label_width(app: &AppState) -> u16 {
    workspace_tab_label_text(app).map_or(0, |label| display_width_u16(&label))
}
const NEW_TAB_WIDTH: u16 = 3;
const TAB_SCROLL_BUTTON_WIDTH: u16 = 3;

#[derive(Debug, Clone, Default)]
pub(crate) struct TabBarView {
    pub scroll: usize,
    pub tab_hit_areas: Vec<Rect>,
    pub scroll_left_hit_area: Rect,
    pub scroll_right_hit_area: Rect,
    pub new_tab_hit_area: Rect,
}

fn tab_width(ws: &crate::workspace::Workspace, tab_idx: usize, number_prefix: bool) -> u16 {
    display_width_u16(&tab_chrome_label(ws, tab_idx, number_prefix))
        .saturating_add(4)
        .max(MIN_TAB_WIDTH)
}

fn tab_chrome_label(
    ws: &crate::workspace::Workspace,
    tab_idx: usize,
    number_prefix: bool,
) -> String {
    let base = ws
        .tab_display_name(tab_idx)
        .unwrap_or_else(|| (tab_idx + 1).to_string());
    let name = if number_prefix {
        let number = (tab_idx + 1).to_string();
        // Unnamed tabs already show their number, so avoid a "1: 1" prefix.
        if base == number {
            base
        } else {
            format!("{number}: {base}")
        }
    } else {
        base
    };
    if ws.tabs.get(tab_idx).is_some_and(|tab| tab.zoomed) {
        format!("{name} Z")
    } else {
        name
    }
}

fn layout_tab_hit_areas(
    ws: &crate::workspace::Workspace,
    area: Rect,
    scroll: usize,
    number_prefix: bool,
) -> Vec<Rect> {
    let mut rects = vec![Rect::default(); ws.tabs.len()];
    if area.width == 0 || area.height == 0 {
        return rects;
    }

    let mut x = area.x;
    let right = area.x + area.width;
    for (idx, rect) in rects.iter_mut().enumerate().skip(scroll) {
        if x >= right {
            break;
        }
        let desired = tab_width(ws, idx, number_prefix);
        let remaining = right.saturating_sub(x);
        let width = desired.min(remaining).max(1);
        *rect = Rect::new(x, area.y, width, 1);
        x = x.saturating_add(width + 1);
    }
    rects
}

fn centered_tab_scroll(ws: &crate::workspace::Workspace, area: Rect, number_prefix: bool) -> usize {
    let mut best_scroll = ws.active_tab;
    let mut best_distance = u16::MAX;
    let viewport_center = area.x.saturating_mul(2).saturating_add(area.width);

    for scroll in 0..=ws.active_tab {
        let rects = layout_tab_hit_areas(ws, area, scroll, number_prefix);
        let Some(active_rect) = rects.get(ws.active_tab).copied() else {
            continue;
        };
        if active_rect.width == 0 {
            continue;
        }

        let active_center = active_rect
            .x
            .saturating_mul(2)
            .saturating_add(active_rect.width);
        let distance = active_center.abs_diff(viewport_center);
        if distance <= best_distance {
            best_distance = distance;
            best_scroll = scroll;
        }
    }

    best_scroll
}

fn trailing_tab_controls_x(tab_hit_areas: &[Rect], fallback_x: u16) -> u16 {
    tab_hit_areas
        .iter()
        .rev()
        .find(|rect| rect.width > 0)
        .map(|rect| rect.x + rect.width)
        .unwrap_or(fallback_x)
}

fn max_tab_scroll(ws: &crate::workspace::Workspace, area: Rect, number_prefix: bool) -> usize {
    (0..ws.tabs.len())
        .find(|&scroll| {
            layout_tab_hit_areas(ws, area, scroll, number_prefix)
                .last()
                .is_some_and(|rect| rect.width > 0)
        })
        .unwrap_or(0)
}

pub(crate) fn compute_tab_bar_view(
    ws: &crate::workspace::Workspace,
    area: Rect,
    current_scroll: usize,
    follow_active: bool,
    mouse_chrome: bool,
    number_prefix: bool,
    label_width: u16,
) -> TabBarView {
    // Reserve the rightmost columns for the workspace label so tabs and trailing
    // controls lay out within the remaining area and never overlap the label.
    let area = Rect {
        width: area.width.saturating_sub(label_width),
        ..area
    };
    if area.width == 0 || area.height == 0 {
        return TabBarView::default();
    }

    if !mouse_chrome {
        let max_scroll = max_tab_scroll(ws, area, number_prefix);
        let scroll = if follow_active {
            centered_tab_scroll(ws, area, number_prefix).min(max_scroll)
        } else {
            current_scroll.min(max_scroll)
        };
        return TabBarView {
            scroll,
            tab_hit_areas: layout_tab_hit_areas(ws, area, scroll, number_prefix),
            scroll_left_hit_area: Rect::default(),
            scroll_right_hit_area: Rect::default(),
            new_tab_hit_area: Rect::default(),
        };
    }

    let area_right = area.x + area.width;
    let all_tabs_area = Rect::new(
        area.x,
        area.y,
        area.width.saturating_sub(NEW_TAB_WIDTH),
        area.height,
    );
    let all_tabs = layout_tab_hit_areas(ws, all_tabs_area, 0, number_prefix);
    let overflow = all_tabs.iter().any(|rect| rect.width == 0);
    if !overflow {
        let new_tab_x = trailing_tab_controls_x(&all_tabs, area.x);
        let new_tab_hit_area = Rect::new(
            new_tab_x,
            area.y,
            area_right.saturating_sub(new_tab_x).min(NEW_TAB_WIDTH),
            1,
        );
        return TabBarView {
            scroll: 0,
            tab_hit_areas: all_tabs,
            scroll_left_hit_area: Rect::default(),
            scroll_right_hit_area: Rect::default(),
            new_tab_hit_area,
        };
    }

    let left_hit_area = Rect::new(area.x, area.y, TAB_SCROLL_BUTTON_WIDTH.min(area.width), 1);
    let tab_area_x = left_hit_area.x + left_hit_area.width;
    let reserved_trailing_width = NEW_TAB_WIDTH.saturating_add(TAB_SCROLL_BUTTON_WIDTH);
    let tab_area_right = area_right.saturating_sub(reserved_trailing_width);
    let tab_area = Rect::new(
        tab_area_x,
        area.y,
        tab_area_right.saturating_sub(tab_area_x),
        area.height,
    );

    let max_scroll = max_tab_scroll(ws, tab_area, number_prefix);
    let scroll = if follow_active {
        centered_tab_scroll(ws, tab_area, number_prefix).min(max_scroll)
    } else {
        current_scroll.min(max_scroll)
    };
    let tab_hit_areas = layout_tab_hit_areas(ws, tab_area, scroll, number_prefix);
    let trailing_x = trailing_tab_controls_x(&tab_hit_areas, tab_area_x).min(tab_area_right);
    let right_hit_area = Rect::new(
        trailing_x,
        area.y,
        area_right
            .saturating_sub(trailing_x)
            .min(TAB_SCROLL_BUTTON_WIDTH),
        1,
    );
    let new_tab_x = right_hit_area.x + right_hit_area.width;
    let new_tab_hit_area = Rect::new(
        new_tab_x,
        area.y,
        area_right.saturating_sub(new_tab_x).min(NEW_TAB_WIDTH),
        1,
    );

    TabBarView {
        scroll,
        tab_hit_areas,
        scroll_left_hit_area: left_hit_area,
        scroll_right_hit_area: right_hit_area,
        new_tab_hit_area,
    }
}

fn tab_drop_indicator_x(
    app: &AppState,
    ws: &crate::workspace::Workspace,
    insert_idx: usize,
) -> Option<u16> {
    let mut visible_tabs = app
        .view
        .tab_hit_areas
        .iter()
        .enumerate()
        .filter(|(_, rect)| rect.width > 0);
    let first_visible = visible_tabs.clone().next()?;
    let last_visible = visible_tabs.next_back().unwrap_or(first_visible);

    if insert_idx == 0 {
        return Some(if first_visible.0 == 0 {
            first_visible.1.x
        } else {
            app.view.tab_scroll_left_hit_area.x + app.view.tab_scroll_left_hit_area.width
        });
    }

    if let Some((_, rect)) = app
        .view
        .tab_hit_areas
        .iter()
        .enumerate()
        .find(|(idx, rect)| *idx == insert_idx && rect.width > 0)
    {
        return Some(rect.x.saturating_sub(1));
    }

    if insert_idx >= ws.tabs.len() {
        return Some(if last_visible.0 + 1 >= ws.tabs.len() {
            last_visible.1.x + last_visible.1.width
        } else {
            app.view.tab_scroll_right_hit_area.x.saturating_sub(1)
        });
    }

    None
}

pub(super) fn render_tab_bar(app: &AppState, frame: &mut Frame, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let Some(active_ws_idx) = app.active else {
        return;
    };
    let Some(ws) = app.workspaces.get(active_ws_idx) else {
        return;
    };
    let p = &app.palette;

    // Fill the whole row, then reserve the rightmost columns for the workspace
    // label by shadowing `area` with the reduced content area (matching the layout
    // computed in `compute_tab_bar_view`); the label is drawn into the strip below.
    let full_area = area;
    frame.render_widget(
        Paragraph::new(" ".repeat(full_area.width as usize)).style(Style::default().bg(p.panel_bg)),
        full_area,
    );
    let workspace_label = workspace_tab_label_text(app);
    let label_w = workspace_label
        .as_ref()
        .map_or(0, |label| display_width_u16(label))
        .min(full_area.width);
    let area = Rect {
        width: full_area.width.saturating_sub(label_w),
        ..full_area
    };

    let first_visible_idx = app
        .view
        .tab_hit_areas
        .iter()
        .enumerate()
        .find(|(_, rect)| rect.width > 0)
        .map(|(idx, _)| idx);
    let last_visible_idx = app
        .view
        .tab_hit_areas
        .iter()
        .enumerate()
        .rev()
        .find(|(_, rect)| rect.width > 0)
        .map(|(idx, _)| idx);
    let can_scroll_left = app.view.tab_scroll_left_hit_area.width > 0 && app.tab_scroll > 0;
    let can_scroll_right = app.view.tab_scroll_right_hit_area.width > 0
        && last_visible_idx.is_some_and(|idx| idx + 1 < ws.tabs.len());

    if app.mouse_capture && app.view.tab_scroll_left_hit_area.width > 0 {
        let style = if can_scroll_left {
            Style::default().fg(p.overlay1).bg(p.surface0)
        } else {
            Style::default()
                .fg(p.overlay0)
                .bg(p.surface0)
                .add_modifier(Modifier::DIM)
        };
        frame.render_widget(
            Paragraph::new(" < ").style(style),
            app.view.tab_scroll_left_hit_area,
        );
    }

    if app.mouse_capture && app.view.tab_scroll_right_hit_area.width > 0 {
        let style = if can_scroll_right {
            Style::default().fg(p.overlay1).bg(p.surface0)
        } else {
            Style::default()
                .fg(p.overlay0)
                .bg(p.surface0)
                .add_modifier(Modifier::DIM)
        };
        frame.render_widget(
            Paragraph::new(" > ").style(style),
            app.view.tab_scroll_right_hit_area,
        );
    }

    for (idx, tab) in ws.tabs.iter().enumerate() {
        let Some(rect) = app.view.tab_hit_areas.get(idx).copied() else {
            break;
        };
        if rect.width == 0 {
            continue;
        }
        let active = idx == ws.active_tab;
        let style = if active {
            let base = Style::default().fg(panel_contrast_fg(p)).bg(p.accent);
            if tab.is_auto_named() {
                base
            } else {
                base.add_modifier(Modifier::BOLD)
            }
        } else if tab.is_auto_named() {
            Style::default()
                .fg(p.overlay0)
                .bg(p.surface0)
                .add_modifier(Modifier::DIM)
        } else {
            Style::default().fg(p.overlay1).bg(p.surface0)
        };
        let width = rect.width as usize;
        let name = tab_chrome_label(ws, idx, app.tab_number_prefix);
        let text = format!(" {:width$}", name, width = width.saturating_sub(1));
        frame.render_widget(Paragraph::new(text).style(style), rect);

        // Agent/CLI-reported marker in a fixed 2-column slot immediately left of
        // the status glyph. Like the glyph it is drawn into the existing trailing
        // padding, so tab widths and hit areas are unchanged. The most recently
        // marked pane in the tab wins.
        if app.tab_markers {
            if let Some((marker, _)) = tab.aggregate_marker(&app.terminals) {
                let name_width = display_width_u16(&name);
                // Reserve the glyph cell only while that feature is on, so the
                // marker sits flush right when it is the only trailing glyph and
                // does not shift as agent state changes.
                let glyph_reserve = if app.tab_agent_status { 1 } else { 0 };
                let slot_start = rect
                    .width
                    .saturating_sub(1 + glyph_reserve + MARKER_SLOT_WIDTH);
                if slot_start >= name_width.saturating_add(2) {
                    let marker_bg = if active { p.accent } else { p.surface0 };
                    let marker_rect = Rect::new(rect.x + slot_start, rect.y, MARKER_SLOT_WIDTH, 1);
                    frame.render_widget(
                        Paragraph::new(marker).style(Style::default().bg(marker_bg)),
                        marker_rect,
                    );
                }
            }
        }

        // Aggregate agent status glyph at the tab's trailing edge. Drawn into the
        // existing trailing padding (no width change) so hit areas stay aligned;
        // only shown for attention states (working / blocked / done).
        if app.tab_agent_status {
            let (state, seen) = tab.aggregate_state(&app.terminals);
            let attention = matches!(
                (state, seen),
                (AgentState::Blocked, _) | (AgentState::Working, _) | (AgentState::Idle, false)
            );
            let name_width = display_width_u16(&name);
            if attention && rect.width >= name_width.saturating_add(3) {
                let (glyph, glyph_style) = agent_icon(state, seen, app.spinner_tick, p);
                let glyph_bg = if active { p.accent } else { p.surface0 };
                let glyph_rect = Rect::new(rect.x + rect.width - 2, rect.y, 1, 1);
                frame.render_widget(
                    Paragraph::new(glyph).style(glyph_style.bg(glyph_bg)),
                    glyph_rect,
                );
            }
        }
    }

    if let Some(crate::app::state::DragState {
        target:
            crate::app::state::DragTarget::TabReorder {
                ws_idx,
                insert_idx: Some(insert_idx),
                ..
            },
    }) = &app.drag
    {
        if *ws_idx == active_ws_idx {
            if let Some(x) = tab_drop_indicator_x(app, ws, *insert_idx) {
                frame.buffer_mut()[(x.min(area.x + area.width.saturating_sub(1)), area.y)]
                    .set_symbol("│")
                    .set_style(Style::default().fg(p.accent));
            }
        }
    }

    if app.mouse_capture && app.view.new_tab_hit_area.width > 0 {
        frame.render_widget(
            Paragraph::new(" + ").style(Style::default().fg(p.overlay1)),
            app.view.new_tab_hit_area,
        );
    }

    if first_visible_idx.is_some_and(|idx| idx > 0) {
        let x = if app.mouse_capture && app.view.tab_scroll_left_hit_area.width > 0 {
            app.view.tab_scroll_left_hit_area.x + app.view.tab_scroll_left_hit_area.width
        } else {
            area.x
        };
        if x < area.x + area.width {
            frame.buffer_mut()[(x, area.y)]
                .set_symbol("…")
                .set_style(Style::default().fg(p.overlay0));
        }
    }
    if last_visible_idx.is_some_and(|idx| idx + 1 < ws.tabs.len()) {
        let x = if app.mouse_capture && app.view.tab_scroll_right_hit_area.width > 0 {
            app.view.tab_scroll_right_hit_area.x.saturating_sub(1)
        } else {
            area.x + area.width.saturating_sub(1)
        };
        if x >= area.x && x < area.x + area.width {
            frame.buffer_mut()[(x, area.y)]
                .set_symbol("…")
                .set_style(Style::default().fg(p.overlay0));
        }
    }

    // Workspace label in the reserved right strip, tinted with its custom color.
    if let Some(label) = workspace_label {
        if label_w > 0 {
            let bg = ws
                .custom_color
                .map(|color| p.workspace_color(color))
                .unwrap_or(p.surface0);
            let fg = super::panes::readable_text_color(bg);
            let label_rect = Rect::new(
                full_area.x + full_area.width - label_w,
                full_area.y,
                label_w,
                1,
            );
            frame.render_widget(
                Paragraph::new(label)
                    .style(Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)),
                label_rect,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;
    use crate::workspace::Workspace;
    use ratatui::{backend::TestBackend, Terminal};

    fn buffer_row_text(buffer: &ratatui::buffer::Buffer, area: Rect, row: u16) -> String {
        (area.x..area.x + area.width)
            .map(|x| buffer[(x, row)].symbol())
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    #[test]
    fn tab_bar_marks_zoomed_tabs_without_renaming_them() {
        let mut app = AppState::test_new();
        let mut ws = Workspace::test_new("test");
        ws.tabs[0].zoomed = true;
        let custom_tab = ws.test_add_tab(Some("test"));
        ws.tabs[custom_tab].zoomed = true;

        app.workspaces = vec![ws];
        app.active = Some(0);
        app.view.tab_bar_rect = Rect::new(0, 0, 30, 1);
        let view = compute_tab_bar_view(
            &app.workspaces[0],
            app.view.tab_bar_rect,
            0,
            true,
            false,
            false,
            0,
        );
        app.view.tab_hit_areas = view.tab_hit_areas;

        let backend = TestBackend::new(30, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_tab_bar(&app, frame, app.view.tab_bar_rect))
            .unwrap();

        let row = buffer_row_text(terminal.backend().buffer(), app.view.tab_bar_rect, 0);
        assert!(row.contains(" 1 Z"), "tab row: {row:?}");
        assert!(row.contains(" test Z"), "tab row: {row:?}");
        assert_eq!(app.workspaces[0].tab_display_name(0).as_deref(), Some("1"));
        assert_eq!(
            app.workspaces[0].tab_display_name(custom_tab).as_deref(),
            Some("test")
        );
    }

    fn tab_bar_row(app: &AppState) -> String {
        let backend = TestBackend::new(30, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_tab_bar(app, frame, app.view.tab_bar_rect))
            .unwrap();
        buffer_row_text(terminal.backend().buffer(), app.view.tab_bar_rect, 0)
    }

    #[test]
    fn tab_markers_render_and_respect_the_toggle() {
        let mut app = AppState::test_new();
        let ws = Workspace::test_new("test");
        let pane = ws.tabs[0].root_pane;
        app.workspaces = vec![ws];
        app.ensure_test_terminals();
        app.tab_markers = true;
        app.tab_agent_status = false;
        app.active = Some(0);
        app.view.tab_bar_rect = Rect::new(0, 0, 30, 1);
        let view = compute_tab_bar_view(
            &app.workspaces[0],
            app.view.tab_bar_rect,
            0,
            true,
            false,
            false,
            0,
        );
        app.view.tab_hit_areas = view.tab_hit_areas;

        let terminal_id = app.workspaces[0].tabs[0].panes[&pane]
            .attached_terminal_id
            .clone();
        app.terminals
            .get_mut(&terminal_id)
            .unwrap()
            .set_agent_metadata(crate::terminal::AgentMetadataReport {
                source: "build".into(),
                agent_label: None,
                applies_to_source: None,
                title: None,
                display_agent: None,
                marker: Some("\u{1f528}".into()),
                state_labels: std::collections::HashMap::new(),
                clear_title: false,
                clear_display_agent: false,
                clear_marker: false,
                clear_state_labels: false,
                ttl: None,
                seq: None,
            });

        assert!(
            tab_bar_row(&app).contains('\u{1f528}'),
            "marker should render: {:?}",
            tab_bar_row(&app)
        );

        // The marker draws into existing trailing padding, so the tab geometry
        // used for hit testing is unchanged.
        let with_marker = compute_tab_bar_view(
            &app.workspaces[0],
            app.view.tab_bar_rect,
            0,
            true,
            false,
            false,
            0,
        );
        assert_eq!(with_marker.tab_hit_areas, app.view.tab_hit_areas);

        app.tab_markers = false;
        assert!(
            !tab_bar_row(&app).contains('\u{1f528}'),
            "toggled off: {:?}",
            tab_bar_row(&app)
        );
    }

    #[test]
    fn tab_agent_status_shows_attention_glyphs_only() {
        use crate::detect::AgentState;

        let mut app = AppState::test_new();
        let ws = Workspace::test_new("test");
        let pane = ws.tabs[0].root_pane;
        app.workspaces = vec![ws];
        app.ensure_test_terminals();
        app.tab_agent_status = true;
        app.spinner_tick = 0; // spinner_frame(0) == "⠋"
        app.active = Some(0);
        app.view.tab_bar_rect = Rect::new(0, 0, 30, 1);
        let view = compute_tab_bar_view(
            &app.workspaces[0],
            app.view.tab_bar_rect,
            0,
            true,
            false,
            false,
            0,
        );
        app.view.tab_hit_areas = view.tab_hit_areas;

        let terminal_id = app.workspaces[0].tabs[0].panes[&pane]
            .attached_terminal_id
            .clone();
        let set_state = |app: &mut AppState, state: AgentState, seen: bool| {
            app.terminals.get_mut(&terminal_id).unwrap().state = state;
            app.workspaces[0].tabs[0].panes.get_mut(&pane).unwrap().seen = seen;
        };

        set_state(&mut app, AgentState::Working, true);
        assert!(
            tab_bar_row(&app).contains('⠋'),
            "working: {:?}",
            tab_bar_row(&app)
        );

        set_state(&mut app, AgentState::Blocked, true);
        assert!(
            tab_bar_row(&app).contains('◉'),
            "blocked: {:?}",
            tab_bar_row(&app)
        );

        set_state(&mut app, AgentState::Idle, false);
        assert!(
            tab_bar_row(&app).contains('●'),
            "done: {:?}",
            tab_bar_row(&app)
        );

        // Idle + seen is quiet: no status glyph.
        set_state(&mut app, AgentState::Idle, true);
        let row = tab_bar_row(&app);
        assert!(
            !row.contains('◉') && !row.contains('●') && !row.contains('⠋') && !row.contains('✓'),
            "idle seen should be quiet: {row:?}"
        );

        // Toggle off hides the glyph even while working.
        set_state(&mut app, AgentState::Working, true);
        app.tab_agent_status = false;
        assert!(!tab_bar_row(&app).contains('⠋'), "toggled off");
    }

    #[test]
    fn workspace_tab_label_visibility_follows_config_and_sidebar() {
        let mut app = AppState::test_new();
        let mut ws = Workspace::test_new("ws");
        ws.set_custom_name("myproject".to_string());
        app.workspaces = vec![ws];
        app.active = Some(0);
        app.view.tab_bar_rect = Rect::new(0, 0, 30, 1);

        fn refresh(app: &mut AppState) {
            let lw = workspace_tab_label_width(app);
            let view = compute_tab_bar_view(
                &app.workspaces[0],
                app.view.tab_bar_rect,
                0,
                true,
                false,
                false,
                lw,
            );
            app.view.tab_hit_areas = view.tab_hit_areas;
        }

        // "on" always shows the label.
        app.workspace_tab_label = WorkspaceTabLabel::On;
        app.sidebar_collapsed = false;
        refresh(&mut app);
        assert!(
            tab_bar_row(&app).contains("myproject"),
            "on: {:?}",
            tab_bar_row(&app)
        );

        // "off" never shows it.
        app.workspace_tab_label = WorkspaceTabLabel::Off;
        refresh(&mut app);
        assert!(!tab_bar_row(&app).contains("myproject"), "off");

        // "auto" shows it only when the sidebar is collapsed.
        app.workspace_tab_label = WorkspaceTabLabel::Auto;
        app.sidebar_collapsed = false;
        refresh(&mut app);
        assert!(
            !tab_bar_row(&app).contains("myproject"),
            "auto + sidebar open"
        );
        app.sidebar_collapsed = true;
        refresh(&mut app);
        assert!(
            tab_bar_row(&app).contains("myproject"),
            "auto + sidebar collapsed"
        );
    }

    #[test]
    fn tab_number_prefix_numbers_named_tabs_only() {
        let mut app = AppState::test_new();
        let mut ws = Workspace::test_new("test");
        // tab 0 is auto-named (shows its number); add a named tab at index 1.
        let named = ws.test_add_tab(Some("main"));
        app.workspaces = vec![ws];
        app.active = Some(0);
        app.tab_number_prefix = true;
        app.view.tab_bar_rect = Rect::new(0, 0, 40, 1);

        let view = compute_tab_bar_view(
            &app.workspaces[0],
            app.view.tab_bar_rect,
            0,
            true,
            false,
            true,
            0,
        );
        app.view.tab_hit_areas = view.tab_hit_areas;

        let backend = TestBackend::new(40, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_tab_bar(&app, frame, app.view.tab_bar_rect))
            .unwrap();

        let row = buffer_row_text(terminal.backend().buffer(), app.view.tab_bar_rect, 0);
        // Named tab is prefixed with its position number; unnamed tab stays "1" (no "1: 1").
        assert!(row.contains("2: main"), "tab row: {row:?}");
        assert!(!row.contains("1: 1"), "tab row: {row:?}");
        assert_eq!(named, 1);
    }

    #[test]
    fn active_auto_named_tab_keeps_readable_weight() {
        let mut app = AppState::test_new();
        let ws = Workspace::test_new("test");

        app.workspaces = vec![ws];
        app.active = Some(0);
        app.view.tab_bar_rect = Rect::new(0, 0, 30, 1);
        let view = compute_tab_bar_view(
            &app.workspaces[0],
            app.view.tab_bar_rect,
            0,
            true,
            false,
            false,
            0,
        );
        app.view.tab_hit_areas = view.tab_hit_areas;

        let backend = TestBackend::new(30, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_tab_bar(&app, frame, app.view.tab_bar_rect))
            .unwrap();

        let tab_rect = app.view.tab_hit_areas[0];
        let style = terminal.backend().buffer()[(tab_rect.x + 1, tab_rect.y)].style();

        assert_eq!(style.bg, Some(app.palette.accent));
        assert!(!style.add_modifier.contains(Modifier::DIM));
        assert!(!style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn zoom_marker_counts_toward_tab_width() {
        let mut ws = Workspace::test_new("test");
        ws.tabs[0].set_custom_name("abcdefgh".into());
        ws.tabs[0].zoomed = true;

        assert_eq!(tab_width(&ws, 0, false), 14);
    }

    #[test]
    fn tab_width_uses_display_width_for_cjk_labels() {
        let mut ws = Workspace::test_new("test");
        ws.tabs[0].set_custom_name("提交 herdr 的反馈".into());

        assert_eq!(
            tab_width(&ws, 0, false),
            display_width_u16("提交 herdr 的反馈") + 4
        );
    }

    #[test]
    fn tab_bar_renders_trailing_cjk_character() {
        let mut app = AppState::test_new();
        let mut ws = Workspace::test_new("test");
        ws.tabs[0].set_custom_name("提交 herdr 的反馈".into());

        app.active = Some(0);
        app.workspaces = vec![ws];
        app.view.tab_bar_rect = Rect::new(0, 0, 30, 1);
        let view = compute_tab_bar_view(
            &app.workspaces[0],
            app.view.tab_bar_rect,
            0,
            true,
            false,
            false,
            0,
        );
        app.view.tab_hit_areas = view.tab_hit_areas;

        let backend = TestBackend::new(30, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_tab_bar(&app, frame, app.view.tab_bar_rect))
            .unwrap();

        let row = buffer_row_text(terminal.backend().buffer(), app.view.tab_bar_rect, 0);
        assert!(row.contains('馈'), "tab row: {row:?}");
    }
}
