//! Iced application of the overlay (SPEC-V2 §5): boot, state, keyboard and
//! mouse handling, and the layer-shell surface settings.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use iced::theme::Style;
use iced::widget::Id;
use iced::widget::operation::RelativeOffset;
use iced::widget::{column, container, mouse_area, operation, row, scrollable, text, text_input};
use iced::{
    Alignment, Background, Element, Event, Font, Length, Padding, Point, Size, Subscription, Task,
    keyboard::{self, Key, key::Named},
    mouse, window,
};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::{application, to_layer_message};
use iced_runtime::core::widget::operation::focusable;
use iced_runtime::task::widget as task_widget;

use super::items::{self, Row};
use super::theme::Palette;
use crate::l10n::{self, Msg};
use crate::model::Bind;

const SEARCH_INPUT: &str = "kbinds-overlay-search";
const LIST_SCROLL: &str = "kbinds-overlay-list";
/// Scrollable of the right cheat-sheet panel.
const CHEATSHEET_SCROLL: &str = "kbinds-overlay-cheatsheet";

/// Everything the overlay needs before the UI boots.
#[derive(Debug, Clone)]
pub struct OverlayOptions {
    pub config_path: PathBuf,
    pub dry_run: bool,
}

/// Parse the binds and boot the layer-shell overlay on the focused output.
/// All errors are human-readable; callers turn them into exit code 2.
pub fn run(opts: OverlayOptions) -> Result<()> {
    let lang = l10n::current();
    if std::env::var_os("WAYLAND_DISPLAY").is_none_or(|v| v.is_empty()) {
        bail!("{}", l10n::tr(lang, Msg::WaylandSessionNotFound));
    }
    let binds = crate::parser::parse_config(&opts.config_path)
        .with_context(|| l10n::config_parse(lang, &opts.config_path))?;
    let (output, logical) = focused_output().context(l10n::tr(lang, Msg::FocusedOutputFail))?;
    let palette = Palette::load();

    application(
        boot(binds, logical, palette, opts.dry_run),
        namespace,
        update,
        view,
    )
    .style(|state: &Overlay, _theme| Style {
        // Dim at 55% over the desktop; colors from the noctalia palette
        // cache (design D9).
        background_color: state.palette.dim(),
        text_color: state.palette.text_primary(),
    })
    .subscription(subscription)
    .settings(Settings {
        layer_settings: LayerShellSettings {
            // Fullscreen on the focused output (SPEC-V2 §5.2).
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: -1,
            size: None,
            margin: (0, 0, 0, 0),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::TargetScreen(output),
            events_transparent: false,
        },
        ..Default::default()
    })
    .run()
    .map_err(|err| anyhow::anyhow!("kbinds-overlay: {err}"))
}

fn boot(
    binds: Vec<Bind>,
    logical: Size,
    palette: Palette,
    dry_run: bool,
) -> impl Fn() -> (Overlay, Task<Message>) {
    move || {
        let rows = items::filter_rows(&binds, "");
        let selected = items::first_bind_row(&rows).unwrap_or(0);
        let state = Overlay {
            // `boot` is an `Fn`: clone the binds for every call.
            binds: binds.clone(),
            dry_run,
            query: String::new(),
            rows,
            selected,
            cheatsheet: items::cheatsheet_binds(&binds),
            cursor: Point::ORIGIN,
            // Layer surfaces never receive a `Resized` event through the
            // iced subscription, so the logical output size from
            // `niri msg --json focused-output` is the window size.
            window: logical,
            palette,
        };
        (state, task_widget(focusable::focus(Id::from(SEARCH_INPUT))))
    }
}

