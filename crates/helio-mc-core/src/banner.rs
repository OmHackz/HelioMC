//! The Helio sun banner. Rendered in the three icon modes.
//!
//! `render` returns the banner lines for the active mode. `banner_lines`
//! takes an explicit mode so callers can preview a different mode.

use crate::icons::Icons;

/// Number of lines the banner occupies in the terminal (height, including the
/// tagline).
pub const BANNER_LINES: usize = 7;

/// Render the banner in the active icon mode, with a tagline.
pub fn render() -> Vec<String> {
    banner_lines(
        &Icons::new(crate::icons::active_mode(), crate::icons::pad_default()),
        true,
    )
}

/// Render the banner lines for a specific mode. `with_tagline` adds a tagline
/// under the wordmark.
pub fn banner_lines(icons: &Icons, with_tagline: bool) -> Vec<String> {
    let sun = icons.render("Sun");
    let title = format!(
        "{NAME} — terminal Minecraft launcher",
        NAME = crate::brand::NAME
    );
    let mut lines = vec![
        "  ┌──────────────────────────────────────".into(),
        format!(
            "  │  {sun}  {NAME}    {TAGLINE_SHORT}",
            NAME = crate::brand::NAME,
            TAGLINE_SHORT = "terminal Minecraft launcher"
        ),
        "  │  ".into(),
        "  └──────────────────────────────────────".into(),
    ];
    if with_tagline {
        lines.push(title);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::IconMode;

    #[test]
    fn banner_renders_in_each_mode() {
        for mode in [IconMode::Nerd, IconMode::Unicode, IconMode::Ascii] {
            let icons = Icons::new(mode, 1);
            let lines = banner_lines(&icons, true);
            assert!(!lines.is_empty());
            for line in &lines {
                // No emoji codepoints in any banner line.
                for c in line.chars() {
                    assert!(!crate::icons::is_emoji(c), "emoji in banner: {c}");
                }
            }
        }
    }

    #[test]
    fn banner_height_is_stable() {
        let icons = Icons::new(IconMode::Unicode, 1);
        assert_eq!(banner_lines(&icons, false).len(), 4);
        assert_eq!(banner_lines(&icons, true).len(), 5);
    }
}
