//! Avatar colors. Every bot is drawn as the Botloft mascot in its own color
//! (spec 15.3); the color identifies the bot and never encodes its state.

/// Default palette, assigned in order as bots join a crew. The owner may
/// also pick any other `#RRGGBB`.
pub const PALETTE: [&str; 16] = [
    "#FF7A59", "#5EC8FF", "#A48BFF", "#9BE564", "#FFC857", "#FF7EB6", "#3DD9C1", "#E8D5B0",
    "#F25F5C", "#4C7BFF", "#E26BE0", "#3FBF6F", "#FF9F43", "#8C9BB0", "#B07A54", "#E9EEF2",
];

/// Palette color for the `index`-th bot of a crew, wrapping around.
pub fn palette_color(index: usize) -> &'static str {
    PALETTE[index % PALETTE.len()]
}

/// Accepts `#RRGGBB` (any case) and returns it uppercased.
pub fn normalize_color(input: &str) -> Option<String> {
    let hex = input.trim().strip_prefix('#')?;
    let valid = hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit());
    valid.then(|| format!("#{}", hex.to_ascii_uppercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_wraps_around() {
        assert_eq!(palette_color(0), PALETTE[0]);
        assert_eq!(palette_color(PALETTE.len() + 1), PALETTE[1]);
    }

    #[test]
    fn palette_colors_are_valid() {
        for color in PALETTE {
            assert_eq!(normalize_color(color).as_deref(), Some(color));
        }
    }

    #[test]
    fn normalizes_or_rejects_colors() {
        assert_eq!(normalize_color("#a48bff").as_deref(), Some("#A48BFF"));
        assert_eq!(normalize_color("a48bff"), None);
        assert_eq!(normalize_color("#abc"), None);
        assert_eq!(normalize_color("#GGGGGG"), None);
    }
}