struct Overlay {
    binds: Vec<Bind>,
    dry_run: bool,
    /// Search field content (plain text; Cyrillic included as typed).
    query: String,
    /// Visible rows for the current query.
    rows: Vec<Row>,
    /// Position of the selection within `rows`.
    selected: usize,
    /// Cheat-sheet bind indexes (every `hotkey-overlay-title` bind, config
    /// order), computed once at boot: the right panel is read-only and is
    /// never filtered by the search query.
    cheatsheet: Vec<usize>,
    /// Last known cursor position (hit-testing clicks outside the card).
    cursor: Point,
    /// Logical window size: the focused output's logical size from
    /// `niri msg` at boot (layer surfaces get no `Resized` subscription
    /// event); later `Resized` events, if any, overwrite it.
    window: Size,
    /// Theme resolved from the noctalia palette cache (design D9).
    palette: Palette,
}

// The macro must come before the derives: it appends its own variants to
// the enum, and derives expand after macros in attribute order.
#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    /// Window and mouse events (resize, cursor moves, clicks).
    Iced(Event),
    /// Search field content changed.
    Query(String),
    /// Enter in the search field: execute the selected bind.
    Submit,
    /// ↑↓ move the selection cyclically.
    Move(i32),
    /// Click on a visible bind row at `position`.
    ClickRow(usize),
    /// Close the overlay without executing anything (Esc).
    Close,
}

fn namespace() -> String {
    String::from("kbinds-overlay")
}

fn update(state: &mut Overlay, message: Message) -> Task<Message> {
    match message {
        Message::Iced(event) => on_iced_event(state, event),
        Message::Query(query) => {
            state.query = query;
            state.rows = items::filter_rows(&state.binds, &state.query);
            // SPEC-V2 §5.6: the selection resets on a filter change.
            state.selected = items::first_bind_row(&state.rows).unwrap_or(0);
            snap_list(Some(0.0))
        }
        Message::Submit => execute(state, None),
        Message::Move(delta) => {
            state.selected = items::move_selection(&state.rows, state.selected, delta);
            scroll_to_selected(state)
        }
        Message::ClickRow(position) => execute(state, Some(position)),
        Message::Close => iced::exit(),
        _ => Task::none(), // generated layer-shell variants are unused here
    }
}

/// Key handling independent of widget capture: the focused search field
/// captures Escape (it unfocuses) and the arrow keys, so an ignored-only
/// subscription would miss them. Escape must close the overlay on the
/// FIRST press (SPEC-V2 §5.6), and ↑↓ must move the selection while
/// typing — hence the raw listener.
fn on_key(key: Key) -> Option<Message> {
    match key {
        Key::Named(Named::Escape) => Some(Message::Close),
        Key::Named(Named::ArrowUp) => Some(Message::Move(-1)),
        Key::Named(Named::ArrowDown) => Some(Message::Move(1)),
        _ => None,
    }
}

fn on_iced_event(state: &mut Overlay, event: Event) -> Task<Message> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            state.cursor = position;
            Task::none()
        }
        // Click outside the card closes the overlay (SPEC-V2 §5.7). The
        // window size is unknown until the first resize event, so clicks
        // before that are ignored.
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            if state.window.width > 0.0
                && items::is_outside_card(
                    (state.cursor.x, state.cursor.y),
                    (state.window.width, state.window.height),
                    !state.cheatsheet.is_empty(),
                )
            {
                iced::exit()
            } else {
                Task::none()
            }
        }
        Event::Window(window::Event::Resized(size)) => {
            state.window = size;
            Task::none()
        }
        _ => Task::none(),
    }
}

/// Execute the bind at `position` (or the selected one) and close:
/// `--dry-run` prints the command of every action to stderr, the
/// real run goes through MVP1 `exec` (SPEC-V2 §5.8).
fn execute(state: &Overlay, position: Option<usize>) -> Task<Message> {
    let Some(Row::Bind { index }) = state.rows.get(position.unwrap_or(state.selected)) else {
        return Task::none();
    };
    let bind = &state.binds[*index];
    if state.dry_run {
        for command in bind_commands(bind) {
            eprintln!("{command}");
        }
    } else if let Err(err) = crate::exec::exec_bind(bind, false) {
        // The overlay still closes; the failure is visible on stderr.
        eprintln!("kbinds-overlay: {err:#}");
    }
    iced::exit()
}

