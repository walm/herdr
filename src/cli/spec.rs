//! Single source of truth for the CLI surface.
//!
//! This spec drives both shell completions and `--help`/`-h` output. The arg
//! helpers below take their help text as a required parameter, so a new flag
//! cannot be added without documenting it — missing help is a compile error
//! rather than something noticed later.
//!
//! `-h` prints the terse help; `--help` adds the EXAMPLES block registered with
//! [`with_examples`]. Examples are written to be copy-pasteable inside a pane,
//! where `$HERDR_PANE_ID` is already exported.

use clap::{Arg, ArgAction, Command, ValueHint};

pub(super) fn command() -> Command {
    Command::new("herdr")
        .about("terminal workspace manager for AI coding agents")
        .disable_version_flag(true)
        .arg(flag(
            "no-session",
            "Run monolithically without server/client session mode",
        ))
        .arg(option(
            "session",
            "NAME",
            "Use or create a named persistent session",
        ))
        .arg(option(
            "remote",
            "TARGET",
            "Attach through SSH to a remote Herdr server",
        ))
        .arg(
            option(
                "remote-keybindings",
                "MODE",
                "Choose local or server keybindings for remote attach",
            )
            .value_parser(["local", "server"]),
        )
        .arg(flag(
            "handoff",
            "Opt into live handoff for update or remote attach",
        ))
        .arg(flag(
            "default-config",
            "Print default configuration and exit",
        ))
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::SetTrue)
                .help("Print version and exit"),
        )
        .subcommand(completion_command())
        .subcommand(update_command())
        .subcommand(status_command())
        .subcommand(config_command())
        .subcommand(channel_command())
        .subcommand(server_command())
        .subcommand(api_command())
        .subcommand(workspace_command())
        .subcommand(worktree_command())
        .subcommand(tab_command())
        .subcommand(notification_command())
        .subcommand(agent_command())
        .subcommand(pane_command())
        .subcommand(wait_command())
        .subcommand(terminal_command())
        .subcommand(session_command())
        .subcommand(integration_command())
        .subcommand(plugin_command())
        .after_long_help(
            "EXAMPLES:
  # start herdr, or attach to a named persistent session
  herdr
  herdr --session work

  # inspect the running session
  herdr status
  herdr pane list
  herdr agent list

  # scripts and agents address their own pane through $HERDR_PANE_ID
  herdr pane current
  herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --marker \"OK\"",
        )
}

fn completion_command() -> Command {
    with_examples(
        Command::new("completion")
            .visible_alias("completions")
            .about("Generate shell completion scripts")
            .arg(
                Arg::new("shell")
                    .value_name("SHELL")
                    .required(true)
                    .value_parser(super::completion::SUPPORTED_SHELLS)
                    .help("Shell to generate completions for"),
            ),
        "  herdr completion zsh > ~/.zfunc/_herdr
  herdr completion fish > ~/.config/fish/completions/herdr.fish",
    )
}

fn update_command() -> Command {
    with_examples(
        Command::new("update")
            .about("Download and install the latest version")
            .arg(flag("handoff", "Try live handoff after installing")),
        "  herdr update
  herdr update --handoff",
    )
}

fn status_command() -> Command {
    with_examples(
        Command::new("status")
            .about("Show local client and running server status")
            .arg(json_flag())
            .subcommand(
                Command::new("server")
                    .about("Show running server status")
                    .arg(json_flag()),
            )
            .subcommand(
                Command::new("client")
                    .about("Show local client status")
                    .arg(json_flag()),
            ),
        "  herdr status
  herdr status --json
  herdr status server",
    )
}

fn config_command() -> Command {
    with_examples(
        Command::new("config")
            .about("Manage local configuration")
            .subcommand(Command::new("reset-keys").about("Reset custom keybindings")),
        "  herdr --default-config > ~/.config/herdr/config.toml
  herdr config reset-keys",
    )
}

fn channel_command() -> Command {
    with_examples(
        Command::new("channel")
            .about("Manage stable and preview update channels")
            .subcommand(Command::new("show").about("Print the configured update channel"))
            .subcommand(
                Command::new("set").about("Choose the update channel").arg(
                    Arg::new("channel")
                        .value_name("CHANNEL")
                        .required(true)
                        .value_parser(["stable", "preview"])
                        .help("Update channel to follow"),
                ),
            ),
        "  herdr channel show
  herdr channel set preview && herdr update",
    )
}

fn server_command() -> Command {
    with_examples(
        Command::new("server")
            .about("Run or control the headless server")
            .subcommand(Command::new("stop").about("Stop the running server"))
            .subcommand(Command::new("reload-config").about("Reload config in the running server"))
            .subcommand(
                Command::new("agent-manifests")
                    .about("Show active agent detection manifests")
                    .arg(json_flag()),
            )
            .subcommand(
                Command::new("update-agent-manifests")
                    .about("Fetch and reload agent detection manifests")
                    .arg(json_flag()),
            )
            .subcommand(
                Command::new("reload-agent-manifests")
                    .about("Reload local agent detection manifest overrides"),
            ),
        "  herdr server stop
  herdr server reload-config
  herdr server reload-agent-manifests",
    )
}

fn api_command() -> Command {
    with_examples(
        Command::new("api")
            .about("Inspect socket API metadata and live runtime state")
            .subcommand(Command::new("snapshot").about("Print the live session snapshot"))
            .subcommand(
                Command::new("schema")
                    .about("Print or write the bundled API schema")
                    .arg(json_flag())
                    .arg(path_option(
                        "output",
                        "PATH",
                        "Write the schema to this file instead of stdout",
                    )),
            ),
        "  herdr api snapshot
  herdr api schema --output herdr-api.schema.json",
    )
}

