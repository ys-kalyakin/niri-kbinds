//! Shared test fixtures (compiled only under `cfg(test)`): one canonical
//! fully-populated [`Bind`] and small action builders, so the 14-field
//! literals live in exactly one place. Tests override fields with
//! structural-update syntax: `Bind { line: 2, ..sample_bind() }`.

use crate::model::{Action, ActionKind, Bind, TitleSource};

/// The canonical sample (promoted from the `model.rs` fixture); its exact
/// values are pinned by the JSON golden test there.
pub(crate) fn sample_bind() -> Bind {
    Bind {
        id: "mod-shift-w".into(),
        title: "Случайные обои".into(),
        title_source: TitleSource::OverlayTitle,
        actions: vec![spawn(&["noctalia", "msg", "wallpaper-random"])],
        keys_raw: "Mod+Shift+W".into(),
        keys_display: vec!["Mod".into(), "Shift".into(), "W".into()],
        combo: "mod+shift+w".into(),
        group: "Система".into(),
        repeat: true,
        overlay_title_present: true,
        overlay_title: Some("Случайные обои".into()),
        file: "/home/helloween/.config/niri/config.kdl".into(),
        line: 214,
        search:
            "случайные обои mod+shift+w mod shift w spawn noctalia msg wallpaper-random система"
                .into(),
        raw: String::new(),
    }
}

/// A `builtin` action, no arguments.
pub(crate) fn builtin(name: &str) -> Action {
    Action {
        kind: ActionKind::Builtin,
        name: Some(name.into()),
        cmd: None,
    }
}

/// A `builtin` action with trailing arguments (`None` when empty).
pub(crate) fn builtin_with_args(name: &str, args: &[&str]) -> Action {
    Action {
        kind: ActionKind::Builtin,
        name: Some(name.into()),
        cmd: if args.is_empty() {
            None
        } else {
            Some(args.iter().map(|s| s.to_string()).collect())
        },
    }
}

/// A `spawn` action with the given argv.
pub(crate) fn spawn(argv: &[&str]) -> Action {
    Action {
        kind: ActionKind::Spawn,
        name: None,
        cmd: Some(argv.iter().map(|s| s.to_string()).collect()),
    }
}

/// A `spawn-sh` action with one shell line.
pub(crate) fn spawn_sh(line: &str) -> Action {
    Action {
        kind: ActionKind::SpawnSh,
        name: None,
        cmd: Some(vec![line.into()]),
    }
}