/// Shell-quoted command lines of a bind, one per action — the exact
/// strings `--dry-run` prints to stderr.
fn bind_commands(bind: &Bind) -> Vec<String> {
    bind.actions
        .iter()
        .map(|action| crate::exec::display_command(&crate::exec::action_argv(action)))
        .collect()
}

/// Scroll the list back to the top (used after a filter change).
fn snap_list(ratio: Option<f32>) -> Task<Message> {
    operation::snap_to(
        Id::from(LIST_SCROLL),
        RelativeOffset {
            x: Some(0.0),
            y: ratio,
        },
    )
}

/// Keep the selected row visible: snap the list to the row's relative
/// position among the visible bind rows.
fn scroll_to_selected(state: &Overlay) -> Task<Message> {
    let bind_positions: Vec<usize> = state
        .rows
        .iter()
        .enumerate()
        .filter_map(|(position, row)| row.bind_index().map(|_| position))
        .collect();
    let Some(&selected) = bind_positions.iter().find(|&&p| p == state.selected) else {
        return Task::none();
    };
    let ordinal = bind_positions
        .iter()
        .position(|&p| p == selected)
        .unwrap_or_default();
    let total = bind_positions.len();
    if total <= 1 {
        return Task::none();
    }
    let ratio = (ordinal as f32 / (total - 1) as f32).clamp(0.0, 1.0);
    snap_list(Some(ratio))
}

fn subscription(_state: &Overlay) -> Subscription<Message> {
    Subscription::batch([
        // Raw listener for keys (see `on_key`): delivered regardless of
        // the widget capture status, so Escape closes on the first press
        // even while the search field is focused.
        iced::event::listen_raw(|event, _status, _window| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => on_key(key),
            _ => None,
        }),
        // Ignored-only listener is enough for mouse and resize: a click
        // outside the card is ignored by every widget by definition.
        iced::event::listen().map(Message::Iced),
    ])
}

fn view(state: &Overlay) -> Element<'_, Message> {
    // The card rectangle comes from `items::card_rect` — the same function
    // the outside-click hit test uses (design D2).
    let has_cheatsheet = !state.cheatsheet.is_empty();
    let (_, _, card_width, card_height) =
        items::card_rect((state.window.width, state.window.height), has_cheatsheet);
    let (card_width, card_height) = (card_width.max(1.0), card_height.max(1.0));
    // Palette is `Copy`: capture it by value in the style closures.
    let palette = state.palette;

    let search = text_input(
        l10n::tr(l10n::current(), Msg::SearchPlaceholder),
        &state.query,
    )
    .on_input(Message::Query)
    .on_submit(Message::Submit)
    .size(16)
    .padding(Padding {
        top: 8.0,
        bottom: 8.0,
        left: 12.0,
        right: 12.0,
    })
    .id(Id::from(SEARCH_INPUT))
    .style(move |theme, status| palette.field_style(theme, status))
    .width(Length::Fill);

    let rows: Vec<Element<'_, Message>> = state
        .rows
        .iter()
        .enumerate()
        .map(|(position, row)| match row {
            Row::Header(name) => container(
                text(name.clone())
                    .size(12)
                    .style(move |theme| palette.secondary_style(theme)),
            )
            .padding(Padding {
                top: 10.0,
                bottom: 2.0,
                left: 4.0,
                right: 4.0,
            })
            .width(Length::Fill)
            .into(),
            Row::Bind { index } => bind_row(state, position, *index),
        })
        .collect();

    let visible = state
        .rows
        .iter()
        .filter(|row| matches!(row, Row::Bind { .. }))
        .count();
    let counter = text(l10n::counter(l10n::current(), visible, state.binds.len()))
        .size(11)
        .style(move |theme| palette.secondary_style(theme));
    let footer = text(l10n::tr(l10n::current(), Msg::FooterHints))
        .size(11)
        .style(move |theme| palette.secondary_style(theme));

    let list = scrollable(column(rows))
        .id(Id::from(LIST_SCROLL))
        .width(Length::Fill)
        .height(Length::Fill);

    let left = column![search, list, counter, footer]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill);

    // Read-only cheat sheet on the right (binds with a
    // `hotkey-overlay-title`); without any titled binds the card stays
    // single-column as in MVP2.
    let content: Element<'_, Message> = if state.cheatsheet.is_empty() {
        left.into()
    } else {
        let separator = container(iced::widget::Space::new().width(1.0).height(Length::Fill))
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(move |_| iced::widget::container::Style {
                background: Some(Background::Color(palette.card_border())),
                ..Default::default()
            });
        row![left, separator, cheatsheet_panel(state, palette)]
            .spacing(12)
            .into()
    };

    let card = container(content)
        .padding(16)
        .width(Length::Fixed(card_width))
        .height(Length::Fixed(card_height));

    container(card.style(move |_theme| palette.card_style()))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Keycap chips of a bind — the shared look of the list rows and the