fn workspace_command() -> Command {
    with_examples(
        Command::new("workspace")
            .about("Manage workspaces over the socket API")
            .subcommand(Command::new("list").about("List workspaces"))
            .subcommand(
                Command::new("create")
                    .about("Create a workspace")
                    .arg(path_option(
                        "cwd",
                        "PATH",
                        "Working directory for the new workspace",
                    ))
                    .arg(option("label", "TEXT", "Name shown in the sidebar"))
                    .arg(env_option())
                    .args(focus_flags("workspace")),
            )
            .subcommand(id_command(
                "get",
                "workspace_id",
                "Show a workspace",
                "Workspace to show",
            ))
            .subcommand(id_command(
                "focus",
                "workspace_id",
                "Focus a workspace",
                "Workspace to focus",
            ))
            .subcommand(
                Command::new("rename")
                    .about("Rename a workspace")
                    .arg(required(
                        "workspace_id",
                        "WORKSPACE_ID",
                        "Workspace to rename",
                    ))
                    .arg(required("label", "LABEL", "New workspace name").num_args(1..)),
            )
            .subcommand(id_command(
                "close",
                "workspace_id",
                "Close a workspace",
                "Workspace to close",
            )),
        "  herdr workspace list
  herdr workspace create --cwd ~/src/herdr --label herdr
  herdr workspace rename ws1 my project
  herdr workspace close ws1",
    )
}

fn worktree_command() -> Command {
    with_examples(
        Command::new("worktree")
            .about("Manage Git worktree-backed workspaces")
            .subcommand(
                Command::new("list")
                    .about("List worktree workspaces")
                    .arg(option("workspace", "ID", "Limit to this workspace"))
                    .arg(path_option("cwd", "PATH", "Repository to inspect"))
                    .arg(json_flag()),
            )
            .subcommand(
                Command::new("create")
                    .about("Create and open a Git worktree")
                    .arg(option(
                        "workspace",
                        "ID",
                        "Workspace to attach the worktree to",
                    ))
                    .arg(path_option("cwd", "PATH", "Repository to create from"))
                    .arg(option("branch", "NAME", "Branch to create or check out"))
                    .arg(option("base", "REF", "Base revision for a new branch"))
                    .arg(path_option(
                        "path",
                        "PATH",
                        "Checkout path for the worktree",
                    ))
                    .arg(option("label", "TEXT", "Name shown in the sidebar"))
                    .args(focus_flags("worktree workspace"))
                    .arg(json_flag()),
            )
            .subcommand(
                Command::new("open")
                    .about("Open an existing Git worktree")
                    .arg(option(
                        "workspace",
                        "ID",
                        "Workspace to attach the worktree to",
                    ))
                    .arg(path_option("cwd", "PATH", "Repository to open from"))
                    .arg(path_option(
                        "path",
                        "PATH",
                        "Existing worktree checkout path",
                    ))
                    .arg(option("branch", "NAME", "Branch to open"))
                    .arg(option("label", "TEXT", "Name shown in the sidebar"))
                    .args(focus_flags("worktree workspace"))
                    .arg(json_flag()),
            )
            .subcommand(
                Command::new("remove")
                    .about("Remove a worktree checkout")
                    .arg(option("workspace", "ID", "Worktree workspace to remove"))
                    .arg(flag("force", "Remove even with uncommitted changes"))
                    .arg(json_flag()),
            ),
        "  herdr worktree list --json
  herdr worktree create --branch issue/42-fix --base master
  herdr worktree open --path ../herdr-worktrees/issue-42
  herdr worktree remove --workspace ws2",
    )
}

fn tab_command() -> Command {
    with_examples(
        Command::new("tab")
            .about("Manage tabs over the socket API")
            .subcommand(Command::new("list").about("List tabs").arg(option(
                "workspace",
                "WORKSPACE_ID",
                "Limit to this workspace",
            )))
            .subcommand(
                Command::new("create")
                    .about("Create a tab")
                    .arg(option(
                        "workspace",
                        "WORKSPACE_ID",
                        "Workspace to create in",
                    ))
                    .arg(path_option(
                        "cwd",
                        "PATH",
                        "Working directory for the new tab",
                    ))
                    .arg(option("label", "TEXT", "Name shown in the tab bar"))
                    .arg(env_option())
                    .args(focus_flags("tab")),
            )
            .subcommand(id_command("get", "tab_id", "Show a tab", "Tab to show"))
            .subcommand(id_command("focus", "tab_id", "Focus a tab", "Tab to focus"))
            .subcommand(
                Command::new("rename")
                    .about("Rename a tab")
                    .arg(required("tab_id", "TAB_ID", "Tab to rename"))
                    .arg(required("label", "LABEL", "New tab name").num_args(1..)),
            )
            .subcommand(id_command("close", "tab_id", "Close a tab", "Tab to close")),
        "  herdr tab list
  herdr tab create --label build --cwd .
  herdr tab rename tab1 tests
  herdr tab close tab1",
    )
}

fn notification_command() -> Command {
    with_examples(
        Command::new("notification")
            .about("Show Herdr notifications")
            .subcommand(
                Command::new("show")
                    .about("Show a notification")
                    .arg(required("title", "TITLE", "Notification title"))
                    .arg(option("body", "TEXT", "Notification body text"))
                    .arg(
                        option("position", "POSITION", "Where the toast appears").value_parser([
                            "top-left",
                            "top-right",
                            "bottom-left",
                            "bottom-right",
                        ]),
                    )
                    .arg(
                        option("sound", "SOUND", "Sound to play")
                            .value_parser(["none", "done", "request"]),
                    ),
            ),
        "  herdr notification show \"build finished\"
  herdr notification show \"tests failed\" --body \"3 failures\" --sound request",
    )
}

