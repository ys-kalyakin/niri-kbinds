//! Data model of a parsed bind (serde-serializable, see SPEC §6).

use serde::Serialize;

/// Kind of one action node inside a bind body (`SPEC §6` JSON value of
/// `kind`; serialized kebab-case, so the JSON is unchanged by the enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionKind {
    /// A niri builtin action with a name (`close-window;`).
    Builtin,
    /// `spawn "argv"…` — direct program launch.
    Spawn,
    /// `spawn-sh "line"` — one shell line.
    SpawnSh,
}

impl ActionKind {
    /// String label used in `describe()`, search text and audit details.
    pub fn as_str(self) -> &'static str {
        match self {
            ActionKind::Builtin => "builtin",
            ActionKind::Spawn => "spawn",
            ActionKind::SpawnSh => "spawn-sh",
        }
    }
}

/// Where a bind's title came from (`SPEC §6` JSON value of `title_source`;
/// serialized kebab-case, so the JSON is unchanged by the enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TitleSource {
    /// The `hotkey-overlay-title` attribute.
    OverlayTitle,
    /// The dictionary title of a known builtin action.
    ActionName,
    /// The raw unknown action name.
    RawAction,
    /// The spawn command line.
    Command,
}

impl TitleSource {
    /// String label used in the JSON field value.
    pub fn as_str(self) -> &'static str {
        match self {
            TitleSource::OverlayTitle => "overlay-title",
            TitleSource::ActionName => "action-name",
            TitleSource::RawAction => "raw-action",
            TitleSource::Command => "command",
        }
    }
}

/// One action node inside a bind body, in config order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Action {
    /// Kind of the action node.
    pub kind: ActionKind,
    /// Action name for [`ActionKind::Builtin`] (e.g. `close-window`), `None`
    /// otherwise.
    pub name: Option<String>,
    /// Arguments: full argv for `spawn`, the shell line for `spawn-sh`,
    /// trailing arguments for `builtin`; `None` when there are none.
    pub cmd: Option<Vec<String>>,
}

impl Action {
    /// Human-readable description used in `search` and audit messages.
    pub fn describe(&self) -> String {
        let mut text = self.kind.as_str().to_string();
        if let Some(name) = &self.name {
            text.push(' ');
            text.push_str(name);
        }
        if let Some(cmd) = &self.cmd {
            text.push(' ');
            text.push_str(&cmd.join(" "));
        }
        text
    }
}

/// A single key binding extracted from the niri config.
#[derive(Debug, Clone, Serialize)]
pub struct Bind {
    /// Stable kebab-case identifier derived from `combo` (duplicates get `-2`, `-3`, …).
    pub id: String,
    /// Human-readable title (priority: overlay title → dictionary → command).
    pub title: String,
    /// Where the title came from.
    pub title_source: TitleSource,
    /// All action nodes of the bind body, in config order.
    pub actions: Vec<Action>,
    /// Key combo as written in the config, e.g. `Mod+Shift+W`.
    pub keys_raw: String,
    /// Display form of every key token (arrows, Enter, Cyrillic letters, …).
    pub keys_display: Vec<String>,
    /// Canonical combo key for audit: modifiers in `mod, ctrl, shift, alt` order, lowercase key.
    pub combo: String,
    /// Deterministic action group, e.g. `Окна`.
    pub group: String,
    /// The `repeat` property (`true` when absent).
    pub repeat: bool,
    /// Whether the `hotkey-overlay-title` attribute is present.
    pub overlay_title_present: bool,
    /// The `hotkey-overlay-title` value itself (used for the title priority);
    /// not part of the JSON schema.
    #[serde(skip_serializing)]
    pub overlay_title: Option<String>,
    /// Absolute path of the file the bind was parsed from.
    pub file: String,
    /// 1-based line of the bind node.
    pub line: usize,
    /// Lowercase concatenation of title, keys, actions and group.
    pub search: String,
    /// Unknown properties/attributes kept verbatim as diagnostics data;
    /// audit rules A1–A3 never read it. Not part of the JSON schema.
    #[serde(skip_serializing)]
    pub raw: String,
}

#[cfg(test)]
mod tests {
    use crate::testutil::sample_bind;

    #[test]
    fn json_matches_schema() {
        let expected: serde_json::Value = serde_json::json!({
            "id": "mod-shift-w",
            "title": "Случайные обои",
            "title_source": "overlay-title",
            "actions": [
                { "kind": "spawn", "name": null, "cmd": ["noctalia", "msg", "wallpaper-random"] }
            ],
            "keys_raw": "Mod+Shift+W",
            "keys_display": ["Mod", "Shift", "W"],
            "combo": "mod+shift+w",
            "group": "Система",
            "repeat": true,
            "overlay_title_present": true,
            "file": "/home/helloween/.config/niri/config.kdl",
            "line": 214,
            "search": "случайные обои mod+shift+w mod shift w spawn noctalia msg wallpaper-random система"
        });
        let actual = serde_json::to_value(sample_bind()).expect("serializable");
        assert_eq!(actual, expected);
    }

    #[test]
    fn raw_field_is_not_serialized() {
        let mut bind = sample_bind();
        bind.raw = "cooldown-ms=100".into();
        let json = serde_json::to_string(&bind).expect("serializable");
        assert!(
            !json.contains("\"raw\""),
            "raw must stay out of the JSON output"
        );
    }

    #[test]
    fn action_description_contains_kind_name_and_command() {
        use crate::testutil::{builtin_with_args, spawn};

        let builtin = builtin_with_args("switch-layout", &["next"]);
        assert_eq!(builtin.describe(), "builtin switch-layout next");

        let spawn = spawn(&["kitty"]);
        assert_eq!(spawn.describe(), "spawn kitty");
    }
}