/// cheat-sheet panel (mono 11px, keycap style).
fn keycap_chips(bind: &Bind, palette: Palette) -> Vec<Element<'_, Message>> {
    items::keycaps(bind)
        .into_iter()
        .map(|key| {
            container(
                text(key)
                    .size(11)
                    .font(Font::MONOSPACE)
                    .style(move |theme| palette.text_primary_style(theme)),
            )
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 6.0,
                right: 6.0,
            })
            .style(move |_| palette.keycap_style())
            .into()
        })
        .collect()
}

/// One visible bind: `title + keycaps` as in the mockup, accent background
/// when selected, click executes (SPEC-V2 §5.7).
fn bind_row(state: &Overlay, position: usize, index: usize) -> Element<'_, Message> {
    let bind = &state.binds[index];
    let selected = position == state.selected;
    let palette = state.palette;
    let title = text(bind.title.clone())
        .size(14)
        .style(move |theme| palette.text_primary_style(theme));
    let content = row![
        container(title).width(Length::Fill),
        row(keycap_chips(bind, palette)).spacing(4)
    ]
    .align_y(Alignment::Center);
    container(mouse_area(content).on_press(Message::ClickRow(position)))
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 10.0,
            right: 10.0,
        })
        .style(move |_| palette.row_style(selected))
        .width(Length::Fill)
        .into()
}

/// The right read-only panel: every bind with a `hotkey-overlay-title` —
/// the set niri's built-in PNG overlay used to show. Same «title + keycaps»
/// look as the list rows, but no clicks, no selection and no filtering;
/// scrolls independently of the list.
fn cheatsheet_panel(state: &Overlay, palette: Palette) -> Element<'_, Message> {
    let rows: Vec<Element<'_, Message>> = state
        .cheatsheet
        .iter()
        .map(|&index| cheat_row(state, index, palette))
        .collect();
    let header = text(l10n::tr(l10n::current(), Msg::CheatsheetHeader))
        .size(12)
        .style(move |theme| palette.secondary_style(theme));
    let list = scrollable(column(rows))
        .id(Id::from(CHEATSHEET_SCROLL))
        .width(Length::Fill)
        .height(Length::Fill);
    column![header, list]
        .spacing(6)
        .width(Length::Fixed(items::CHEATSHEET_WIDTH))
        .height(Length::Fill)
        .into()
}

