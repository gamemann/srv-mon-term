/// Strips Quake 3 style color codes (`^1`, `^7`, ...) from a string.
pub fn strip_quake_colors(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '^' && chars.peek().is_some_and(|n| n.is_ascii_alphanumeric()) {
            chars.next();

            continue;
        }

        out.push(c);
    }

    out
}

/// Strips Minecraft section-sign formatting codes (`§a`, `§l`, ...) from a string.
pub fn strip_minecraft_colors(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();

            continue;
        }

        out.push(c);
    }

    out
}

/// Collapses new lines and control characters so a value stays on a single TUI row.
pub fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_quake_colors() {
        assert_eq!(strip_quake_colors("^1Best ^7Server"), "Best Server");
        assert_eq!(strip_quake_colors("no colors"), "no colors");
        assert_eq!(strip_quake_colors("trailing^"), "trailing^");
    }

    #[test]
    fn strips_minecraft_colors() {
        assert_eq!(strip_minecraft_colors("§aGreen §lBold"), "Green Bold");
        assert_eq!(strip_minecraft_colors("plain"), "plain");
    }

    #[test]
    fn sanitizes_control_chars() {
        assert_eq!(sanitize("line one\nline two\t "), "line one line two");
    }
}
