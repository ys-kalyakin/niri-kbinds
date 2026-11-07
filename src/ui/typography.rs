//! Overlay text metrics (change `overlay-font-scale`, design D1/D2): the
//! base sizes at scale 1.0 and the resolution of the font scale — the
//! `--font-scale` CLI flag wins over the `font_scale` token of the noctalia
//! theme cache, over the default 1.0; the result is clamped into
//! `[MIN_SCALE, MAX_SCALE]`.

/// Base sizes (px at scale 1.0), pinned by `consts_match_spec_table`
/// against the spec of change `overlay-font-scale`.
pub const ROW_TITLE: f32 = 18.0;
pub const KEYCAP: f32 = 14.0;
pub const SEARCH: f32 = 20.0;
pub const GROUP_HEADER: f32 = 15.0;
pub const CHEATSHEET_HEADER: f32 = 15.0;
pub const COUNTER_FOOTER: f32 = 13.0;

/// Clamp bounds of the font scale (spec: 0.5–3.0).
pub const MIN_SCALE: f32 = 0.5;
pub const MAX_SCALE: f32 = 3.0;

/// Resolve the font scale: the CLI flag wins over the theme-cache token,
/// the default is 1.0; the chosen value is clamped into
/// `[MIN_SCALE, MAX_SCALE]` (spec «Typography and font scale»).
pub fn resolve(cli: Option<f32>, cache: Option<f32>) -> f32 {
    cli.and_then(sane)
        .or(cache.and_then(sane))
        .unwrap_or(1.0)
        .clamp(MIN_SCALE, MAX_SCALE)
}

/// The `font_scale` token of the theme cache JSON: only a finite positive
/// number is valid — absent, malformed, zero, negative or non-finite falls
/// back per-token to `None` (design D5, like the color tokens).
pub fn cache_scale(json: &str) -> Option<f32> {
    let value = serde_json::from_str::<serde_json::Value>(json).ok()?;
    let scale = value.get("font_scale")?.as_f64()? as f32;
    sane(scale)
}

/// A usable scale value: finite and positive. Guards the renderer from
/// NaN/inf sizes (Rust's `f32` parser accepts "NaN", so the CLI can carry
/// it too); anything else falls through to the next source.
fn sane(scale: f32) -> Option<f32> {
    (scale.is_finite() && scale > 0.0).then_some(scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consts_match_spec_table() {
        // Base sizes of change `overlay-font-scale` (scale 1.0).
        assert_eq!(ROW_TITLE, 18.0);
        assert_eq!(KEYCAP, 14.0);
        assert_eq!(SEARCH, 20.0);
        assert_eq!(GROUP_HEADER, 15.0);
        assert_eq!(CHEATSHEET_HEADER, 15.0);
        assert_eq!(COUNTER_FOOTER, 13.0);
        assert_eq!(MIN_SCALE, 0.5);
        assert_eq!(MAX_SCALE, 3.0);
    }

    #[test]
    fn resolve_prefers_cli_over_cache_over_default() {
        assert_eq!(resolve(Some(1.5), Some(1.2)), 1.5, "CLI beats the cache");
        assert_eq!(resolve(None, Some(1.2)), 1.2, "cache without the flag");
        assert_eq!(resolve(None, None), 1.0, "default");
        assert_eq!(resolve(Some(1.5), None), 1.5);
    }

    #[test]
    fn resolve_clamps_into_range() {
        assert_eq!(resolve(Some(10.0), None), MAX_SCALE);
        assert_eq!(resolve(Some(0.1), None), MIN_SCALE);
        assert_eq!(
            resolve(None, Some(10.0)),
            MAX_SCALE,
            "cache value clamps too"
        );
        assert_eq!(resolve(None, Some(0.1)), MIN_SCALE);
        // A non-finite CLI value (Rust's f32 parser accepts "NaN") falls
        // through to the next source instead of reaching the renderer.
        assert_eq!(resolve(Some(f32::NAN), Some(1.2)), 1.2);
        assert_eq!(resolve(Some(f32::INFINITY), None), 1.0);
    }

    #[test]
    fn cache_scale_accepts_only_finite_positive_numbers() {
        assert_eq!(cache_scale(r##"{"font_scale": 1.2}"##), Some(1.2));
        assert_eq!(cache_scale(r##"{"font_scale": 2}"##), Some(2.0));
        // Per-token fallback on everything else (design D5).
        assert_eq!(cache_scale(r##"{}"##), None, "absent token");
        assert_eq!(cache_scale(r##"{"font_scale": "big"}"##), None, "string");
        assert_eq!(cache_scale(r##"{"font_scale": null}"##), None);
        assert_eq!(cache_scale(r##"{"font_scale": true}"##), None);
        assert_eq!(cache_scale(r##"{"font_scale": 0}"##), None, "zero");
        assert_eq!(cache_scale(r##"{"font_scale": -1.5}"##), None, "negative");
        assert_eq!(
            cache_scale(r##"{"font_scale": 1e999}"##),
            None,
            "overflows f64"
        );
        assert_eq!(cache_scale("not json"), None, "unparsable cache");
    }
}
