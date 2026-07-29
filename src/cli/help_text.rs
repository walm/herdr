//! Help text for every CLI argument, plus per-command examples.
//!
//! This lives apart from [`super::spec`] on purpose. The spec is upstream code
//! we re-merge often, so keeping it byte-identical to upstream means our help
//! text never conflicts. [`apply`] walks the built command tree and fills in
//! `help`/`after_long_help` from the tables below.
//!
//! Two tiers: [`COMMON`] gives an argument name one meaning everywhere, and
//! [`SPECIFIC`] overrides it for a command where the meaning is narrower.
//! Anything reachable from `--help` must be documented — `spec` has a test that
//! fails when an argument has no help text.

use clap::Command;

/// Help by argument name, applied to every command that has that argument.
const COMMON: &[(&str, &str)] = &[
    ("agent", "Agent label to display"),
    ("ansi", "Shorthand for --format ansi"),
    ("branch", "Branch name"),
    ("clear", "Clear the custom name"),
    ("clear-token", "Remove a metadata token by name"),
    ("cols", "Width in columns"),
    ("current", "Act on the pane this command runs in"),
    ("cwd", "Working directory"),
    ("direction", "Direction to act in"),
    ("focus", "Focus it after creating it"),
    ("force", "Proceed even when it is unsafe"),
    ("format", "Output format"),
    ("json", "Print JSON output"),
    ("label", "Display name"),
    ("limit", "Maximum number of entries"),
    ("lines", "Number of lines to read"),
    ("name", "Name to use"),
    ("no-focus", "Leave focus where it is"),
    ("pane", "Pane to act on; defaults to the current pane"),
    ("pane_id", "Pane to act on"),
    ("path", "Filesystem path"),
    ("plugin", "Plugin that owns it"),
    ("plugin_id", "Plugin to act on"),
    ("raw", "Do not trim trailing blank lines"),
    ("ratio", "Size of the new pane, 0.0-1.0"),
    ("rows", "Height in rows"),
    ("seq", "Sequence number; older reports are ignored"),
    ("source", "Identifier for the reporting source"),
    ("tab_id", "Tab to act on"),
    ("takeover", "Detach any other client already attached"),
    ("target-pane", "Pane to target"),
    ("text", "Literal text (no Enter appended)"),
    ("title", "Title text"),
    ("token", "Set a metadata token as NAME=VALUE"),
    ("ttl-ms", "Expire this metadata after N milliseconds"),
    ("workspace", "Workspace to act on"),
    ("workspace_id", "Workspace to act on"),
];

