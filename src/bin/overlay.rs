//! `kbinds-overlay` binary: thin entry over the shared overlay UI
//! (`niri_kbinds::ui::app`), see SPEC-V2 §5.

use std::path::PathBuf;

use clap::{CommandFactory, FromArgMatches, Parser};

use niri_kbinds::l10n;

/// GUI cheatsheet overlay for niri key binds
/// (display strings are applied from the l10n catalog before parsing).
#[derive(Parser)]
#[command(name = "kbinds-overlay", version)]
struct Args {
    /// Path to the niri config (default: ~/.config/niri/config.kdl)
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Print the chosen command(s) to stderr instead of executing
    #[arg(long)]
    dry_run: bool,

    /// Overlay mode (only `center` is implemented)
    #[arg(long, default_value = "center", value_name = "MODE", value_parser = ["center"])]
    mode: String,
}

fn main() {
    l10n::init();
    let lang = l10n::current();
    let mut cmd = Args::command();
    l10n::apply_overlay_cli_texts(&mut cmd, lang);
    let matches = cmd.get_matches();
    let args = match Args::from_arg_matches(&matches) {
        Ok(args) => args,
        Err(err) => err.exit(),
    };
    let result = niri_kbinds::default_config_path_if_none(args.config).and_then(|config_path| {
        niri_kbinds::ui::app::run(niri_kbinds::ui::app::OverlayOptions {
            config_path,
            dry_run: args.dry_run,
        })
    });
    if let Err(err) = result {
        eprintln!("{err:#}");
        std::process::exit(2);
    }
}