fn agent_command() -> Command {
    with_examples(
        Command::new("agent")
            .about("Control and inspect agent panes")
            .subcommand(Command::new("list").about("List agents"))
            .subcommand(id_command(
                "get",
                "target",
                "Show an agent",
                AGENT_TARGET_HELP,
            ))
            .subcommand(
                Command::new("read")
                    .about("Read agent terminal output")
                    .arg(required("target", "TARGET", AGENT_TARGET_HELP))
                    .arg(read_source_option(true))
                    .arg(lines_option())
                    .arg(text_ansi_format_option())
                    .arg(flag("ansi", "Shorthand for --format ansi")),
            )
            .subcommand(
                Command::new("send")
                    .about("Send text to an agent")
                    .arg(required("target", "TARGET", AGENT_TARGET_HELP))
                    .arg(required("text", "TEXT", "Literal text to send (no Enter)")),
            )
            .subcommand(
                Command::new("rename")
                    .about("Rename an agent")
                    .arg(required("target", "TARGET", AGENT_TARGET_HELP))
                    .arg(
                        Arg::new("name")
                            .value_name("NAME")
                            .help("New agent name; omit with --clear"),
                    )
                    .arg(flag("clear", "Clear the custom name")),
            )
            .subcommand(id_command(
                "focus",
                "target",
                "Focus an agent",
                AGENT_TARGET_HELP,
            ))
            .subcommand(
                Command::new("wait")
                    .about("Wait for an agent status")
                    .arg(required("target", "TARGET", AGENT_TARGET_HELP))
                    .arg(agent_wait_status_option())
                    .arg(timeout_option()),
            )
            .subcommand(
                Command::new("attach")
                    .about("Attach directly to an agent terminal")
                    .arg(required("target", "TARGET", AGENT_TARGET_HELP))
                    .arg(takeover_flag()),
            )
            .subcommand(
                Command::new("start")
                    .about("Start an agent command")
                    .arg(required("name", "NAME", "Agent name to start"))
                    .arg(path_option(
                        "cwd",
                        "PATH",
                        "Working directory for the agent",
                    ))
                    .arg(option("workspace", "ID", "Workspace to start in"))
                    .arg(option("tab", "ID", "Tab to start in"))
                    .arg(split_option())
                    .arg(env_option())
                    .args(focus_flags("agent pane")),
            )
            .subcommand(
                Command::new("explain")
                    .about("Explain agent detection state")
                    .arg(
                        Arg::new("target")
                            .value_name("TARGET")
                            .help(AGENT_TARGET_HELP),
                    )
                    .arg(path_option(
                        "file",
                        "PATH",
                        "Explain a captured screen file instead of a live pane",
                    ))
                    .arg(option("agent", "LABEL", "Agent label to match against"))
                    .arg(json_flag())
                    .arg(text_json_format_option())
                    .arg(
                        Arg::new("verbose")
                            .short('v')
                            .long("verbose")
                            .action(ArgAction::SetTrue)
                            .help("Show every evaluated detection rule"),
                    ),
            ),
        "  herdr agent list
  herdr agent read claude --source detection --format text
  herdr agent wait claude --status idle --timeout 60000
  herdr agent explain claude --json
  herdr agent start claude --cwd . --split right -- --model opus",
    )
}