/// Help for one argument of one command. Wins over [`COMMON`].
const SPECIFIC: &[(&str, &str, &str)] = &[
    // agent
    ("agent get", "target", TARGET),
    ("agent read", "target", TARGET),
    ("agent send-keys", "target", TARGET),
    ("agent send-keys", "key", "Key names, e.g. ctrl+c enter"),
    ("agent prompt", "target", TARGET),
    ("agent prompt", "text", "Prompt text to submit"),
    ("agent rename", "target", TARGET),
    ("agent rename", "name", "New agent name; omit with --clear"),
    ("agent focus", "target", TARGET),
    ("agent wait", "target", TARGET),
    ("agent attach", "target", TARGET),
    ("agent start", "name", "Agent to start"),
    (
        "agent start",
        "agent_args",
        "Arguments passed to the agent after --",
    ),
    ("agent explain", "target", TARGET),
    (
        "agent explain",
        "file",
        "Explain a captured screen file instead of a live pane",
    ),
    ("agent explain", "agent", "Agent label to match against"),
    (
        "agent explain",
        "verbose",
        "Show every evaluated detection rule",
    ),
    // pane
    ("pane list", "workspace", "Limit to this workspace"),
    ("pane list", "pinned", "Only list pinned panes"),
    (
        "pane list",
        "unpinned",
        "Only list panes that are not pinned",
    ),
    ("pane resize", "amount", "Fraction to resize by, e.g. 0.05"),
    (
        "pane zoom",
        "pane_id",
        "Pane to zoom; defaults to the current pane",
    ),
    ("pane zoom", "toggle", "Toggle zoom (default)"),
    ("pane zoom", "on", "Zoom the pane"),
    ("pane zoom", "off", "Unzoom the pane"),
    ("pane rename", "label", "New pane name; omit with --clear"),
    (
        "pane split",
        "pane_id",
        "Pane to split; defaults to the current pane",
    ),
    ("pane swap", "source-pane", "Pane to swap from"),
    ("pane swap", "target-pane", "Pane to swap with"),
    ("pane move", "tab", "Destination tab"),
    ("pane move", "split", "Split the destination pane this way"),
    ("pane move", "target-pane", "Destination pane to split"),
    ("pane move", "new-tab", "Move into a new tab"),
    ("pane move", "workspace", "Destination workspace"),
    ("pane move", "new-workspace", "Move into a new workspace"),
    ("pane move", "label", "Name for the moved pane"),
    ("pane move", "tab-label", "Name for a newly created tab"),
    ("pane run", "command", "Command text, sent with Enter"),
    ("pane close", "force", "Close even when the pane is pinned"),
    ("pane pin", "pane_id", "Pane to pin"),
    ("pane unpin", "pane_id", "Pane to unpin"),
    ("pane send-keys", "key", "Key names, e.g. ctrl+c enter"),
    ("pane report-agent", "state", "Lifecycle state to report"),
    (
        "pane report-agent",
        "message",
        "Detail message for the state",
    ),
    (
        "pane report-agent",
        "agent-session-id",
        "Agent session id to record",
    ),
    (
        "pane report-agent",
        "agent-session-path",
        "Agent session file to record",
    ),
    (
        "pane report-agent-session",
        "agent-session-id",
        "Agent session id to record",
    ),
    (
        "pane report-agent-session",
        "agent-session-path",
        "Agent session file to record",
    ),
    (
        "pane report-agent-session",
        "session-start-source",
        "How the session was started",
    ),
    (
        "pane release-agent",
        "agent",
        "Agent label that held authority",
    ),
    (
        "pane report-metadata",
        "applies-to-source",
        "Only apply while this source holds authority",
    ),
    ("pane report-metadata", "title", "Override the pane title"),
    (
        "pane report-metadata",
        "clear-title",
        "Clear a reported title",
    ),
    (
        "pane report-metadata",
        "display-agent",
        "Override the displayed agent name",
    ),
    (
        "pane report-metadata",
        "clear-display-agent",
        "Clear a reported agent name",
    ),
    (
        "pane report-metadata",
        "state-label",
        "Rename one status, e.g. working=building",
    ),
    (
        "pane report-metadata",
        "clear-state-labels",
        "Clear reported state labels",
    ),
    // workspace / worktree / tab
    ("workspace rename", "label", "New workspace name"),
    (
        "workspace report-metadata",
        "source",
        "Identifier for the reporting source",
    ),
    ("worktree create", "base", "Base revision for a new branch"),
    ("worktree create", "path", "Checkout path for the worktree"),
    ("worktree create", "branch", "Branch to create or check out"),
    ("worktree open", "path", "Existing worktree checkout path"),
    ("worktree open", "branch", "Branch to open"),
    (
        "worktree remove",
        "force",
        "Remove even with uncommitted changes",
    ),
    ("worktree list", "cwd", "Repository to inspect"),
    ("tab list", "workspace", "Limit to this workspace"),
    ("tab rename", "label", "New tab name"),
    // misc leaf commands
    ("channel set", "channel", "Update channel to follow"),
    (
        "api schema",
        "output",
        "Write the schema to this file instead of stdout",
    ),
    ("notification show", "title", "Notification title"),
    ("notification show", "body", "Notification body text"),
    ("notification show", "position", "Where the toast appears"),
    ("notification show", "sound", "Sound to play"),
    ("terminal attach", "terminal_id", "Terminal to attach to"),
    (
        "terminal session control",
        "target",
        "Terminal or session to control",
    ),
    (
        "terminal session observe",
        "target",
        "Terminal or session to observe",
    ),
    ("terminal title set", "title", "Title to set"),
    ("session attach", "name", "Session to attach to"),
    ("session stop", "name", "Session to stop"),
    ("session delete", "name", "Session to delete"),
    (
        "integration install",
        "target",
        "Agent to install the integration for",
    ),
    (
        "integration uninstall",
        "target",
        "Agent to uninstall the integration for",
    ),
    (
        "integration status",
        "outdated-only",
        "Only show integrations needing an update",
    ),
    (
        "plugin install",
        "source",
        "GitHub source, OWNER/REPO[/SUBDIR]",
    ),
    ("plugin install", "ref", "Git ref to install"),
    ("plugin install", "yes", "Skip the confirmation prompt"),
    ("plugin uninstall", "plugin", "Plugin to uninstall"),
    ("plugin link", "path", "Local plugin directory to link"),
    (
        "plugin link",
        "disabled",
        "Link the plugin without enabling it",
    ),
    ("plugin link", "enabled", "Link and enable the plugin"),
    ("plugin list", "plugin", "Limit to this plugin"),
    ("plugin action list", "plugin", "Limit to this plugin"),
    ("plugin action invoke", "action_id", "Action to invoke"),
    (
        "plugin action invoke",
        "plugin",
        "Plugin that owns the action",
    ),
    ("plugin log list", "plugin", "Limit to this plugin"),
    ("plugin pane open", "entrypoint", "Plugin entrypoint to run"),
    ("plugin pane open", "placement", "Where to open the pane"),
    ("plugin pane open", "target-pane", "Pane to split"),
    ("plugin pane open", "direction", "Split direction"),
];

