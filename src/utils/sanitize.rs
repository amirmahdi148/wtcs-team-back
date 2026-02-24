pub fn text(input: &str, max_len: usize) -> String {
    input
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(max_len)
        .map(|c| match c {
            '&' => ' ',
            '<' => ' ',
            '>' => ' ',
            '"' => ' ',
            '\'' => ' ',
            _ => c,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn username(input: &str, max_len: usize) -> String {
    let mut out = String::new();
    let mut prev_sep = false;

    for c in input.trim().chars() {
        if out.len() >= max_len {
            break;
        }

        let lc = c.to_ascii_lowercase();
        let allowed = lc.is_ascii_alphanumeric() || lc == '_' || lc == '-' || lc == '.';

        if allowed {
            let is_sep = lc == '_' || lc == '-' || lc == '.';
            if is_sep {
                if out.is_empty() || prev_sep {
                    continue;
                }
                prev_sep = true;
            } else {
                prev_sep = false;
            }
            out.push(lc);
        }
    }

    while out.ends_with('_') || out.ends_with('-') || out.ends_with('.') {
        out.pop();
    }

    out
}
