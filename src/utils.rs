pub(crate) fn debug_utf8(bytes: &[u8]) -> &str {
    str::from_utf8(bytes).unwrap_or("[non-utf8]")
}

pub fn unescape_xml(input: &[u8]) -> String {
    let s = String::from_utf8_lossy(input);
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

pub fn escape_xml(input: &str) -> (String, char) {
    let quote = if input.contains('"') { '\'' } else { '"' };

    let mut escaped_body = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => escaped_body.push_str("&amp;"),
            '<' => escaped_body.push_str("&lt;"),
            '>' => escaped_body.push_str("&gt;"),
            '"' if quote == '"' => escaped_body.push_str("&quot;"),
            '\'' if quote == '\'' => escaped_body.push_str("&apos;"),
            _ => escaped_body.push(c),
        }
    }

    (escaped_body, quote)
}
