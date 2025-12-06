/// HTML encoding utilities
pub struct XssEncoder;

impl XssEncoder {
    /// Encode HTML entities to prevent XSS
    pub fn encode_html(text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&#x27;".to_string(),
                '&' => "&amp;".to_string(),
                '/' => "&#x2F;".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }

    /// Encode HTML attributes
    pub fn encode_html_attribute(text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '"' => "&quot;".to_string(),
                '\'' => "&#x27;".to_string(),
                '&' => "&amp;".to_string(),
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }

    /// Encode for JavaScript context
    pub fn encode_javascript(text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '\\' => "\\\\".to_string(),
                '"' => "\\\"".to_string(),
                '\'' => "\\'".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '<' => "\\x3C".to_string(),
                '>' => "\\x3E".to_string(),
                '/' => "\\/".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }

    /// Encode for URL context
    pub fn encode_url(text: &str) -> String {
        text.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                    c.to_string()
                } else {
                    format!("%{:02X}", c as u8)
                }
            })
            .collect()
    }

    /// Encode for CSS context
    pub fn encode_css(text: &str) -> String {
        text.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_string()
                } else {
                    format!("\\{:X} ", c as u32)
                }
            })
            .collect()
    }

    /// Decode HTML entities
    pub fn decode_html(text: &str) -> String {
        text.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
            .replace("&#x2F;", "/")
            .replace("&amp;", "&") // Must be last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_html() {
        let input = r#"<script>alert("XSS")</script>"#;
        let output = XssEncoder::encode_html(input);

        assert_eq!(
            output,
            "&lt;script&gt;alert(&quot;XSS&quot;)&lt;&#x2F;script&gt;"
        );
        assert!(!output.contains('<'));
        assert!(!output.contains('>'));
    }

    #[test]
    fn test_encode_html_attribute() {
        let input = r#"Hello" onclick="alert('XSS')"#;
        let output = XssEncoder::encode_html_attribute(input);

        assert!(output.contains("&quot;"));
        assert!(output.contains("&#x27;"));
    }

    #[test]
    fn test_encode_javascript() {
        let input = r#"'; alert('XSS'); //'"#;
        let output = XssEncoder::encode_javascript(input);

        assert!(output.contains("\\'"));
        assert!(!output.contains("';"));
    }

    #[test]
    fn test_encode_url() {
        let input = "hello world&test=value";
        let output = XssEncoder::encode_url(input);

        assert!(output.contains("%20")); // space
        assert!(output.contains("%26")); // &
    }

    #[test]
    fn test_encode_css() {
        let input = "expression(alert('XSS'))";
        let output = XssEncoder::encode_css(input);

        assert!(!output.contains('('));
        assert!(!output.contains(')'));
    }

    #[test]
    fn test_decode_html() {
        let input = "&lt;script&gt;alert(&quot;XSS&quot;)&lt;&#x2F;script&gt;";
        let output = XssEncoder::decode_html(input);

        assert_eq!(output, r#"<script>alert("XSS")</script>"#);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = r#"<div class="test">Hello & "goodbye"</div>"#;
        let encoded = XssEncoder::encode_html(original);
        let decoded = XssEncoder::decode_html(&encoded);

        assert_eq!(original, decoded);
    }
}


