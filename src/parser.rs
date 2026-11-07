//! Parsing of the `binds` section of the niri config (SPEC §7).
//!
//! Dialect: KDL v1 — the same document language niri 26.04 reads (knuffel 3.2
//! → kdl 2.x). Spans from the `kdl` crate are byte offsets into the source
//! text (see `line_of_offset`).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use kdl::{KdlDocument, KdlNode, KdlValue};

use crate::groups;
use crate::model::{Action, ActionKind, Bind, TitleSource};
use crate::normalize;

/// One top-level node after `include` expansion, addressed by index so the
/// document store can keep growing while includes are being resolved.
#[derive(Debug, Clone, Copy)]
struct TopNodeRef {
    doc_index: usize,
    node_index: usize,
}

struct StoredDoc {
    file: PathBuf,
    text: String,
    doc: KdlDocument,
}

/// Parse the niri config into the full bind model (SPEC §6, §7).
/// Titles and groups follow the process language (`l10n`).
pub fn parse_config(path: &Path) -> Result<Vec<Bind>> {
    parse_config_with_lang(path, crate::l10n::current())
}

/// Same as [`parse_config`] with an explicit language (deterministic tests).
pub fn parse_config_with_lang(path: &Path, lang: crate::l10n::Lang) -> Result<Vec<Bind>> {
    let start = path
        .canonicalize()
        .with_context(|| format!("resolving config path {}", path.display()))?;
    let mut store: Vec<StoredDoc> = Vec::new();
    let mut stack: Vec<PathBuf> = Vec::new();
    // `start` is canonical here; `collect_top_nodes` canonicalizes only the
    // include targets it reaches recursively.
    let top = collect_top_nodes(&start, &mut stack, &mut store)?;

    let mut binds = Vec::new();
    for top_node in &top {
        let stored = &store[top_node.doc_index];
        let node = &stored.doc.nodes()[top_node.node_index];
        if node.name().value() != "binds" {
            continue; // layout, input, window-rules, …
        }
        let Some(children) = node.children() else {
            continue; // `binds { }` without binds
        };
        for child in children.nodes() {
            let mut bind = parse_bind(child, &stored.file, &stored.text);
            groups::finalize(&mut bind, lang);
            binds.push(bind);
        }
    }
    assign_ids(&mut binds);
    Ok(binds)
}

/// Collect the top-level nodes reachable from `path` (already canonical —
/// the config entry from `parse_config`, or a canonicalized include target),
/// splicing `include` targets recursively. Cycle detection works on the
/// canonical paths on `stack`.
fn collect_top_nodes(
    path: &Path,
    stack: &mut Vec<PathBuf>,
    store: &mut Vec<StoredDoc>,
) -> Result<Vec<TopNodeRef>> {
    if stack.iter().any(|p| p.as_path() == path) {
        bail!("cyclic include detected at {}", path.display());
    }
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let doc =
        KdlDocument::parse_v1(&text).with_context(|| format!("parsing {}", path.display()))?;
    let doc_index = store.len();
    store.push(StoredDoc {
        file: path.to_path_buf(),
        text,
        doc,
    });
    stack.push(path.to_path_buf());

    let mut out = Vec::new();
    let node_count = store[doc_index].doc.nodes().len();
    for node_index in 0..node_count {
        let node = &store[doc_index].doc.nodes()[node_index];
        if node.name().value() == "include" {
            let target = include_target(node, doc_index, store)?;
            let canonical = target
                .canonicalize()
                .with_context(|| format!("resolving include path {}", target.display()))?;
            out.extend(collect_top_nodes(&canonical, stack, store)?);
        } else {
            out.push(TopNodeRef {
                doc_index,
                node_index,
            });
        }
    }
    stack.pop();
    Ok(out)
}

/// Resolve an `include "path"` node relative to the including file's directory.
fn include_target(node: &KdlNode, doc_index: usize, store: &[StoredDoc]) -> Result<PathBuf> {
    let stored = &store[doc_index];
    let line = line_of_offset(&stored.text, node.span().offset());
    let arg = node
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .ok_or_else(|| {
            anyhow!(
                "include without a path argument at {}:{line}",
                stored.file.display()
            )
        })?;
    let relative = match arg.value() {
        KdlValue::String(s) => s.clone(),
        other => bail!(
            "include path must be a string, got {other:?} at {}:{line}",
            stored.file.display()
        ),
    };
    let dir = stored
        .file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(dir.join(relative))
}