fn pane_command() -> Command {
    with_examples(
        Command::new("pane")
            .about("Control terminal panes")
            .subcommand(Command::new("list").about("List panes").arg(option(
                "workspace",
                "WORKSPACE_ID",
                "Limit to this workspace",
            )))
            .subcommand(
                Command::new("current")
                    .about("Show the current pane")
                    .args(current_pane_args()),
            )
            .subcommand(id_command("get", "pane_id", "Show a pane", "Pane to show"))
            .subcommand(
                Command::new("layout")
                    .about("Show pane layout information")
                    .args(current_pane_args()),
            )
            .subcommand(
                Command::new("process-info")
                    .about("Show pane process information")
                    .args(current_pane_args()),
            )
            .subcommand(
                Command::new("neighbor")
                    .about("Find a pane neighbor")
                    .arg(direction_option())
                    .args(current_pane_args()),
            )
            .subcommand(
                Command::new("edges")
                    .about("Show pane edge information")
                    .args(current_pane_args()),
            )
            .subcommand(
                Command::new("focus")
                    .about("Focus a neighboring pane")
                    .arg(direction_option())
                    .args(current_pane_args()),
            )
            .subcommand(
                Command::new("resize")
                    .about("Resize a pane split")
                    .arg(direction_option())
                    .arg(option(
                        "amount",
                        "FLOAT",
                        "Fraction to resize by, e.g. 0.05",
                    ))
                    .args(current_pane_args()),
            )
            .subcommand(
                Command::new("zoom")
                    .about("Toggle or set pane zoom")
                    .arg(
                        Arg::new("pane_id")
                            .value_name("PANE_ID")
                            .help("Pane to zoom; defaults to the current pane"),
                    )
                    .args(current_pane_args())
                    .arg(flag("toggle", "Toggle zoom (default)"))
                    .arg(flag("on", "Zoom the pane"))
                    .arg(flag("off", "Unzoom the pane")),
            )
            .subcommand(
                Command::new("read")
                    .about("Read pane terminal output")
                    .arg(required("pane_id", "PANE_ID", "Pane to read"))
                    .arg(read_source_option(true))
                    .arg(lines_option())
                    .arg(text_ansi_format_option())
                    .arg(flag("ansi", "Shorthand for --format ansi"))
                    .arg(flag("raw", "Do not trim trailing blank lines")),
            )
            .subcommand(
                Command::new("rename")
                    .about("Rename a pane")
                    .arg(required("pane_id", "PANE_ID", "Pane to rename"))
                    .arg(
                        Arg::new("label")
                            .value_name("LABEL")
                            .num_args(1..)
                            .help("New pane name; omit with --clear"),
                    )
                    .arg(flag("clear", "Clear the custom name")),
            )
            .subcommand(
                Command::new("split")
                    .about("Split a pane")
                    .arg(
                        Arg::new("pane_id")
                            .value_name("PANE_ID")
                            .help("Pane to split; defaults to the current pane"),
                    )
                    .args(current_pane_args())
                    .arg(split_direction_option())
                    .arg(option("ratio", "FLOAT", "Size of the new pane, 0.0-1.0"))
                    .arg(path_option(
                        "cwd",
                        "PATH",
                        "Working directory for the new pane",
                    ))
                    .arg(env_option())
                    .args(focus_flags("new pane")),
            )
            .subcommand(
                Command::new("swap")
                    .about("Swap panes")
                    .arg(direction_option())
                    .args(current_pane_args())
                    .arg(option("source-pane", "ID", "Pane to swap from"))
                    .arg(option("target-pane", "ID", "Pane to swap with")),
            )
            .subcommand(
                Command::new("move")
                    .about("Move a pane")
                    .arg(required("pane_id", "PANE_ID", "Pane to move"))
                    .arg(option("tab", "TAB_ID", "Destination tab"))
                    .arg(
                        option("split", "DIRECTION", "Split the destination pane this way")
                            .value_parser(["right", "down"]),
                    )
                    .arg(option("target-pane", "ID", "Destination pane to split"))
                    .arg(option("ratio", "FLOAT", "Size of the moved pane, 0.0-1.0"))
                    .arg(flag("new-tab", "Move into a new tab"))
                    .arg(option("workspace", "ID", "Destination workspace"))
                    .arg(flag("new-workspace", "Move into a new workspace"))
                    .arg(option("label", "TEXT", "Name for the moved pane"))
                    .arg(option("tab-label", "TEXT", "Name for a newly created tab"))
                    .args(focus_flags("moved pane")),
            )
            .subcommand(id_command(
                "close",
                "pane_id",
                "Close a pane",
                "Pane to close",
            ))
            .subcommand(
                Command::new("send-text")
                    .about("Send literal text to a pane")
                    .arg(required("pane_id", "PANE_ID", "Pane to send to"))
                    .arg(required("text", "TEXT", "Literal text to send (no Enter)")),
            )
            .subcommand(
                Command::new("send-keys")
                    .about("Send key presses to a pane")
                    .arg(required("pane_id", "PANE_ID", "Pane to send to"))
                    .arg(required("key", "KEY", "Key names, e.g. ctrl+c enter").num_args(1..)),
            )
            .subcommand(
                Command::new("run")
                    .about("Run a command in a pane")
                    .arg(required("pane_id", "PANE_ID", "Pane to run in"))
                    .arg(
                        required("command", "COMMAND", "Command text, sent with Enter")
                            .num_args(1..),
                    ),
            )
            .subcommand(report_agent_command())
            .subcommand(report_agent_session_command())
            .subcommand(release_agent_command())
            .subcommand(report_metadata_command()),
        "  herdr pane list
  herdr pane current
  herdr pane read \"$HERDR_PANE_ID\" --lines 50
  herdr pane split --direction right --cwd .
  herdr pane run \"$HERDR_PANE_ID\" just check

  # report display-only metadata for your own pane
  herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --marker \"OK\"
  herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --clear-marker",
    )
}

fn report_agent_command() -> Command {
    with_examples(
        Command::new("report-agent")
            .about("Report pane agent lifecycle state")
            .long_about(
                "Report pane agent lifecycle state.\n\n\
                 The reporting --source becomes the state authority for the pane, \
                 which suppresses Herdr's own screen detection for it. Use this \
                 from an agent hook that knows its real lifecycle state.",
            )
            .arg(required("pane_id", "PANE_ID", "Pane to report for"))
            .arg(source_option())
            .arg(option("agent", "LABEL", "Agent label to display"))
            .arg(pane_agent_state_option("state"))
            .arg(option("message", "TEXT", "Detail message for the state"))
            .arg(custom_status_option())
            .arg(seq_option())
            .arg(option(
                "agent-session-id",
                "ID",
                "Agent session id to record",
            ))
            .arg(path_option(
                "agent-session-path",
                "PATH",
                "Agent session file to record",
            )),
        "  herdr pane report-agent \"$HERDR_PANE_ID\" --source myagent --state working
  herdr pane report-agent \"$HERDR_PANE_ID\" --source myagent --state blocked \\
    --message \"needs approval\"
  herdr pane report-agent \"$HERDR_PANE_ID\" --source myagent --state idle --seq 3",
    )
}

fn report_agent_session_command() -> Command {
    with_examples(
        Command::new("report-agent-session")
            .about("Report pane agent session identity")
            .arg(required("pane_id", "PANE_ID", "Pane to report for"))
            .arg(source_option())
            .arg(option("agent", "LABEL", "Agent label to display"))
            .arg(seq_option())
            .arg(option(
                "agent-session-id",
                "ID",
                "Agent session id to record",
            ))
            .arg(path_option(
                "agent-session-path",
                "PATH",
                "Agent session file to record",
            ))
            .arg(option(
                "session-start-source",
                "SOURCE",
                "How the session was started",
            )),
        "  herdr pane report-agent-session \"$HERDR_PANE_ID\" --source myagent \\
    --agent-session-id 01J2 --agent-session-path ~/.myagent/sessions/01J2.json",
    )
}