/// One cheat-sheet row: «title + keycaps», compact and inert — no
/// `mouse_area`, no highlight; a click lands on the plain container and
/// does nothing (read-only panel).
fn cheat_row(state: &Overlay, index: usize, palette: Palette) -> Element<'_, Message> {
    let bind = &state.binds[index];
    let title = text(bind.title.clone())
        .size(14)
        .style(move |theme| palette.text_primary_style(theme));
    let content = row![
        container(title).width(Length::Fill),
        row(keycap_chips(bind, palette)).spacing(4)
    ]
    .align_y(Alignment::Center);
    container(content)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 10.0,
            right: 10.0,
        })
        .width(Length::Fill)
        .into()
}

/// Name and logical size of the focused output via
/// `niri msg --json focused-output` (the `--json` flag is global to
/// `niri msg` and must precede the subcommand; the
/// `focused-output --json` form does not exist).
fn focused_output() -> Result<(String, Size)> {
    let lang = l10n::current();
    let output = std::process::Command::new("niri")
        .args(["msg", "--json", "focused-output"])
        .output()
        .context(l10n::tr(lang, Msg::NiriMsgLaunchFail))?;
    if !output.status.success() {
        bail!(
            "{}",
            l10n::niri_msg_exit_code(lang, output.status.code().unwrap_or(-1))
        );
    }
    focused_output_from_json(&String::from_utf8_lossy(&output.stdout))
        .context(l10n::tr(lang, Msg::NiriMsgBadFormat))
}

/// Pure part of `focused_output` for unit tests.
fn focused_output_from_json(json: &str) -> Option<(String, Size)> {
    let value = serde_json::from_str::<serde_json::Value>(json).ok()?;
    let name = value.get("name")?.as_str()?.to_string();
    let logical = value.get("logical")?;
    let width = logical.get("width")?.as_f64()? as f32;
    let height = logical.get("height")?.as_f64()? as f32;
    Some((name, Size::new(width, height)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Action;

    const SAMPLE: &str = r#"{"name":"DP-2","make":"Microstep","model":"MAG 271QPX E2",
        "serial":null,"physical_size":[590,330],"modes":[],
        "current_mode":0,"is_custom_mode":false,"vrr_supported":true,
        "vrr_enabled":false,"logical":{"x":0,"y":0,"width":2560,"height":1440,
        "scale":1.0,"transform":"Normal"}}"#;

    #[test]
    fn focused_output_name_and_logical_size_are_extracted() {
        let (name, logical) = focused_output_from_json(SAMPLE).expect("sample must parse");
        assert_eq!(name, "DP-2", "the `name` field must be extracted");
        assert_eq!((logical.width, logical.height), (2560.0, 1440.0));
    }

    #[test]
    fn focused_output_garbage_is_rejected() {
        assert_eq!(focused_output_from_json("not json"), None);
        assert_eq!(focused_output_from_json("{}"), None);
        assert_eq!(focused_output_from_json(r#"{"name":42}"#), None);
        assert_eq!(
            focused_output_from_json(r#"{"name":"DP-2","logical":{"width":2560}}"#),
            None,
            "logical size must be complete"
        );
    }

    #[test]
    fn bind_commands_covers_every_action() {
        let bind = Bind {
            id: "mod-t".into(),
            title: "Терминал".into(),
            title_source: "overlay-title".into(),
            actions: vec![
                Action {
                    kind: "spawn".into(),
                    name: None,
                    cmd: Some(vec!["kitty".into()]),
                },
                Action {
                    kind: "builtin".into(),
                    name: Some("close-window".into()),
                    cmd: None,
                },
            ],
            keys_raw: "Mod+T".into(),
            keys_display: vec!["Mod".into(), "T".into()],
            combo: "mod+t".into(),
            group: "Программы".into(),
            repeat: true,
            overlay_title_present: true,
            overlay_title: None,
            file: "/tmp/c.kdl".into(),
            line: 1,
            search: String::new(),
            raw: String::new(),
        };
        assert_eq!(
            bind_commands(&bind),
            vec![
                "niri msg action spawn -- kitty".to_string(),
                "niri msg action close-window".to_string(),
            ],
            "dry-run must print one command per action"
        );
    }
}