/// Parse one bind node. Infallible by design (SPEC §7.1 tolerance): every
/// unknown construct is kept in `raw` instead of failing.
fn parse_bind(node: &KdlNode, file: &Path, source: &str) -> Bind {
    let keys_raw = node.name().value().trim().to_string();
    let line = line_of_offset(source, node.span().offset());

    let mut repeat = true;
    let mut overlay_title: Option<String> = None;
    let mut raw: Vec<String> = Vec::new();

    for entry in node.entries() {
        match entry.name() {
            Some(name) => match name.value() {
                "repeat" => match entry.value() {
                    KdlValue::Bool(b) => repeat = *b,
                    other => raw.push(format!("repeat={}", value_repr(other))),
                },
                "hotkey-overlay-title" => match entry.value() {
                    KdlValue::String(s) => overlay_title = Some(s.clone()),
                    other => {
                        raw.push(format!("hotkey-overlay-title={}", value_repr(other)));
                    }
                },
                other_key => raw.push(format!("{other_key}={}", value_repr(entry.value()))),
            },
            // Bare argument on a bind node is not part of any known syntax.
            None => raw.push(value_repr(entry.value())),
        }
    }

    let mut actions = Vec::new();
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            if name == "condition" {
                raw.push("condition".to_string()); // tolerated, not an action (SPEC §7.1)
                continue;
            }
            let args: Vec<String> = child
                .entries()
                .iter()
                .map(|e| value_repr(e.value()))
                .collect();
            match name {
                "spawn" => actions.push(Action {
                    kind: ActionKind::Spawn,
                    name: None,
                    cmd: Some(args),
                }),
                "spawn-sh" => actions.push(Action {
                    kind: ActionKind::SpawnSh,
                    name: None,
                    cmd: Some(args),
                }),
                other => actions.push(Action {
                    kind: ActionKind::Builtin,
                    name: Some(other.to_string()),
                    cmd: if args.is_empty() { None } else { Some(args) },
                }),
            }
        }
    }

    let combo = normalize::canonical_combo(&keys_raw);
    let keys_display = normalize::keys_display(&keys_raw);
    let overlay_title_present = overlay_title.is_some();

    Bind {
        id: String::new(),    // assigned once all binds are collected
        title: String::new(), // filled by groups::finalize
        // Placeholder, always overwritten by groups::finalize before any read.
        title_source: TitleSource::RawAction,
        actions,
        keys_raw,
        keys_display,
        combo,
        group: String::new(),
        repeat,
        overlay_title_present,
        overlay_title,
        file: file.display().to_string(),
        line,
        search: String::new(),
        raw: raw.join("; "),
    }
}

/// Stable ids: kebab-case of the combo, duplicates get `-2`, `-3`, …
/// in config order (design D4).
fn assign_ids(binds: &mut [Bind]) {
    let mut seen: HashMap<String, usize> = HashMap::new();
    for bind in binds.iter_mut() {
        let base = bind.combo.replace('+', "-");
        let count = seen.entry(base.clone()).or_insert(0);
        *count += 1;
        bind.id = if *count == 1 {
            base
        } else {
            format!("{base}-{count}")
        };
    }
}

/// kdl spans are byte offsets into the source text (v1 parser computes them
/// via pointer arithmetic), so line counting walks bytes.
fn line_of_offset(source: &str, offset: usize) -> usize {
    let offset = offset.min(source.len());
    source.as_bytes()[..offset]
        .iter()
        .filter(|b| **b == b'\n')
        .count()
        + 1
}

