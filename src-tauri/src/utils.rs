pub fn format_seconds(seconds: u64) -> String {
    let h = seconds / 3600;
    let rem = seconds % 3600;
    let m = rem / 60;
    let s = rem % 60;

    let mut parts: Vec<String> = Vec::with_capacity(3);
    if h > 0 {
        parts.push(format!("{}h", h));
    }
    if m > 0 {
        parts.push(format!("{}m", m));
    }
    parts.push(format!("{}s", s));

    parts.join(" ")
}