fn release_agent_command() -> Command {
    with_examples(
        Command::new("release-agent")
            .about("Release pane agent lifecycle authority")
            .long_about(
                "Release pane agent lifecycle authority.\n\n\
                 Hands the pane back to Herdr's screen detection after a source \
                 has been reporting state with `report-agent`.",
            )
            .arg(required("pane_id", "PANE_ID", "Pane to release"))
            .arg(source_option())
            .arg(option("agent", "LABEL", "Agent label that held authority"))
            .arg(seq_option()),
        "  herdr pane release-agent \"$HERDR_PANE_ID\" --source myagent",
    )
}

fn report_metadata_command() -> Command {
    with_examples(
        Command::new("report-metadata")
            .about("Report display-only pane metadata")
            .long_about(
                "Report display-only pane metadata.\n\n\
                 Unlike `report-agent`, this never takes over agent state \
                 detection — it only changes what is shown. Any script or agent \
                 can call it for its own pane using $HERDR_PANE_ID.\n\n\
                 --marker sets a short glyph (max 2 columns) shown in the tab bar \
                 and on the sidebar workspace row; --custom-status sets longer \
                 text shown in the agents list. Use --ttl-ms so a forgotten \
                 marker expires on its own.",
            )
            .arg(required("pane_id", "PANE_ID", "Pane to report for"))
            .arg(source_option())
            .arg(option("agent", "LABEL", "Agent label to display"))
            .arg(option(
                "applies-to-source",
                "ID",
                "Only apply while this source holds authority",
            ))
            .arg(option("title", "TEXT", "Override the pane title"))
            .arg(flag("clear-title", "Clear a previously reported title"))
            .arg(option(
                "display-agent",
                "TEXT",
                "Override the displayed agent name",
            ))
            .arg(flag(
                "clear-display-agent",
                "Clear a previously reported agent name",
            ))
            .arg(custom_status_option())
            .arg(flag(
                "clear-custom-status",
                "Clear a previously reported custom status",
            ))
            .arg(option(
                "marker",
                "TEXT",
                "Short glyph for the tab bar, max 2 columns",
            ))
            .arg(flag("clear-marker", "Clear a previously reported marker"))
            .arg(option(
                "state-label",
                "STATUS=TEXT",
                "Rename one status, e.g. working=building",
            ))
            .arg(flag(
                "clear-state-labels",
                "Clear previously reported state labels",
            ))
            .arg(seq_option())
            .arg(option(
                "ttl-ms",
                "N",
                "Expire this metadata after N milliseconds",
            )),
        "  # mark this pane while a build runs, then clear it
  herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --marker \"OK\"
  herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --clear-marker

  # self-expiring marker, and a longer status for the agents list
  herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --marker \"!\" --ttl-ms 30000
  herdr pane report-metadata \"$HERDR_PANE_ID\" --source build --custom-status \"running tests\"",
    )
}

fn wait_command() -> Command {
    with_examples(
        Command::new("wait")
            .about("Wait for pane output or agent state")
            .subcommand(
                Command::new("output")
                    .about("Wait for matching pane output")
                    .arg(required("pane_id", "PANE_ID", "Pane to watch"))
                    .arg(option("match", "TEXT", "Text or pattern to wait for"))
                    .arg(read_source_option(false))
                    .arg(lines_option())
                    .arg(timeout_option())
                    .arg(flag("regex", "Treat --match as a regular expression"))
                    .arg(flag("raw", "Do not trim trailing blank lines")),
            )
            .subcommand(
                Command::new("agent-status")
                    .about("Wait for pane agent status")
                    .arg(required("pane_id", "PANE_ID", "Pane to watch"))
                    .arg(status_option("status", true))
                    .arg(timeout_option()),
            ),
        "  herdr wait output \"$HERDR_PANE_ID\" --match \"BUILD OK\" --timeout 60000
  herdr wait output \"$HERDR_PANE_ID\" --match \"error|failed\" --regex
  herdr wait agent-status pane1 --status idle --timeout 300000",
    )
}

fn terminal_command() -> Command {
    with_examples(
        Command::new("terminal")
            .about("Attach to or observe raw terminal streams")
            .subcommand(
                Command::new("attach")
                    .about("Attach directly to a terminal stream")
                    .arg(required(
                        "terminal_id",
                        "TERMINAL_ID",
                        "Terminal to attach to",
                    ))
                    .arg(takeover_flag()),
            )
            .subcommand(
                Command::new("session")
                    .about("Work with terminal sessions")
                    .subcommand(
                        Command::new("control")
                            .about("Attach to and drive a terminal stream")
                            .arg(required(
                                "target",
                                "TARGET",
                                "Terminal or session to control",
                            ))
                            .arg(takeover_flag())
                            .arg(option("cols", "N", "Attach at this width"))
                            .arg(option("rows", "N", "Attach at this height")),
                    )
                    .subcommand(
                        Command::new("observe")
                            .about("Observe a terminal stream")
                            .arg(required(
                                "target",
                                "TARGET",
                                "Terminal or session to observe",
                            ))
                            .arg(option("cols", "N", "Observe at this width"))
                            .arg(option("rows", "N", "Observe at this height")),
                    ),
            )
            .subcommand(
                Command::new("title")
                    .about("Manage the outer terminal title")
                    .subcommand(
                        Command::new("set")
                            .about("Set the outer terminal title")
                            .arg(required("title", "TITLE", "Title to set")),
                    )
                    .subcommand(Command::new("clear").about("Clear the outer terminal title")),
            ),
        "  herdr terminal attach term1
  herdr terminal session observe term1 --cols 120 --rows 40
  herdr terminal title set \"herdr - build\"",
    )
}