fn value_repr(value: &KdlValue) -> String {
    match value {
        KdlValue::String(s) => s.clone(),
        KdlValue::Integer(i) => i.to_string(),
        KdlValue::Float(f) => f.to_string(),
        KdlValue::Bool(b) => b.to_string(),
        KdlValue::Null => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(source: &str) -> Result<Vec<Bind>> {
        let doc = KdlDocument::parse_v1(source).map_err(|e| anyhow!("parse: {e}"))?;
        let stored = StoredDoc {
            file: PathBuf::from("/tmp/config.kdl"),
            text: source.to_string(),
            doc,
        };
        let mut binds = Vec::new();
        for node in stored.doc.nodes() {
            if node.name().value() != "binds" {
                continue;
            }
            let Some(children) = node.children() else {
                continue;
            };
            for child in children.nodes() {
                let mut bind = parse_bind(child, &stored.file, &stored.text);
                groups::finalize(&mut bind, crate::l10n::Lang::Ru);
                binds.push(bind);
            }
        }
        Ok(binds)
    }

    #[test]
    fn block_form_with_attributes_and_actions() {
        let binds = parse_str(
            r#"binds {
    Mod+Q repeat=false { close-window; }
    Mod+Space repeat=false hotkey-overlay-title="Switch layout" { switch-layout "next"; }
}"#,
        )
        .expect("parses");
        assert_eq!(binds.len(), 2);

        assert_eq!(binds[0].keys_raw, "Mod+Q");
        assert!(!binds[0].repeat);
        assert!(!binds[0].overlay_title_present);
        assert_eq!(binds[0].actions.len(), 1);
        assert_eq!(binds[0].actions[0].kind, ActionKind::Builtin);
        assert_eq!(binds[0].actions[0].name.as_deref(), Some("close-window"));
        assert_eq!(binds[0].title, "Закрыть окно");
        assert_eq!(binds[0].title_source, TitleSource::ActionName);
        assert_eq!(binds[0].group, "Окна");

        assert!(binds[1].overlay_title_present);
        assert_eq!(binds[1].title, "Switch layout");
        assert_eq!(binds[1].title_source, TitleSource::OverlayTitle);
        assert_eq!(binds[1].actions[0].name.as_deref(), Some("switch-layout"));
        assert_eq!(binds[1].actions[0].cmd, Some(vec!["next".to_string()]));
    }

    #[test]
    fn spawn_with_args_and_spawn_sh() {
        let binds = parse_str(
            r#"binds {
    Mod+T { spawn "kitty"; }
    Mod+N { spawn "noctalia" "msg" "panel-toggle" "launcher"; }
    Mod+K { spawn-sh "kitty --class kitty-float"; }
}"#,
        )
        .expect("parses");
        assert_eq!(binds[0].actions[0].kind, ActionKind::Spawn);
        assert_eq!(binds[0].actions[0].cmd, Some(vec!["kitty".to_string()]));
        assert_eq!(binds[1].actions[0].kind, ActionKind::Spawn);
        assert_eq!(
            binds[1].actions[0].cmd,
            Some(vec![
                "noctalia".to_string(),
                "msg".to_string(),
                "panel-toggle".to_string(),
                "launcher".to_string(),
            ])
        );
        assert_eq!(binds[2].actions[0].kind, ActionKind::SpawnSh);
        assert_eq!(
            binds[2].actions[0].cmd,
            Some(vec!["kitty --class kitty-float".to_string()])
        );
        assert_eq!(binds[2].title, "kitty --class kitty-float");
        assert_eq!(binds[2].title_source, TitleSource::Command);
    }

    #[test]
    fn multiple_actions_kept_in_order() {
        let binds = parse_str(
            r#"binds {
    Mod+X { spawn "foo"; focus-column-left; spawn "bar"; }
}"#,
        )
        .expect("parses");
        let kinds: Vec<&str> = binds[0].actions.iter().map(|a| a.kind.as_str()).collect();
        assert_eq!(kinds, ["spawn", "builtin", "spawn"]);
    }

    #[test]
    fn comments_are_ignored() {
        let binds = parse_str(
            r#"// line comment
binds {
    /* block
       comment */
    Mod+W { focus-window-up; } // trailing
}"#,
        )
        .expect("parses");
        assert_eq!(binds.len(), 1);
        assert_eq!(binds[0].keys_raw, "Mod+W");
    }

    #[test]
    fn unknown_properties_go_to_raw_without_failure() {
        let binds = parse_str(
            r#"binds {
    Mod+F cooldown-ms=100 allow-when-locked=true { focus-column-left; }
    XF86AudioMute { spawn "pamixer" "-t"; }
}"#,
        )
        .expect("parses");
        assert!(binds[0].raw.contains("cooldown-ms=100"));
        assert!(binds[0].raw.contains("allow-when-locked=true"));
        assert_eq!(binds[0].actions.len(), 1);
        // XF86 keys are tolerated as plain key names (SPEC §7.1).
        assert_eq!(binds[1].keys_raw, "XF86AudioMute");
        assert_eq!(
            binds[1].actions[0].cmd,
            Some(vec!["pamixer".to_string(), "-t".to_string()])
        );
    }

    #[test]
    fn condition_child_is_not_an_action() {
        let binds = parse_str(
            r#"binds {
    Mod+C {
        condition { match app-id="^org.kde$"; }
        close-window;
    }
}"#,
        )
        .expect("parses");
        assert_eq!(binds[0].actions.len(), 1);
        assert_eq!(binds[0].actions[0].name.as_deref(), Some("close-window"));
        assert!(binds[0].raw.contains("condition"));
    }

    #[test]
    fn file_and_line_are_recorded() {
        let binds = parse_str(
            "binds {\n    Mod+T { spawn \"kitty\"; }\n    Mod+B { spawn \"firefox\"; }\n}\n",
        )
        .expect("parses");
        assert_eq!(binds[0].line, 2);
        assert_eq!(binds[1].line, 3);
        assert_eq!(binds[0].file, "/tmp/config.kdl");
    }

    /// Line numbers must survive multi-byte characters above the bind
    /// (kdl v1 spans are byte offsets — regression test for char/byte mixup).
    #[test]
    fn line_numbers_with_multibyte_comments() {
        let source = "// комментарий с кириллицей и достаточной длиной, чтобы байтовый и символьный индексы разошлись заметно\nbinds {\n    Mod+T { spawn \"kitty\"; }\n}\n";
        let binds = parse_str(source).expect("parses");
        assert_eq!(binds[0].line, 3);
    }

    #[test]
    fn duplicate_combo_gets_numbered_ids() {
        let mut binds = vec![
            Bind {
                line: 1,
                ..crate::testutil::sample_bind()
            },
            Bind {
                line: 2,
                ..crate::testutil::sample_bind()
            },
        ];
        for bind in &mut binds {
            bind.keys_raw = "Mod+D".into();
            bind.keys_display = vec!["Mod".into(), "D".into()];
            bind.combo = "mod+d".into();
        }
        assign_ids(&mut binds);
        assert_eq!(binds[0].id, "mod-d");
        assert_eq!(binds[1].id, "mod-d-2");
    }

    #[test]
    fn include_is_spliced_and_resolved_relative() {
        let dir = std::env::temp_dir().join(format!("kbinds-test-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(
            dir.join("inc.kdl"),
            "binds {\n    Mod+I { focus-column-right; }\n}\n",
        )
        .expect("write");
        let main = dir.join("main.kdl");
        fs::write(&main, "layout {\n    something\n}\ninclude \"./inc.kdl\"\n").expect("write");
        let binds = parse_config(&main).expect("parses");
        fs::remove_dir_all(&dir).ok();
        assert_eq!(binds.len(), 1);
        assert_eq!(binds[0].keys_raw, "Mod+I");
        assert!(binds[0].file.ends_with("inc.kdl"));
    }

    #[test]
    fn cyclic_include_is_an_error() {
        let dir = std::env::temp_dir().join(format!("kbinds-cycle-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join("a.kdl"), "include \"./b.kdl\"\n").expect("write");
        fs::write(dir.join("b.kdl"), "include \"./a.kdl\"\n").expect("write");
        let err = parse_config(&dir.join("a.kdl")).expect_err("must fail");
        assert!(err.to_string().contains("cyclic"));
        fs::remove_dir_all(&dir).ok();
    }
}
