//! Action execution routed through `niri msg action` (SPEC §8, design D2).
//!
//! Confirmed CLI syntax on niri 26.04: `niri msg action spawn -- <COMMAND>...`
//! and `niri msg action spawn-sh -- <COMMAND>` (a single shell line).

use std::io::Write as _;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::l10n::{self, Msg};
use crate::model::{Action, ActionKind, Bind};

/// Full argv to run for one action, always routed through `niri msg action`.
pub fn action_argv(action: &Action) -> Vec<String> {
    let mut argv: Vec<String> = ["niri", "msg", "action"].map(Into::into).into();
    match action.kind {
        ActionKind::Spawn => {
            argv.push("spawn".into());
            argv.push("--".into());
            argv.extend(action.cmd.clone().unwrap_or_default());
        }
        ActionKind::SpawnSh => {
            argv.push("spawn-sh".into());
            argv.push("--".into());
            argv.extend(action.cmd.clone().unwrap_or_default());
        }
        ActionKind::Builtin => {
            argv.push(action.name.clone().unwrap_or_default());
            argv.extend(action.cmd.clone().unwrap_or_default());
        }
    }
    argv
}

fn needs_quoting(arg: &str) -> bool {
    arg.is_empty()
        || arg
            .chars()
            .any(|c| !(c.is_ascii_alphanumeric() || "-./:=@%^+,".contains(c)))
}

fn shell_quote(arg: &str) -> String {
    if !needs_quoting(arg) {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', "'\\''"))
}

/// Shell-quoted form of the command, used by `--dry-run`.
pub fn display_command(argv: &[String]) -> String {
    argv.iter()
        .map(|a| shell_quote(a))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Fallback when `niri` itself cannot be launched: run the command directly,
/// detached (Linux-only, SPEC §8). `spawn-sh` degrades to `sh -c`.
#[cfg(unix)]
fn spawn_direct(argv: &[String]) -> Result<()> {
    use std::os::unix::process::CommandExt;

    let lang = l10n::current();
    let direct = match argv.first().map(String::as_str) {
        Some("niri") => {
            // Strip the `niri msg action [spawn --]` routing prefix.
            match argv.get(3).map(String::as_str) {
                Some("spawn") => argv[4..].to_vec(),
                Some("spawn-sh") => {
                    let line = argv.get(4).cloned().unwrap_or_default();
                    vec!["sh".to_string(), "-c".to_string(), line]
                }
                _ => bail!("{}", l10n::tr(lang, Msg::FallbackNotPossible)),
            }
        }
        _ => argv.to_vec(),
    };
    let (program, args) = direct
        .split_first()
        .context(l10n::tr(lang, Msg::EmptyCommand))?;
    Command::new(program)
        .args(args)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| l10n::direct_spawn_failed(lang, program))?;
    Ok(())
}

#[cfg(not(unix))]
fn spawn_direct(_argv: &[String]) -> Result<()> {
    bail!("{}", l10n::tr(l10n::current(), Msg::DirectFallbackUnixOnly))
}

fn run_action(action: &Action, dry_run: bool) -> Result<()> {
    let lang = l10n::current();
    let argv = action_argv(action);
    if dry_run {
        let mut out = std::io::stdout().lock();
        if let Err(err) = writeln!(out, "{}", display_command(&argv)) {
            return match err.kind() {
                // Downstream closed the pipe (`| head -1`): stop quietly.
                std::io::ErrorKind::BrokenPipe => Ok(()),
                _ => Err(err).context(l10n::tr(lang, Msg::StdoutWriteFailed)),
            };
        }
        return Ok(());
    }
    let (program, args) = argv
        .split_first()
        .context(l10n::tr(lang, Msg::EmptyCommand))?;
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| l10n::exec_launch_failed(lang, &display_command(&argv)));
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => bail!(
            "{}",
            l10n::exec_exit_code(lang, status.code().unwrap_or(-1), &display_command(&argv))
        ),
        Err(err) => {
            // niri недоступен — прямой запуск для spawn/spawn-sh (design D2).
            if action.kind == ActionKind::Builtin {
                return Err(err).context(l10n::niri_unavailable_builtin(
                    lang,
                    &display_command(&argv),
                ));
            }
            eprintln!(
                "{}",
                l10n::niri_unavailable_direct(lang, &err.root_cause().to_string())
            );
            spawn_direct(&argv)
        }
    }
}

/// Execute every action of the bind in config order; `--dry-run` only prints.
pub fn exec_bind(bind: &Bind, dry_run: bool) -> Result<()> {
    for action in &bind.actions {
        run_action(action, dry_run)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{builtin_with_args, spawn, spawn_sh};

    #[test]
    fn builtin_argv_routes_through_niri_msg() {
        assert_eq!(
            action_argv(&builtin_with_args("close-window", &[])),
            vec!["niri", "msg", "action", "close-window"]
        );
        assert_eq!(
            action_argv(&builtin_with_args("switch-layout", &["next"])),
            vec!["niri", "msg", "action", "switch-layout", "next"]
        );
    }

    #[test]
    fn spawn_argv_uses_double_dash() {
        assert_eq!(
            action_argv(&spawn(&["noctalia", "msg", "panel-toggle", "launcher"])),
            vec![
                "niri",
                "msg",
                "action",
                "spawn",
                "--",
                "noctalia",
                "msg",
                "panel-toggle",
                "launcher"
            ]
        );
    }

    #[test]
    fn spawn_sh_argv_uses_spawn_sh_action() {
        assert_eq!(
            action_argv(&spawn_sh("kitty --class kitty-float")),
            vec![
                "niri",
                "msg",
                "action",
                "spawn-sh",
                "--",
                "kitty --class kitty-float"
            ]
        );
    }

    #[test]
    fn display_command_quotes_when_needed() {
        assert_eq!(
            display_command(&[
                "niri".into(),
                "msg".into(),
                "action".into(),
                "spawn".into(),
                "--".into(),
                "kitty --class kitty-float".into(),
            ]),
            "niri msg action spawn -- 'kitty --class kitty-float'"
        );
        assert_eq!(
            display_command(&["niri".into(), "msg".into(), "action".into(), "quit".into()]),
            "niri msg action quit"
        );
    }
}