fn session_command() -> Command {
    with_examples(
        Command::new("session")
            .about("Manage named persistent sessions")
            .subcommand(Command::new("list").about("List sessions").arg(json_flag()))
            .subcommand(
                Command::new("attach")
                    .about("Attach to a session")
                    .arg(required("name", "NAME", "Session to attach to")),
            )
            .subcommand(
                Command::new("stop")
                    .about("Stop a session")
                    .arg(required("name", "NAME", "Session to stop"))
                    .arg(json_flag()),
            )
            .subcommand(
                Command::new("delete")
                    .about("Delete a stopped session")
                    .arg(required("name", "NAME", "Session to delete"))
                    .arg(json_flag()),
            ),
        "  herdr session list
  herdr session attach work
  herdr session stop work && herdr session delete work",
    )
}

fn integration_command() -> Command {
    with_examples(
        Command::new("integration")
            .about("Manage built-in agent integrations")
            .long_about(
                "Manage built-in agent integrations.\n\n\
                 Installing an integration adds hooks to the agent's own config \
                 so it reports its lifecycle state to Herdr automatically.",
            )
            .subcommand(
                Command::new("install")
                    .about("Install an integration")
                    .arg(integration_target_arg()),
            )
            .subcommand(
                Command::new("uninstall")
                    .about("Uninstall an integration")
                    .arg(integration_target_arg()),
            )
            .subcommand(
                Command::new("status")
                    .about("Show integration status")
                    .arg(flag(
                        "outdated-only",
                        "Only show integrations needing an update",
                    )),
            ),
        "  herdr integration status
  herdr integration install claude
  herdr integration uninstall claude",
    )
}

fn plugin_command() -> Command {
    with_examples(
        Command::new("plugin")
            .about("Install and run workflow plugins")
            .subcommand(
                Command::new("install")
                    .about("Install a plugin from GitHub")
                    .arg(required(
                        "source",
                        "OWNER/REPO[/SUBDIR]",
                        "GitHub source to install from",
                    ))
                    .arg(option("ref", "REF", "Git ref to install"))
                    .arg(
                        Arg::new("yes")
                            .short('y')
                            .long("yes")
                            .action(ArgAction::SetTrue)
                            .help("Skip the confirmation prompt"),
                    ),
            )
            .subcommand(
                Command::new("uninstall")
                    .about("Uninstall a plugin")
                    .arg(required("plugin", "PLUGIN", "Plugin to uninstall")),
            )
            .subcommand(
                Command::new("link")
                    .about("Link a local plugin")
                    .arg(path_arg("path", "PATH", "Local plugin directory to link"))
                    .arg(flag("disabled", "Link the plugin without enabling it"))
                    .arg(flag("enabled", "Link and enable the plugin")),
            )
            .subcommand(
                Command::new("unlink")
                    .about("Unlink a local plugin")
                    .arg(required("plugin_id", "PLUGIN_ID", "Plugin to unlink")),
            )
            .subcommand(
                Command::new("enable")
                    .about("Enable a plugin")
                    .arg(required("plugin_id", "PLUGIN_ID", "Plugin to enable")),
            )
            .subcommand(
                Command::new("disable")
                    .about("Disable a plugin")
                    .arg(required("plugin_id", "PLUGIN_ID", "Plugin to disable")),
            )
            .subcommand(
                Command::new("list")
                    .about("List installed plugins")
                    .arg(option("plugin", "ID", "Limit to this plugin"))
                    .arg(json_flag()),
            )
            .subcommand(
                Command::new("config-dir")
                    .about("Print a plugin config directory")
                    .arg(required("plugin_id", "PLUGIN_ID", "Plugin to locate")),
            )
            .subcommand(
                Command::new("action")
                    .about("List or invoke plugin actions")
                    .subcommand(
                        Command::new("list")
                            .about("List plugin actions")
                            .arg(option("plugin", "ID", "Limit to this plugin")),
                    )
                    .subcommand(
                        Command::new("invoke")
                            .about("Invoke a plugin action")
                            .arg(required("action_id", "ACTION_ID", "Action to invoke"))
                            .arg(option("plugin", "ID", "Plugin that owns the action")),
                    ),
            )
            .subcommand(
                Command::new("log")
                    .about("Inspect plugin command logs")
                    .visible_alias("logs")
                    .subcommand(
                        Command::new("list")
                            .about("List plugin command logs")
                            .arg(option("plugin", "ID", "Limit to this plugin"))
                            .arg(option("limit", "N", "Maximum number of entries")),
                    ),
            )
            .subcommand(
                Command::new("pane")
                    .about("Manage plugin-owned panes")
                    .subcommand(
                        Command::new("open")
                            .about("Open a plugin pane")
                            .arg(option("plugin", "ID", "Plugin that owns the pane"))
                            .arg(option("entrypoint", "ID", "Plugin entrypoint to run"))
                            .arg(
                                option("placement", "PLACEMENT", "Where to open the pane")
                                    .value_parser(["overlay", "split", "tab", "zoomed"]),
                            )
                            .arg(option("workspace", "ID", "Workspace to open in"))
                            .arg(option("target-pane", "PANE", "Pane to split"))
                            .arg(split_direction_option())
                            .arg(path_option("cwd", "PATH", "Working directory for the pane"))
                            .arg(env_option())
                            .args(focus_flags("plugin pane")),
                    )
                    .subcommand(
                        Command::new("focus")
                            .about("Focus a plugin pane")
                            .arg(required("pane_id", "PANE_ID", "Plugin pane to focus")),
                    )
                    .subcommand(
                        Command::new("close")
                            .about("Close a plugin pane")
                            .arg(required("pane_id", "PANE_ID", "Plugin pane to close")),
                    ),
            ),
        "  herdr plugin list
  herdr plugin install owner/repo --yes
  herdr plugin link ./my-plugin --enabled
  herdr plugin action invoke my-action --plugin my-plugin",
    )
}

