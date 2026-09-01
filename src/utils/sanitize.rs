use regex::Regex;

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

pub fn validate_and_clean_string(input: &str) -> Result<String, String> {
    // 1. بررسی طول رشته
    let min_len = 3;
    let max_len = 100;
    if input.len() < min_len {
        return Err(format!("ورودی باید حداقل {} کاراکتر باشد.", min_len));
    }
    if input.len() > max_len {
        return Err(format!("ورودی نباید بیشتر از {} کاراکتر باشد.", max_len));
    }

    let forbidden_chars = [
        '\0', '\x01', '\x02', '\x03', '\x04', '\x05', '\x06', '\x07', '\x08', '\t', '\n', '\x0B',
        '\x0C', '\r', '\x0E', '\x0F', '\x10', '\x11', '\x12', '\x13', '\x14', '\x15', '\x16',
        '\x17', '\x18', '\x19', '\x1A', '\x1B', '\x1C', '\x1D', '\x1E', '\x1F', '"', '\'', ';',
        '<', '>', '=', '(', ')', '&', '|', '%', '#',
    ];
    for c in input.chars() {
        if forbidden_chars.contains(&c) {
            return Err(format!("ورودی حاوی کاراکتر غیرمجاز '{}' است.", c));
        }
    }

    let script_regex =
        Regex::new(r"(?i)<script.*?>.*?</script>|<img.*?src=.*?onerror=.*?>|javascript:").unwrap();
    if script_regex.is_match(input) {
        return Err("ورودی حاوی الگوی اسکریپت مشکوک است.".to_string());
    }

    let sql_injection_regex =
        Regex::new(r"(?i)UNION|SELECT|INSERT|DELETE|DROP|ALTER|CREATE|UPDATE").unwrap();
    if sql_injection_regex.is_match(input) {
        return Err("ورودی حاوی کلمات کلیدی مشکوک SQL است.".to_string());
    }

    let cleaned_input = input.to_string();

    Ok(cleaned_input)
}