const TARGET: &str = "Terminal id, unique agent name, agent label, or legacy pane id";

/// Copy-pasteable examples appended to a command's long help.
///
/// Written to run verbatim inside a pane, where `$HERDR_PANE_ID` is exported.
const EXAMPLES: &[(&str, &str)] = &[
    (
        "pane",
        "  herdr pane list\n  \
         herdr pane current\n  \
         herdr pane read \"$HERDR_PANE_ID\" --lines 50\n  \
         herdr pane split --direction right --cwd .\n  \
         herdr pane run \"$HERDR_PANE_ID\" just check",
    ),
    (
        "pane report-metadata",
        "  # mark this pane while a build runs, then clear it\n  \
         herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --marker \"OK\"\n  \
         herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --clear-marker\n\n  \
         # self-expiring marker\n  \
         herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --marker \"!\" --ttl-ms 30000",
    ),
    (
        "pane pin",
        "  # ask before closing this pane, and its tab\n  \
         herdr pane pin \"$HERDR_PANE_ID\"\n  \
         herdr pane unpin \"$HERDR_PANE_ID\"\n  \
         herdr pane close \"$HERDR_PANE_ID\" --force",
    ),
    (
        "pane report-agent",
        "  herdr pane report-agent \"$HERDR_PANE_ID\" --source myagent --state working\n  \
         herdr pane report-agent \"$HERDR_PANE_ID\" --source myagent --state blocked \\\n    \
         --message \"needs approval\"\n  \
         herdr pane report-agent \"$HERDR_PANE_ID\" --source myagent --state idle --seq 3",
    ),
    (
        "agent",
        "  herdr agent list\n  \
         herdr agent read claude --source detection --format text\n  \
         herdr agent wait claude --status idle --timeout 60000\n  \
         herdr agent explain claude --json",
    ),
    (
        "workspace",
        "  herdr workspace list\n  \
         herdr workspace create --cwd ~/src/herdr --label herdr\n  \
         herdr workspace close ws1",
    ),
    (
        "worktree",
        "  herdr worktree list --json\n  \
         herdr worktree create --branch issue/42-fix --base master\n  \
         herdr worktree remove --workspace ws2",
    ),
    (
        "tab",
        "  herdr tab list\n  \
         herdr tab create --label build --cwd .\n  \
         herdr tab close tab1",
    ),
    (
        "session",
        "  herdr session list\n  \
         herdr session attach work\n  \
         herdr session stop work && herdr session delete work",
    ),
    (
        "terminal",
        "  herdr terminal attach term1\n  \
         herdr terminal session observe term1 --cols 120 --rows 40\n  \
         herdr terminal title set \"herdr - build\"",
    ),
    (
        "integration",
        "  herdr integration status\n  \
         herdr integration install claude",
    ),
    (
        "plugin",
        "  herdr plugin list\n  \
         herdr plugin install owner/repo --yes\n  \
         herdr plugin link ./my-plugin --enabled",
    ),
    (
        "status",
        "  herdr status\n  \
         herdr status --json\n  \
         herdr status server",
    ),
    (
        "channel",
        "  herdr channel show\n  \
         herdr channel set preview && herdr update",
    ),
    (
        "completion",
        "  herdr completion zsh > ~/.zfunc/_herdr\n  \
         herdr completion fish > ~/.config/fish/completions/herdr.fish",
    ),
];

fn help_for(path: &str, arg: &str) -> Option<&'static str> {
    SPECIFIC
        .iter()
        .find(|(cmd, id, _)| *cmd == path && *id == arg)
        .map(|(_, _, help)| *help)
        .or_else(|| {
            COMMON
                .iter()
                .find(|(id, _)| *id == arg)
                .map(|(_, help)| *help)
        })
}

fn examples_for(path: &str) -> Option<&'static str> {
    EXAMPLES
        .iter()
        .find(|(cmd, _)| *cmd == path)
        .map(|(_, examples)| *examples)
}

/// Fill in help text and examples across the whole command tree.
pub(super) fn apply(command: Command) -> Command {
    apply_at(command, String::new())
}

fn apply_at(command: Command, path: String) -> Command {
    let command = command.mut_args(|arg| {
        if arg.get_help().is_some() {
            return arg;
        }
        let id = arg.get_id().to_string();
        match help_for(&path, &id) {
            Some(help) => arg.help(help),
            None => arg,
        }
    });

    let command = match examples_for(&path) {
        // Keep any examples the spec already set; ours are the fallback.
        Some(examples) if command.get_after_long_help().is_none() => {
            command.after_long_help(format!("EXAMPLES:\n{examples}"))
        }
        _ => command,
    };

    command.mut_subcommands(move |sub| {
        let child = if path.is_empty() {
            sub.get_name().to_string()
        } else {
            format!("{path} {}", sub.get_name())
        };
        apply_at(sub, child)
    })
}
