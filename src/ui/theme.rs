//! Overlay palette and styles (SPEC-V2 §6, extended by D9): colors follow
//! the noctalia palette cache, the fixed §6 table is the fallback.

use std::path::PathBuf;

use iced::widget::{container, text, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// JSON cache written by the noctalia user template
/// (`~/.config/noctalia/templates/kbinds-overlay.json` → `theme.json`,
/// see design D9).
pub fn cache_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("kbinds").join("theme.json"))
}

/// Resolved overlay palette: M3 tokens from noctalia, mapped to the
/// overlay roles. `Copy` so view closures can capture it cheaply.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Dim layer over the desktop (token `scrim`).
    pub scrim: Color,
    /// Card fill (token `surface_container`).
    pub surface: Color,
    /// Card border (token `outline_variant`).
    pub outline_variant: Color,
    /// Keycap fill (token `surface_container_highest`).
    pub surface_highest: Color,
    /// Keycap border (token `outline`).
    pub outline: Color,
    /// Primary text (token `on_surface`).
    pub on_surface: Color,
    /// Secondary text (token `on_surface_variant`).
    pub on_surface_variant: Color,
    /// Accent: selection and focused field (token `primary`).
    pub primary: Color,
}

impl Palette {
    /// The fixed palette of SPEC-V2 §6 — the fallback when the noctalia
    /// cache is missing or unparsable. Note: base colors here are stored
    /// without the §6 alphas — the alphas live in the role methods.
    pub const fn fallback() -> Self {
        Self {
            scrim: rgba(8, 6, 14, 1.0),
            surface: rgba(21, 18, 30, 1.0),
            outline_variant: rgb(255, 255, 255),
            surface_highest: rgba(255, 255, 255, 1.0),
            outline: rgb(255, 255, 255),
            on_surface: rgb(0xec, 0xe9, 0xf4),
            on_surface_variant: rgb(0x9b, 0x93, 0xb3),
            primary: rgb(0xa7, 0x8b, 0xfa),
        }
    }

    /// Load from the noctalia cache; any unreadable or unparsable entry
    /// falls back to the §6 value for that token alone.
    pub fn load() -> Self {
        let Some(path) = cache_path() else {
            return Self::fallback();
        };
        match std::fs::read_to_string(&path) {
            Ok(content) => Self::from_json(&content),
            Err(_) => Self::fallback(),
        }
    }

