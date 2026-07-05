pub fn truncate_str(s: &str, max: usize) -> String {
    let char_count = s.chars().count();

    if char_count <= max {
        s.to_string()
    } else {
        // Ex: Like thi...
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}
