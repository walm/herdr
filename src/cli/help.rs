//! Renders `--help`/`-h` from the clap spec in [`super::spec`].
//!
//! Help used to be hand-written `eprintln!` usage strings that drifted from the
//! spec driving shell completions. Rendering both from one tree means a new flag
//! shows up in help and completions at once, and cannot be silently
//! undocumented.

/// Which help variant to render.
///
/// `-h` stays terse for humans skimming; `--help` adds the EXAMPLES block, which
/// is what makes the CLI discoverable to agents exploring it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HelpMode {
    Short,
    Long,
}

/// Classifies a help token, or `None` if it is not one.
pub(super) fn help_mode(arg: &str) -> Option<HelpMode> {
    match arg {
        "-h" => Some(HelpMode::Short),
        "--help" | "help" => Some(HelpMode::Long),
        _ => None,
    }
}

/// The help mode requested by the first help token, defaulting to long.
pub(super) fn requested_mode(args: &[String]) -> HelpMode {
    args.iter()
        .find_map(|arg| help_mode(arg))
        .unwrap_or(HelpMode::Long)
}

/// Renders help for a *leaf* subcommand when help is requested after it, e.g.
/// `herdr pane report-metadata --help`.
///
/// Group dispatchers call this before routing to the leaf parser, which would
/// otherwise reject `--help` as an unknown option or a missing argument.
/// Returns `None` when no help was requested, so dispatch continues normally.
pub(super) fn intercept(base: &[&str], args: &[String]) -> Option<i32> {
    let subcommand = args.first()?;
    // A leading help token means "help for the group", handled by the caller.
    if help_mode(subcommand).is_some() {
        return None;
    }
    let mode = args.iter().skip(1).find_map(|arg| help_mode(arg))?;

    let mut path: Vec<&str> = base.to_vec();
    path.push(subcommand.as_str());

    // `herdr plugin pane open -h` reaches the `plugin` dispatcher first. If this
    // subcommand is itself a group and the next token is a deeper subcommand,
    // let that dispatcher answer so the help matches what was actually asked
    // for, not its parent.
    if resolves_to_group(&path) && args.get(1).is_some_and(|arg| help_mode(arg).is_none()) {
        return None;
    }

    let help = render(&path, mode)?;
    println!("{help}");
    Some(0)
}

fn resolves_to_group(path: &[&str]) -> bool {
    let mut command = super::spec::command();
    for name in path {
        match command.find_subcommand(name) {
            Some(found) => command = found.clone(),
            None => return false,
        }
    }
    command.has_subcommands()
}

fn render(path: &[&str], mode: HelpMode) -> Option<String> {
    let mut command = super::spec::command();
    for name in path {
        command = command.find_subcommand(name)?.clone();
    }
    // Rendering a subcommand in isolation loses its parent context, so the usage
    // line would read `report-metadata …` instead of the command a caller can
    // actually run. Restore the full path.
    let invocation = std::iter::once("herdr")
        .chain(path.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");
    command = command
        .bin_name(invocation.clone())
        .display_name(invocation);
    Some(match mode {
        HelpMode::Short => command.render_help().to_string(),
        HelpMode::Long => command.render_long_help().to_string(),
    })
}

/// Print help for a command path to stdout. Returns the process exit code.
pub(super) fn print(path: &[&str], mode: HelpMode) -> i32 {
    match render(path, mode) {
        Some(help) => {
            println!("{help}");
            0
        }
        // A missing path means the spec and the dispatcher disagree; say so
        // rather than printing nothing.
        None => {
            eprintln!("no help available for: herdr {}", path.join(" "));
            2
        }
    }
}

/// Print help to stderr for an invalid invocation. Returns exit code 2.
///
/// Uses short help: the user made a usage mistake and needs the signature, not
/// the examples.
pub(super) fn usage_error(path: &[&str]) -> i32 {
    match render(path, HelpMode::Short) {
        Some(help) => eprintln!("{help}"),
        None => eprintln!("no help available for: herdr {}", path.join(" ")),
    }
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_tokens_pick_long_or_short() {
        assert_eq!(help_mode("-h"), Some(HelpMode::Short));
        assert_eq!(help_mode("--help"), Some(HelpMode::Long));
        assert_eq!(help_mode("help"), Some(HelpMode::Long));
        assert_eq!(help_mode("--json"), None);
    }

    #[test]
    fn renders_nested_command_paths() {
        let help = render(&["pane", "report-metadata"], HelpMode::Long).expect("help");
        assert!(help.contains("--marker"), "{help}");

        let nested = render(&["plugin", "pane", "open"], HelpMode::Short).expect("help");
        assert!(nested.contains("--entrypoint"), "{nested}");
    }

    #[test]
    fn unknown_path_reports_instead_of_rendering() {
        assert!(render(&["nope"], HelpMode::Long).is_none());
    }

    #[test]
    fn every_dispatched_help_path_exists_in_the_spec() {
        // Guards spec/dispatcher drift: every path the CLI prints help for must
        // resolve, otherwise `herdr <cmd> --help` degrades to an error.
        for path in super::super::HELP_PATHS {
            assert!(
                render(path, HelpMode::Long).is_some(),
                "dispatcher prints help for {path:?} but the spec has no such command"
            );
        }
    }
}
