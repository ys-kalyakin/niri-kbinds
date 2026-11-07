//! `kbinds` — interactive cheatsheet for niri key binds.
//!
//! The library modules are shared between the CLI binary and future
//! integrations (the v2 layer-shell overlay): parsing, normalization,
//! grouping, audit and execution logic live here; the binary stays thin.

pub mod audit;
pub mod exec;
pub mod fuzzel;
pub mod groups;
pub mod l10n;
pub mod model;
pub mod normalize;
pub mod parser;
pub mod ui;

#[cfg(test)]
pub(crate) mod testutil;

use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::audit::render_text;
use crate::l10n::Msg;
use crate::model::Bind;

/// `--config PATH` when given, otherwise the default niri config path.
pub fn default_config_path_if_none(path: Option<PathBuf>) -> Result<PathBuf> {
    path.map(Ok).unwrap_or_else(default_config_path)
}

/// Default niri config path: `$HOME/.config/niri/config.kdl`.
pub fn default_config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").with_context(|| l10n::tr(l10n::current(), Msg::HomeNotSet))?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("niri")
        .join("config.kdl"))
}

fn parse_binds(config_path: &Path) -> Result<Vec<Bind>> {
    parser::parse_config(config_path)
        .with_context(|| l10n::config_parse(l10n::current(), config_path))
}

/// Print lines to stdout, treating a closed pipe (`kbinds list | head`) as
/// success instead of panicking: Rust ignores SIGPIPE by default, so an
/// unwrapped `println!` would abort with a panic (SPEC §16 forbids panics).
/// Any other write error is real (EIO, ENOSPC) and surfaces as an error —
/// the standard exit-code-2 path of `binds-cli`.
pub(crate) fn print_lines<I>(lines: I) -> Result<()>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let lang = l10n::current();
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for line in lines {
        match write_line(&mut out, line.as_ref()) {
            Ok(true) => continue,
            // Downstream closed the pipe — stop quietly.
            Ok(false) => return Ok(()),
            Err(err) => return Err(err).context(l10n::tr(lang, Msg::StdoutWriteFailed)),
        }
    }
    match out.flush() {
        Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err).context(l10n::tr(lang, Msg::StdoutWriteFailed)),
        Ok(()) => Ok(()),
    }
}

/// Write one line to `out`: `Ok(true)` to continue, `Ok(false)` when the
/// downstream pipe closed (stop quietly), `Err` for any real IO error.
fn write_line(out: &mut impl std::io::Write, line: &str) -> std::io::Result<bool> {
    match writeln!(out, "{line}") {
        Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => Ok(false),
        Err(err) => Err(err),
        Ok(()) => Ok(true),
    }
}

/// Entry point of `kbinds list`: human-readable lines, or JSON with `--json`.
pub fn cmd_list(config_path: &Path, json: bool) -> Result<()> {
    let mut binds = parse_binds(config_path)?;
    if json {
        print_lines(serde_json::to_string_pretty(&binds)?.lines())?;
        return Ok(());
    }
    // Design D9: one line per bind, clustered by group name (alphabetical
    // order of group names; the stable sort keeps config order within each
    // group — the overlay's first-appearance order is a separate view).
    binds.sort_by(|a, b| a.group.cmp(&b.group));
    print_lines(binds.iter().map(|bind| {
        format!(
            "[{}] {} — {} ({})",
            bind.group,
            bind.keys_display.join("+"),
            bind.title,
            bind.id
        )
    }))
}

/// Entry point of `kbinds exec`.
pub fn cmd_exec(config_path: &Path, id: &str, dry_run: bool) -> Result<()> {
    let lang = l10n::current();
    let binds = parse_binds(config_path)?;
    let bind = binds
        .iter()
        .find(|b| b.id == id)
        .with_context(|| l10n::bind_not_found(lang, id))?;
    exec::exec_bind(bind, dry_run)
}

/// Entry point of `kbinds dmenu`.
pub fn cmd_dmenu(config_path: &Path, launcher: &str) -> Result<()> {
    let lang = l10n::current();
    if launcher != "fuzzel" {
        bail!("{}", l10n::only_fuzzel(lang, launcher));
    }
    let binds = parse_binds(config_path)?;
    fuzzel::run(launcher, &binds)
}

/// Entry point of `kbinds audit`; returns whether error-level findings exist.
pub fn cmd_audit(config_path: &Path, json: bool) -> Result<bool> {
    let lang = l10n::current();
    let binds = parse_binds(config_path)?;
    let findings = audit::audit(&binds, lang);
    let has_errors = findings.iter().any(|f| f.level == audit::Level::Error);
    let text = if json {
        serde_json::to_string_pretty(&findings)?
    } else {
        render_text(&findings, lang)
            .trim_end_matches('\n')
            .to_string()
    };
    print_lines(text.lines())?;
    Ok(has_errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A writer that always fails with the injected error kind.
    struct FailingWriter {
        kind: std::io::ErrorKind,
    }

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(self.kind, "injected"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::new(self.kind, "injected"))
        }
    }

    /// Broken pipe is a quiet stop; every other write error is real.
    #[test]
    fn write_line_distinguishes_broken_pipe_from_real_errors() {
        let mut broken = FailingWriter {
            kind: std::io::ErrorKind::BrokenPipe,
        };
        assert_eq!(write_line(&mut broken, "x").ok(), Some(false));

        let mut eio = FailingWriter {
            kind: std::io::ErrorKind::Other,
        };
        let err = write_line(&mut eio, "x").expect_err("non-pipe error propagates");
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
    }
}
