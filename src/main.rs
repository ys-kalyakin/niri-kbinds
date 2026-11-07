use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

use niri_kbinds::l10n;

/// Interactive cheatsheet for niri key binds.
/// (Display strings are applied from the l10n catalog before parsing.)
#[derive(Parser)]
#[command(name = "kbinds", version)]
struct Cli {
    /// Path to the niri config (default: ~/.config/niri/config.kdl)
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List binds in human-readable form
    List {
        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },
    /// Execute a bind by id
    Exec {
        /// Bind id, as shown by `kbinds list --json`
        id: String,
        /// Print the command(s) without executing anything
        #[arg(long)]
        dry_run: bool,
    },
    /// Interactive search over binds via a dmenu-compatible launcher
    Dmenu {
        /// Launcher binary to use
        #[arg(long, default_value = "fuzzel", value_name = "NAME")]
        launcher: String,
    },
    /// Static analysis: combo conflicts and binds invisible in the hotkey overlay
    Audit {
        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
        /// Exit with code 1 when error-level findings are present
        #[arg(long)]
        strict: bool,
    },
    /// GUI overlay «Центр» over the binds (SPEC-V2)
    Overlay {
        /// Print the chosen command(s) to stderr instead of executing
        #[arg(long)]
        dry_run: bool,
        /// Overlay mode (only `center` is implemented)
        #[arg(long, default_value = "center", value_name = "MODE", value_parser = ["center"])]
        mode: String,
        /// Font scale of the overlay text (overrides the theme-cache
        /// `font_scale` token; clamped to 0.5–3.0)
        #[arg(long, value_name = "SCALE")]
        font_scale: Option<f32>,
    },
}

fn main() {
    l10n::init();
    let lang = l10n::current();
    // Derive keeps the parsing structure; the display strings come from
    // the l10n catalog (design D5).
    let mut cmd = Cli::command();
    l10n::apply_cli_texts(&mut cmd, lang);
    let matches = cmd.get_matches();
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(err) => err.exit(),
    };
    match run(cli) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{}: {err:#}", l10n::tr(lang, l10n::Msg::ErrorPrefix));
            std::process::exit(2);
        }
    }
}

fn run(cli: Cli) -> Result<i32> {
    let config_path = niri_kbinds::default_config_path_if_none(cli.config.clone())?;
    match cli.command {
        Command::List { json } => {
            niri_kbinds::cmd_list(&config_path, json)?;
            Ok(0)
        }
        Command::Exec { id, dry_run } => {
            niri_kbinds::cmd_exec(&config_path, &id, dry_run)?;
            Ok(0)
        }
        Command::Dmenu { launcher } => {
            niri_kbinds::cmd_dmenu(&config_path, &launcher)?;
            Ok(0)
        }
        Command::Audit { json, strict } => {
            let has_errors = niri_kbinds::cmd_audit(&config_path, json)?;
            Ok(i32::from(strict && has_errors))
        }
        Command::Overlay {
            dry_run,
            font_scale,
            ..
        } => {
            niri_kbinds::ui::app::run(niri_kbinds::ui::app::OverlayOptions {
                config_path,
                dry_run,
                font_scale,
            })?;
            Ok(0)
        }
    }
}