    /// Parse the cache JSON; unknown or malformed values keep §6 values.
    pub fn from_json(json: &str) -> Self {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
            return Self::fallback();
        };
        let get = |key: &str, fallback: Color| -> Color {
            value
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(parse_hex)
                .unwrap_or(fallback)
        };
        let f = Self::fallback();
        Self {
            scrim: get("scrim", f.scrim),
            surface: get("surface_container", f.surface),
            outline_variant: get("outline_variant", f.outline_variant),
            surface_highest: get("surface_container_highest", f.surface_highest),
            outline: get("outline", f.outline),
            on_surface: get("on_surface", f.on_surface),
            on_surface_variant: get("on_surface_variant", f.on_surface_variant),
            primary: get("primary", f.primary),
        }
    }

    // --- Role colors with the SPEC-V2 §6 alpha levels ---

    /// Scene dim over the desktop: scrim at 55%.
    pub fn dim(&self) -> Color {
        Color {
            a: 0.55,
            ..self.scrim
        }
    }

    /// Card fill: surface at 94%.
    pub fn card_bg(&self) -> Color {
        Color {
            a: 0.94,
            ..self.surface
        }
    }

    /// Card border: outline_variant at 55% (subtle, as in the shell).
    pub fn card_border(&self) -> Color {
        Color {
            a: 0.55,
            ..self.outline_variant
        }
    }

    /// Primary text.
    pub fn text_primary(&self) -> Color {
        self.on_surface
    }

    /// Secondary text — never used on the accent background (contrast
    /// rule of SPEC-V2 §6).
    pub fn text_secondary(&self) -> Color {
        self.on_surface_variant
    }

    /// Accent: row selection, focused field.
    pub fn accent(&self) -> Color {
        self.primary
    }

    /// Selection fill: primary at 16%.
    pub fn accent_bg(&self) -> Color {
        Color {
            a: 0.16,
            ..self.primary
        }
    }

    /// Keycap fill.
    pub fn keycap_bg(&self) -> Color {
        self.surface_highest
    }

    /// Keycap border.
    pub fn keycap_border(&self) -> Color {
        self.outline
    }

    // --- Widget styles (self by value; Palette is Copy) ---

    /// Card container: theme fill, subtle border, radius 20, soft shadow.
    pub fn card_style(&self) -> container::Style {
        container::Style {
            text_color: Some(self.text_primary()),
            background: Some(Background::Color(self.card_bg())),
            border: Border {
                color: self.card_border(),
                width: 1.0,
                radius: CARD_RADIUS.into(),
            },
            shadow: Shadow {
                color: Color {
                    a: 0.45,
                    ..self.scrim
                },
                offset: Vector::new(0.0, 10.0),
                blur_radius: 30.0,
            },
            ..Default::default()
        }
    }

    /// Bind row: accent background when selected, radius 10.
    pub fn row_style(&self, selected: bool) -> container::Style {
        container::Style {
            text_color: Some(self.text_primary()),
            background: Some(Background::Color(if selected {
                self.accent_bg()
            } else {
                Color::TRANSPARENT
            })),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: ROW_RADIUS.into(),
            },
            ..Default::default()
        }
    }

    /// Search field: radius 12; the border turns accent when focused.
    pub fn field_style(&self, _theme: &Theme, status: text_input::Status) -> text_input::Style {
        let border_color = match status {
            text_input::Status::Focused { .. } => self.accent(),
            _ => self.card_border(),
        };
        text_input::Style {
            background: Background::Color(Color {
                a: 0.35,
                ..self.surface_highest
            }),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: FIELD_RADIUS.into(),
            },
            icon: self.text_secondary(),
            placeholder: self.text_secondary(),
            value: self.text_primary(),
            selection: self.accent_bg(),
        }
    }

    /// Keycap: translucent fill, light border, radius 6. The 2px bottom
    /// edge of the spec is approximated with a 1px lip shadow under the
    /// 1px border.
    pub fn keycap_style(&self) -> container::Style {
        container::Style {
            text_color: Some(self.text_primary()),
            background: Some(Background::Color(self.keycap_bg())),
            border: Border {
                color: self.keycap_border(),
                width: 1.0,
                radius: KEYCAP_RADIUS.into(),
            },
            shadow: Shadow {
                color: self.keycap_border(),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 0.0,
            },
            ..Default::default()
        }
    }

    /// Secondary text style for counters, group headers and the footer.
    pub fn secondary_style(&self, _theme: &Theme) -> text::Style {
        text::Style {
            color: Some(self.text_secondary()),
        }
    }

    /// Primary text style for row titles and keycap labels.
    pub fn text_primary_style(&self, _theme: &Theme) -> text::Style {
        text::Style {
            color: Some(self.text_primary()),
        }
    }
}

/// Radii per SPEC-V2 §6: card 20, rows 10, search field 12, keycaps 6.
pub const CARD_RADIUS: f32 = 20.0;
pub const ROW_RADIUS: f32 = 10.0;
pub const FIELD_RADIUS: f32 = 12.0;
pub const KEYCAP_RADIUS: f32 = 6.0;

