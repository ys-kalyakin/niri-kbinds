//! fuzzel dmenu adapter: search binds, execute the selection (SPEC §8, §10;
//! design D6).
//!
//! Protocol: items on stdin (`<keys_display> — <title>`, action names
//! included so fuzzel searches by them), selection on stdout. Empty stdout
//! (Esc or cancel) exits 0 without executing anything. Items are sanitized
//! to one line each; collisions get a ` [<id>]` suffix so the reverse
//! mapping stays exact.

use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::exec;
use crate::l10n::{self, Msg};
use crate::model::Bind;

/// One menu entry: the rendered item and the bind id it maps back to.
pub type MenuItem = (String, String);

fn sanitize(line: &str) -> String {
    line.replace(['\n', '\r'], " ")
}

/// Render the item for one bind: `<keys_display> — <title>` with builtin
/// action names appended when the title doesn't already contain them.
fn base_item(bind: &Bind) -> String {
    let mut item = format!("{} — {}", bind.keys_display.join("+"), bind.title);
    for action in &bind.actions {
        if action.kind == "builtin"
            && let Some(name) = &action.name
            && !item.to_lowercase().contains(&name.to_lowercase())
        {
            item.push_str(&format!(" ({name})"));
        }
    }
    item
}

/// Build unique menu items (design D6): sanitized to one line, collisions
/// disambiguated with ` [<id>]`.
pub fn build_menu(binds: &[Bind]) -> Vec<MenuItem> {
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut menu = Vec::with_capacity(binds.len());
    for bind in binds {
        let mut item = sanitize(&base_item(bind));
        let count = seen.entry(item.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            item.push_str(&format!(" [{}]", bind.id));
        }
        menu.push((item, bind.id.clone()));
    }
    menu
}

/// Map the launcher's selection back to the bind (exact item match).
pub fn select_bind<'a>(menu: &[MenuItem], binds: &'a [Bind], choice: &str) -> Option<&'a Bind> {
    let (_, id) = menu.iter().find(|(item, _)| item == choice)?;
    binds.iter().find(|b| b.id == *id)
}

/// Run the launcher, return the bind the user picked (`None` = cancelled).
pub fn pick(launcher: &str, binds: &[Bind]) -> Result<Option<Bind>> {
    let lang = l10n::current();
    let menu = build_menu(binds);
    if menu.is_empty() {
        bail!("{}", l10n::tr(lang, Msg::EmptyConfig));
    }
    let mut child = Command::new(launcher)
        .args(["--dmenu", "-p", "binds> "])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| l10n::launcher_launch_failed(lang, launcher))?;
    if let Some(mut stdin) = child.stdin.take() {
        // The launcher may exit before reading all items (instant Esc,
        // crash): a broken pipe while feeding means the same as an empty
        // selection, so stop feeding and let wait_with_output decide (D6).
        for (item, _) in &menu {
            if writeln!(stdin, "{item}").is_err() {
                break;
            }
        }
    }
    let output = child.wait_with_output()?;
    let choice = String::from_utf8_lossy(&output.stdout);
    let choice = choice.trim_end_matches(['\n', '\r']).to_string();
    if choice.is_empty() {
        // Esc / cancel — fuzzel may exit non-zero; nothing to execute (D6).
        return Ok(None);
    }
    Ok(Some(
        select_bind(&menu, binds, &choice)
            .cloned()
            .context(l10n::choice_not_matched(lang, &choice))?,
    ))
}

/// `kbinds dmenu`: pick a bind via the launcher and execute it.
pub fn run(launcher: &str, binds: &[Bind]) -> Result<()> {
    let Some(bind) = pick(launcher, binds)? else {
        return Ok(());
    };
    exec::exec_bind(&bind, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Action;

    fn bind(id: &str, keys: &str, title: &str, action_name: Option<&str>) -> Bind {
        Bind {
            id: id.into(),
            title: title.into(),
            title_source: "overlay-title".into(),
            actions: vec![Action {
                kind: "builtin".into(),
                name: action_name.map(Into::into),
                cmd: None,
            }],
            keys_raw: keys.into(),
            keys_display: keys.split('+').map(Into::into).collect(),
            combo: "mod+x".into(),
            group: "Окна".into(),
            repeat: true,
            overlay_title_present: true,
            overlay_title: Some(title.into()),
            file: "/tmp/c.kdl".into(),
            line: 1,
            search: String::new(),
            raw: String::new(),
        }
    }

    #[test]
    fn items_have_keys_title_and_action_name() {
        let binds = vec![bind("mod-q", "Mod+Q", "Закрыть окно", Some("close-window"))];
        let menu = build_menu(&binds);
        assert_eq!(menu[0].0, "Mod+Q — Закрыть окно (close-window)");
    }

    #[test]
    fn action_name_not_duplicated_when_title_contains_it() {
        let binds = vec![bind("mod-t", "Mod+T", "kitty", None)];
        let menu = build_menu(&binds);
        assert_eq!(menu[0].0, "Mod+T — kitty");
    }

    #[test]
    fn newlines_are_sanitized() {
        let mut b = bind("mod-n", "Mod+N", "строка\nвторая", None);
        b.title_source = "command".into();
        let menu = build_menu(&[b]);
        assert_eq!(menu[0].0, "Mod+N — строка вторая");
        assert!(!menu[0].0.contains('\n'));
    }

    #[test]
    fn collisions_get_id_suffix_and_map_back() {
        // Полностью одинаковые пункты (одинаковые клавиши и overlay-title у
        // разных биндов) требуют различителя для обратного маппинга.
        let binds = vec![
            bind("mod-d", "Mod+D", "Launcher", Some("same-action")),
            bind("mod-d-2", "Mod+D", "Launcher", Some("same-action")),
        ];
        let menu = build_menu(&binds);
        assert_eq!(menu[0].0, "Mod+D — Launcher (same-action)");
        assert_eq!(menu[1].0, "Mod+D — Launcher (same-action) [mod-d-2]");
        assert_eq!(
            select_bind(&menu, &binds, &menu[1].0).unwrap().id,
            "mod-d-2"
        );
        assert_eq!(select_bind(&menu, &binds, &menu[0].0).unwrap().id, "mod-d");
        assert!(select_bind(&menu, &binds, "нет такого").is_none());
    }

    #[test]
    fn pick_returns_none_on_empty_stdout() {
        let binds = vec![bind("mod-q", "Mod+Q", "Закрыть окно", Some("close-window"))];
        // `true` ignores stdin, outputs nothing — models Esc/cancel.
        let picked = pick("true", &binds).expect("launcher runs");
        assert!(picked.is_none());
    }
}