const AGENT_TARGET_HELP: &str = "Terminal id, unique agent name, agent label, or legacy pane id";

/// Attaches an EXAMPLES block that only `--help` shows; `-h` stays terse.
fn with_examples(command: Command, examples: &'static str) -> Command {
    command.after_long_help(format!("EXAMPLES:\n{examples}"))
}

fn current_pane_args() -> [Arg; 2] {
    [
        option("pane", "ID", "Pane to act on; defaults to the current pane"),
        flag("current", "Act on the pane this command runs in"),
    ]
}

fn focus_flags(what: &'static str) -> [Arg; 2] {
    [
        Arg::new("focus")
            .long("focus")
            .action(ArgAction::SetTrue)
            .help(leak(format!("Focus the {what} after creating it"))),
        Arg::new("no-focus")
            .long("no-focus")
            .action(ArgAction::SetTrue)
            .help("Leave focus where it is"),
    ]
}

/// clap wants `'static` help text, but a few strings are composed per call site.
/// These are built once while assembling the spec, so leaking is bounded and
/// avoids threading lifetimes through every builder.
fn leak(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn integration_target_arg() -> Arg {
    Arg::new("target")
        .value_name("TARGET")
        .required(true)
        .value_parser([
            "pi", "omp", "claude", "codex", "copilot", "devin", "droid", "kimi", "opencode",
            "kilo", "hermes", "qodercli", "cursor",
        ])
        .help("Agent to install the integration for")
}

fn id_command(
    name: &'static str,
    id: &'static str,
    about: &'static str,
    id_help: &'static str,
) -> Command {
    Command::new(name)
        .about(about)
        .arg(required(id, id, id_help))
}

fn direction_option() -> Arg {
    option("direction", "DIRECTION", "Direction to act in")
        .value_parser(["left", "right", "up", "down"])
}

fn split_option() -> Arg {
    option("split", "DIRECTION", "Split the current pane this way").value_parser(["right", "down"])
}

fn split_direction_option() -> Arg {
    option("direction", "DIRECTION", "Split direction").value_parser(["right", "down"])
}

fn status_option(name: &'static str, required: bool) -> Arg {
    option(name, "STATUS", "Agent status to wait for")
        .required(required)
        .value_parser(["idle", "working", "blocked", "done", "unknown"])
}

fn agent_wait_status_option() -> Arg {
    option("status", "STATUS", "Agent status to wait for")
        .required(true)
        .value_parser(["idle", "working", "blocked", "unknown"])
}

fn pane_agent_state_option(name: &'static str) -> Arg {
    option(name, "STATUS", "Lifecycle state to report")
        .required(true)
        .value_parser(["idle", "working", "blocked", "unknown"])
}

fn read_source_option(include_detection: bool) -> Arg {
    let values = if include_detection {
        vec!["visible", "recent", "recent-unwrapped", "detection"]
    } else {
        vec!["visible", "recent", "recent-unwrapped"]
    };
    option("source", "SOURCE", "Which buffer to read from").value_parser(values)
}

fn text_ansi_format_option() -> Arg {
    option("format", "FORMAT", "Output format").value_parser(["text", "ansi"])
}

fn text_json_format_option() -> Arg {
    option("format", "FORMAT", "Output format").value_parser(["text", "json"])
}

fn json_flag() -> Arg {
    flag("json", "Print JSON output")
}

fn lines_option() -> Arg {
    option("lines", "N", "Number of lines to read")
}

fn timeout_option() -> Arg {
    option("timeout", "MS", "Give up after this many milliseconds")
}

fn seq_option() -> Arg {
    option("seq", "N", "Sequence number; older reports are ignored")
}

fn source_option() -> Arg {
    option("source", "ID", "Identifier for the reporting source")
}

fn custom_status_option() -> Arg {
    option(
        "custom-status",
        "TEXT",
        "Status text shown in the agents list",
    )
}

fn takeover_flag() -> Arg {
    flag("takeover", "Detach any other client already attached")
}

fn env_option() -> Arg {
    option(
        "env",
        "KEY=VALUE",
        "Set an environment variable for the launched process",
    )
    .action(ArgAction::Append)
}

fn flag(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .action(ArgAction::SetTrue)
        .help(help)
}

fn option(name: &'static str, value_name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .value_name(value_name)
        .action(ArgAction::Set)
        .help(help)
}

fn path_option(name: &'static str, value_name: &'static str, help: &'static str) -> Arg {
    option(name, value_name, help).value_hint(ValueHint::AnyPath)
}

fn required(name: &'static str, value_name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .value_name(value_name)
        .required(true)
        .help(help)
}

fn path_arg(name: &'static str, value_name: &'static str, help: &'static str) -> Arg {
    required(name, value_name, help).value_hint(ValueHint::AnyPath)
}

#[cfg(test)]
mod tests {
    use clap::Command;

    fn command_path<'a>(cmd: &'a Command, path: &[&str]) -> &'a Command {
        let mut current = cmd;
        for name in path {
            current = current
                .get_subcommands()
                .find(|subcommand| subcommand.get_name() == *name)
                .unwrap_or_else(|| panic!("missing command path segment {name}"));
        }
        current
    }

    fn option_values(cmd: &Command, option: &str) -> Vec<String> {
        let arg = cmd
            .get_arguments()
            .find(|arg| arg.get_long() == Some(option))
            .unwrap_or_else(|| panic!("missing --{option}"));
        arg.get_value_parser()
            .possible_values()
            .into_iter()
            .flatten()
            .map(|value| value.get_name().to_string())
            .collect()
    }

    fn has_option(cmd: &Command, option: &str) -> bool {
        cmd.get_arguments()
            .any(|arg| arg.get_long() == Some(option))
    }

    fn assert_command_descriptions(cmd: &Command, path: &mut Vec<String>) {
        if !path.is_empty() {
            assert!(
                cmd.get_about().is_some(),
                "missing completion description for {}",
                path.join(" ")
            );
        }
        for subcommand in cmd.get_subcommands() {
            path.push(subcommand.get_name().to_string());
            assert_command_descriptions(subcommand, path);
            path.pop();
        }
    }

    fn assert_arg_descriptions(cmd: &Command, path: &mut Vec<String>) {
        for arg in cmd.get_arguments() {
            // clap injects `help`/`version`; only our own args are our problem.
            if matches!(arg.get_id().as_str(), "help" | "version") {
                continue;
            }
            assert!(
                arg.get_help().is_some(),
                "missing help text for {} argument {}",
                if path.is_empty() {
                    "herdr".to_string()
                } else {
                    path.join(" ")
                },
                arg.get_id()
            );
        }
        for subcommand in cmd.get_subcommands() {
            path.push(subcommand.get_name().to_string());
            assert_arg_descriptions(subcommand, path);
            path.pop();
        }
    }

    #[test]
    fn spec_describes_all_completion_commands() {
        let cmd = super::command();
        assert_command_descriptions(&cmd, &mut Vec::new());
    }

    /// Every flag an agent can discover through `--help` must explain itself.
    /// The arg helpers take help text as a required parameter, so this mainly
    /// guards hand-rolled `Arg::new(..)` sites.
    #[test]
    fn spec_describes_every_argument() {
        let cmd = super::command();
        assert_arg_descriptions(&cmd, &mut Vec::new());
    }

    #[test]
    fn spec_includes_completion_alias_and_shells() {
        let cmd = super::command();
        let completion = command_path(&cmd, &["completion"]);
        assert!(completion
            .get_all_aliases()
            .any(|alias| alias == "completions"));
        let shells = completion
            .get_arguments()
            .find(|arg| arg.get_id() == "shell")
            .unwrap()
            .get_value_parser()
            .possible_values()
            .unwrap()
            .map(|value| value.get_name().to_string())
            .collect::<Vec<_>>();
        assert!(shells.contains(&"zsh".to_string()));
        assert!(shells.contains(&"fish".to_string()));
    }

    #[test]
    fn spec_includes_nested_plugin_pane_open_options() {
        let cmd = super::command();
        let open = command_path(&cmd, &["plugin", "pane", "open"]);
        assert!(open
            .get_arguments()
            .any(|arg| arg.get_long() == Some("entrypoint")));
        assert!(option_values(open, "placement").contains(&"zoomed".to_string()));
    }

    #[test]
    fn spec_includes_agent_status_values() {
        let cmd = super::command();
        let wait = command_path(&cmd, &["agent", "wait"]);
        let values = option_values(wait, "status");
        assert!(values.contains(&"idle".to_string()));
        assert!(values.contains(&"working".to_string()));
        assert!(values.contains(&"blocked".to_string()));
        assert!(!values.contains(&"done".to_string()));
    }

    #[test]
    fn spec_includes_pane_read_raw_flag() {
        let cmd = super::command();
        let pane_read = command_path(&cmd, &["pane", "read"]);
        assert!(has_option(pane_read, "raw"));
    }

    #[test]
    fn spec_matches_pane_split_direction_flag() {
        let cmd = super::command();
        let pane_split = command_path(&cmd, &["pane", "split"]);
        assert!(has_option(pane_split, "direction"));
        assert!(!has_option(pane_split, "split"));
        assert_eq!(option_values(pane_split, "direction"), ["right", "down"]);
    }

    #[test]
    fn spec_does_not_complete_agent_start_argv_without_separator() {
        let cmd = super::command();
        let agent_start = command_path(&cmd, &["agent", "start"]);
        assert!(!agent_start
            .get_arguments()
            .any(|arg| arg.get_id() == "argv"));
    }

    /// Long help carries the EXAMPLES block; short help stays terse.
    #[test]
    fn long_help_shows_examples_and_short_help_does_not() {
        let mut cmd = super::command();
        let pane = cmd
            .find_subcommand_mut("pane")
            .expect("pane command")
            .clone();

        let long = pane.clone().render_long_help().to_string();
        assert!(long.contains("EXAMPLES:"), "{long}");
        assert!(long.contains("report-metadata"), "{long}");

        let short = pane.clone().render_help().to_string();
        assert!(!short.contains("EXAMPLES:"), "{short}");
    }

    /// The marker flag is the kind of thing an agent only finds through help.
    #[test]
    fn report_metadata_long_help_documents_marker() {
        let mut cmd = super::command();
        let pane = cmd.find_subcommand_mut("pane").expect("pane command");
        let mut report = pane
            .find_subcommand_mut("report-metadata")
            .expect("report-metadata command")
            .clone();
        let long = report.render_long_help().to_string();
        assert!(long.contains("--marker"), "{long}");
        assert!(long.contains("--clear-marker"), "{long}");
        assert!(long.contains("HERDR_PANE_ID"), "{long}");
    }

    #[test]
    fn zsh_completion_contains_public_commands_and_values() {
        let mut cmd = super::command();
        let mut output = Vec::new();
        clap_complete::generate(clap_complete::Shell::Zsh, &mut cmd, "herdr", &mut output);
        let script = String::from_utf8(output).unwrap();
        assert!(script.contains("#compdef herdr"));
        assert!(script.contains("--help"));
        assert!(script.contains("'completion:Generate shell completion scripts'"));
        assert!(script.contains("bash elvish fish powershell zsh"));
        assert!(script.contains("'pane:Control terminal panes'"));
        assert!(script.contains("idle working blocked done unknown"));
        assert!(!script.contains("live-handoff"));
    }
}