/// `#RRGGBB` (and `#RRGGBBAA`) → Color; `None` on anything else.
fn parse_hex(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#')?;
    let (r, g, b, a) = match hex.len() {
        6 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            255u8,
        ),
        8 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            u8::from_str_radix(&hex[6..8], 16).ok()?,
        ),
        _ => return None,
    };
    Some(Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    })
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing_handles_short_and_long_forms() {
        let color = parse_hex("#c8c093").expect("valid hex");
        assert_eq!(
            color,
            Color {
                r: 0xc8 as f32 / 255.0,
                g: 0xc0 as f32 / 255.0,
                b: 0x93 as f32 / 255.0,
                a: 1.0
            }
        );
        let with_alpha = parse_hex("#a78bfa80").expect("valid hex+alpha");
        assert!((with_alpha.a - 128.0 / 255.0).abs() < 1e-6);
        assert!(parse_hex("c8c093").is_none(), "no # prefix");
        assert!(parse_hex("#c8c0").is_none(), "too short");
        assert!(parse_hex("#zzzzzz").is_none(), "not hex");
    }

    #[test]
    fn kanagawa_cache_maps_tokens_to_roles() {
        let json = r##"{
            "scrim": "#000000",
            "surface_container": "#2a2a37",
            "surface_container_high": "#333343",
            "surface_container_highest": "#3c3c4e",
            "on_surface": "#c8c093",
            "on_surface_variant": "#717c7c",
            "outline": "#68697a",
            "outline_variant": "#363646",
            "primary": "#76946a",
            "shadow": "#1f1f28"
        }"##;
        let palette = Palette::from_json(json);
        assert_eq!(palette.text_primary(), parse_hex("#c8c093").unwrap());
        assert_eq!(palette.accent(), parse_hex("#76946a").unwrap());
        assert_eq!(palette.keycap_bg(), parse_hex("#3c3c4e").unwrap());
        // §6 alpha levels are applied on top of the tokens.
        assert!((palette.dim().a - 0.55).abs() < 1e-6);
        assert!((palette.card_bg().a - 0.94).abs() < 1e-6);
        assert!((palette.accent_bg().a - 0.16).abs() < 1e-6);
        assert_eq!(
            palette.dim(),
            Color {
                a: 0.55,
                ..parse_hex("#000000").unwrap()
            }
        );
    }

    #[test]
    fn broken_cache_falls_back_per_token() {
        let fallback = Palette::fallback();
        // Unparsable JSON: full fallback.
        assert_eq!(Palette::from_json("not json").accent(), fallback.accent());
        // Partial cache: known tokens apply, unknown ones keep §6.
        let partial = Palette::from_json(r##"{"primary": "#76946a"}"##);
        assert_eq!(partial.accent(), parse_hex("#76946a").unwrap());
        assert_eq!(partial.text_primary(), fallback.text_primary());
        // Wrong-typed value: fallback for that token.
        let wrong = Palette::from_json(r##"{"on_surface": 42}"##);
        assert_eq!(wrong.text_primary(), fallback.text_primary());
    }

    #[test]
    fn fallback_matches_spec_table() {
        // SPEC-V2 §6, exact tokens.
        let f = Palette::fallback();
        assert_eq!(f.on_surface, Color::from_rgb8(0xec, 0xe9, 0xf4));
        assert_eq!(f.on_surface_variant, Color::from_rgb8(0x9b, 0x93, 0xb3));
        assert_eq!(f.primary, Color::from_rgb8(0xa7, 0x8b, 0xfa));
        assert_eq!(
            f.dim(),
            Color::from_rgba(8.0 / 255.0, 6.0 / 255.0, 14.0 / 255.0, 0.55)
        );
        assert_eq!(
            f.card_bg(),
            Color::from_rgba(21.0 / 255.0, 18.0 / 255.0, 30.0 / 255.0, 0.94)
        );
    }

    #[test]
    fn radii_match_spec_table() {
        assert_eq!(CARD_RADIUS, 20.0);
        assert_eq!(ROW_RADIUS, 10.0);
        assert_eq!(FIELD_RADIUS, 12.0);
        assert_eq!(KEYCAP_RADIUS, 6.0);
    }
}
