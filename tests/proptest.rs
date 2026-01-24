//! Property-based tests using proptest for the BBCode parser.
//!
//! These tests use random input generation to find edge cases and ensure
//! the parser never panics or produces unsafe output.

use proptest::prelude::*;
use bbcode::{parse, Parser, Renderer};

// ============================================================================
// Strategy Generators for Malicious Input
// ============================================================================

/// Generates strings that look like JavaScript protocol URLs
fn javascript_url_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("javascript:alert(1)".to_string()),
        Just("JAVASCRIPT:alert(1)".to_string()),
        Just("JaVaScRiPt:alert(1)".to_string()),
        Just("java\tscript:alert(1)".to_string()),
        Just("java\nscript:alert(1)".to_string()),
        Just("java\rscript:alert(1)".to_string()),
        Just("java\x00script:alert(1)".to_string()),
        Just("&#106;avascript:alert(1)".to_string()),
        Just("&#x6A;avascript:alert(1)".to_string()),
        Just("jav\x00ascript:alert(1)".to_string()),
        Just("j\x07avascript:alert(1)".to_string()),
    ]
}

/// Generates strings with event handler injection attempts
fn event_handler_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(" onclick=\"alert(1)\"".to_string()),
        Just(" onerror=\"alert(1)\"".to_string()),
        Just(" onload=\"alert(1)\"".to_string()),
        Just(" onmouseover=\"alert(1)\"".to_string()),
        Just(" onfocus=\"alert(1)\"".to_string()),
        Just(" onblur=\"alert(1)\"".to_string()),
        Just(" ONCLICK=\"alert(1)\"".to_string()),
        Just(" OnClick=\"alert(1)\"".to_string()),
        Just(" onanimationstart=\"alert(1)\"".to_string()),
        Just(" ontouchstart=\"alert(1)\"".to_string()),
        Just(" ondrag=\"alert(1)\"".to_string()),
        Just(" onscroll=\"alert(1)\"".to_string()),
    ]
}

/// Generates control characters that might bypass filters
fn control_char_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("\x00".to_string()),
        Just("\x01".to_string()),
        Just("\x02".to_string()),
        Just("\x07".to_string()),
        Just("\x08".to_string()),
        Just("\x0B".to_string()),
        Just("\x0C".to_string()),
        Just("\x0D".to_string()),
        Just("\x1B".to_string()),
        Just("\x00\x01".to_string()),
        Just("\x01\x02\x03".to_string()),
    ]
}

/// Generates HTML entity encoded strings
fn html_entity_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("&#60;script&#62;".to_string()),           // <script>
        Just("&#x3C;script&#x3E;".to_string()),         // <script>
        Just("&lt;script&gt;".to_string()),
        Just("&#60;&#62;".to_string()),                 // <>
        Just("&#34;onclick&#34;".to_string()),          // "onclick"
        Just("&#39;onclick&#39;".to_string()),          // 'onclick'
        Just("&quot;onclick=&quot;".to_string()),
        Just("&#106;avascript:".to_string()),           // javascript:
        Just("&#x6A;avascript:".to_string()),
        Just("&#60;img onerror=&#62;".to_string()),
    ]
}

/// Generates data: URL variations
fn data_url_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("data:text/html,<script>alert(1)</script>".to_string()),
        Just("data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==".to_string()),
        Just("DATA:text/html,<script>alert(1)</script>".to_string()),
        Just("data:image/svg+xml,<svg onload='alert(1)'>".to_string()),
        Just("data:application/xhtml+xml,<script>alert(1)</script>".to_string()),
        Just("data:  text/html,<script>alert(1)</script>".to_string()),
    ]
}

// ============================================================================
// Core Safety Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// Parsing should never panic on any input
    #[test]
    fn never_panics(s in ".*") {
        let _ = parse(&s);
    }

    /// Parsing empty or whitespace should produce valid output
    #[test]
    fn whitespace_handled(s in "\\s*") {
        let result = parse(&s);
        // Should not crash and should preserve whitespace
        prop_assert!(!result.is_empty() || s.is_empty());
    }

    /// Plain text should be escaped properly
    #[test]
    fn plain_text_escaped(s in "[a-zA-Z0-9 ]{0,100}") {
        let result = parse(&s);
        // No BBCode tags, so output should match input (except for auto-linking)
        // Check that it doesn't contain unescaped HTML
        prop_assert!(!result.contains('<') || result.contains("&lt;") || result.contains("<a"));
    }

    /// Simple tags should always produce matching HTML tags
    #[test]
    fn simple_tags_balanced(text in "[a-zA-Z0-9 ]{1,20}") {
        let input = format!("[b]{}[/b]", text);
        let result = parse(&input);
        
        let open_count = result.matches("<strong>").count();
        let close_count = result.matches("</strong>").count();
        prop_assert_eq!(open_count, close_count);
    }

    /// Nested tags should produce nested HTML
    #[test]
    fn nested_tags_work(text in "[a-zA-Z]{1,10}") {
        let input = format!("[b][i]{}[/i][/b]", text);
        let result = parse(&input);
        
        prop_assert!(result.contains("<strong>"));
        prop_assert!(result.contains("<em>"));
        prop_assert!(result.contains("</em>"));
        prop_assert!(result.contains("</strong>"));
    }

    /// URL validation should reject bad schemes
    #[test]
    fn url_schemes_validated(path in "[a-zA-Z0-9/]{0,20}") {
        let js_url = format!("[url=javascript:{}]Link[/url]", path);
        let result = parse(&js_url);
        prop_assert!(!result.contains("href=\"javascript"));

        let data_url = format!("[url=data:text/html,{}]Link[/url]", path);
        let result = parse(&data_url);
        prop_assert!(!result.contains("href=\"data:"));
    }

    /// HTML special characters should always be escaped
    #[test]
    fn html_chars_escaped(s in ".*<.*>.*") {
        let result = parse(&s);
        // Raw < and > should not appear in output (except in valid HTML tags we generate)
        // They should be escaped as &lt; and &gt;
        for c in result.chars() {
            if c == '<' {
                // Should only be from our generated HTML tags
                let idx = result.find('<').unwrap();
                let rest = &result[idx..];
                // Check it starts with a valid tag
                prop_assert!(
                    rest.starts_with("<strong") ||
                    rest.starts_with("<em") ||
                    rest.starts_with("<u>") ||
                    rest.starts_with("<s>") ||
                    rest.starts_with("<a ") ||
                    rest.starts_with("<span") ||
                    rest.starts_with("<div") ||
                    rest.starts_with("<pre") ||
                    rest.starts_with("<code") ||
                    rest.starts_with("<img") ||
                    rest.starts_with("<blockquote") ||
                    rest.starts_with("<ul") ||
                    rest.starts_with("<ol") ||
                    rest.starts_with("<li") ||
                    rest.starts_with("<table") ||
                    rest.starts_with("<tr") ||
                    rest.starts_with("<td") ||
                    rest.starts_with("<th") ||
                    rest.starts_with("<h") ||
                    rest.starts_with("<hr") ||
                    rest.starts_with("<br") ||
                    rest.starts_with("<details") ||
                    rest.starts_with("<summary") ||
                    rest.starts_with("<sub") ||
                    rest.starts_with("<sup") ||
                    rest.starts_with("</") ||
                    rest.starts_with("<!"),
                    "Found unexpected < at: {}", &rest[..rest.len().min(50)]
                );
            }
        }
    }

    /// Output length should be reasonable (no explosion)
    #[test]
    fn output_length_bounded(s in ".{0,100}") {
        let result = parse(&s);
        // Output shouldn't be more than 10x input length (generous bound)
        let max_len = (s.len() + 1) * 10 + 100;
        prop_assert!(result.len() <= max_len, 
            "Output too long: {} chars for input of {} chars",
            result.len(), s.len());
    }

    /// Color values should be validated
    #[test]
    fn color_validated(color in "[a-zA-Z0-9#]{1,20}") {
        let input = format!("[color={}]text[/color]", color);
        let result = parse(&input);
        
        // If color appears in style, it should be properly formatted
        if result.contains("style=") {
            prop_assert!(
                !result.contains("style=\"color: ;") &&
                !result.contains("style=\"color: \""),
                "Invalid color format in output"
            );
        }
    }

    /// Quote with author should include attribution
    #[test]
    fn quote_author_included(author in "[a-zA-Z][a-zA-Z0-9]{0,10}") {
        let input = format!("[quote=\"{}\"]Some text[/quote]", author);
        let result = parse(&input);
        
        prop_assert!(
            result.contains(&author) || result.contains("blockquote"),
            "Author should appear in quote or blockquote should exist"
        );
    }

    /// List items should always be in a list
    #[test]
    fn list_items_in_list(items in prop::collection::vec("[a-zA-Z]{1,5}", 1..5)) {
        let mut input = String::from("[list]");
        for item in &items {
            input.push_str(&format!("[*]{}", item));
        }
        input.push_str("[/list]");

        let result = parse(&input);
        
        // Should have list container
        prop_assert!(
            result.contains("<ul") || result.contains("<ol"),
            "List should have container"
        );
        
        // Should have list items
        for item in &items {
            prop_assert!(result.contains(item), "Item {} should appear in output", item);
        }
    }

    /// Deeply nested tags should not cause stack overflow
    #[test]
    fn deep_nesting_safe(depth in 1usize..50) {
        let mut input = String::new();
        for _ in 0..depth {
            input.push_str("[b]");
        }
        input.push_str("deep");
        for _ in 0..depth {
            input.push_str("[/b]");
        }

        let result = parse(&input);
        prop_assert!(result.contains("deep"));
    }
}

// ============================================================================
// Roundtrip and Idempotence Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Parsing the same input twice should give the same result
    #[test]
    fn parsing_deterministic(s in ".*") {
        let result1 = parse(&s);
        let result2 = parse(&s);
        prop_assert_eq!(result1, result2);
    }

    /// Parser and renderer should be consistent across configurations
    #[test]
    fn config_consistency(text in "[a-zA-Z ]{1,20}") {
        let input = format!("[b]{}[/b]", text);

        let parser = Parser::new();
        let renderer = Renderer::new();

        let doc = parser.parse(&input);
        let html = renderer.render(&doc);

        prop_assert!(html.contains("<strong>"));
        prop_assert!(html.contains("</strong>"));
    }
}

// ============================================================================
// Aggressive Fuzzing-style Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    /// Random bracket sequences should never crash
    #[test]
    fn bracket_sequences_safe(s in "[\\[\\]a-z=/]{0,50}") {
        let _ = parse(&s);
    }

    /// Random tag-like sequences should never crash
    #[test]
    fn taglike_sequences_safe(s in "(\\[[a-z]{0,5}(=[^\\]]{0,10})?\\]|\\[/[a-z]{0,5}\\]|[^\\[\\]]){0,20}") {
        let _ = parse(&s);
    }

    /// Unicode input should never crash
    #[test]
    fn unicode_safe(s in "\\PC{0,100}") {
        let _ = parse(&s);
    }

    /// Completely random bytes should never crash
    #[test]
    fn random_bytes_safe(bytes in prop::collection::vec(any::<u8>(), 0..200)) {
        if let Ok(s) = String::from_utf8(bytes) {
            let _ = parse(&s);
        }
    }

    /// Random ASCII with high bracket density
    #[test]
    fn bracket_heavy_input(s in "[\\[\\]/=a-zA-Z0-9\"' ]{0,100}") {
        let _ = parse(&s);
    }
}

// ============================================================================
// XSS-Focused Security Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// JavaScript URLs should never appear in href attributes
    #[test]
    fn no_javascript_urls(url in javascript_url_strategy()) {
        let input = format!("[url={}]Click[/url]", url);
        let result = parse(&input);
        prop_assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "JavaScript URL leaked: {}", result
        );
    }

    /// Data URLs should never appear in href or src attributes
    #[test]
    fn no_data_urls(url in data_url_strategy()) {
        let input = format!("[url={}]Click[/url]", url);
        let result = parse(&input);
        prop_assert!(
            !result.to_lowercase().contains("href=\"data:"),
            "Data URL leaked in href: {}", result
        );

        let img_input = format!("[img]{}[/img]", url);
        let img_result = parse(&img_input);
        prop_assert!(
            !img_result.to_lowercase().contains("src=\"data:"),
            "Data URL leaked in src: {}", img_result
        );
    }

    /// Event handlers should never appear as actual HTML attributes
    #[test]
    fn no_event_handlers_in_url(handler in event_handler_strategy(), path in "[a-z]{0,10}") {
        let input = format!("[url=http://x.com/{}{}]Click[/url]", path, handler);
        let result = parse(&input);

        // Check that the event handler doesn't appear in an HTML attribute context
        let lower = result.to_lowercase();
        let handlers = ["onclick=", "onerror=", "onload=", "onmouseover=", "onfocus=",
                       "onblur=", "onanimationstart=", "ontouchstart="];

        for h in handlers {
            if lower.contains(h) {
                // Make sure it's not inside an actual HTML tag
                let in_tag = is_in_html_tag(&result, h);
                prop_assert!(!in_tag, "Event handler {} leaked in HTML tag: {}", h, result);
            }
        }
    }

    /// Control characters should not allow protocol bypass
    #[test]
    fn control_chars_no_bypass(ctrl in control_char_strategy()) {
        let input = format!("[url={}javascript:alert(1)]Click[/url]", ctrl);
        let result = parse(&input);
        prop_assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Control char bypass: {}", result
        );
    }

    /// HTML entities should not decode into dangerous content
    #[test]
    fn html_entities_safe(entities in html_entity_strategy()) {
        let input = format!("[url={}]Click[/url]", entities);
        let result = parse(&input);
        prop_assert!(
            !result.contains("<script>") && !result.contains("<img") || result.contains("&lt;"),
            "HTML entity produced dangerous content: {}", result
        );
    }

    /// Script tags should always be escaped
    #[test]
    fn script_tags_escaped(payload in "[a-zA-Z0-9()]{0,30}") {
        let input = format!("<script>{}</script>", payload);
        let result = parse(&input);
        prop_assert!(
            !result.contains("<script>"),
            "Script tag not escaped: {}", result
        );
        prop_assert!(
            result.contains("&lt;script&gt;"),
            "Script tag should be HTML escaped: {}", result
        );
    }

    /// Quote attribute injection should be blocked
    #[test]
    fn quote_breakout_blocked(payload in "[a-zA-Z0-9()]{0,20}") {
        let input = format!(r#"[url=http://x.com" onclick="{}"]Click[/url]"#, payload);
        let result = parse(&input);
        // Either rendered as text (safe) or onclick not in HTML attribute
        let has_dangerous_onclick = result.contains("<a ") &&
            result.to_lowercase().contains(" onclick=");
        prop_assert!(!has_dangerous_onclick, "Quote breakout leaked: {}", result);
    }

    /// Img tag injection attempts should be blocked
    #[test]
    fn img_onerror_blocked(payload in "[a-zA-Z0-9()]{0,20}") {
        let input = format!(r#"[img]http://x.com/x.jpg" onerror="{}[/img]"#, payload);
        let result = parse(&input);
        let has_dangerous_onerror = result.contains("<img") &&
            result.to_lowercase().contains(" onerror=");
        prop_assert!(!has_dangerous_onerror, "Img onerror leaked: {}", result);
    }
}

// ============================================================================
// Stress Tests for Parser Robustness
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Extremely deep nesting should not cause stack overflow
    #[test]
    fn extreme_nesting_safe(depth in 1usize..200) {
        let mut input = String::new();
        for _ in 0..depth {
            input.push_str("[b]");
        }
        input.push_str("X");
        for _ in 0..depth {
            input.push_str("[/b]");
        }
        let _ = parse(&input);
    }

    /// Many unclosed tags should not cause issues
    #[test]
    fn many_unclosed_tags(count in 1usize..100) {
        let input = "[b]".repeat(count);
        let _ = parse(&input);
    }

    /// Many close tags without opens should not cause issues
    #[test]
    fn many_unmatched_closes(count in 1usize..100) {
        let input = "[/b]".repeat(count);
        let _ = parse(&input);
    }

    /// Interleaved unclosed tags
    #[test]
    fn interleaved_unclosed(count in 1usize..50) {
        let mut input = String::new();
        for i in 0..count {
            match i % 4 {
                0 => input.push_str("[b]"),
                1 => input.push_str("[i]"),
                2 => input.push_str("[u]"),
                _ => input.push_str("[s]"),
            }
        }
        let _ = parse(&input);
    }

    /// Very long tag options should not cause issues
    #[test]
    fn long_option_value(len in 1usize..1000) {
        let value = "a".repeat(len);
        let input = format!("[color={}]text[/color]", value);
        let _ = parse(&input);
    }

    /// Very long content should not cause issues
    #[test]
    fn long_content(len in 1usize..10000) {
        let content = "x".repeat(len);
        let input = format!("[b]{}[/b]", content);
        let result = parse(&input);
        prop_assert!(result.len() >= len);
    }

    /// Many different tags should work
    #[test]
    fn many_different_tags(count in 1usize..50) {
        let tags = ["b", "i", "u", "s", "code", "quote"];
        let mut input = String::new();
        for i in 0..count {
            let tag = tags[i % tags.len()];
            input.push_str(&format!("[{}]x[/{}]", tag, tag));
        }
        let _ = parse(&input);
    }

    /// Random valid BBCode structure
    #[test]
    fn random_valid_structure(
        tag in prop_oneof![Just("b"), Just("i"), Just("u"), Just("s")],
        text in "[a-zA-Z0-9 ]{1,20}"
    ) {
        let input = format!("[{}]{}[/{}]", tag, text, tag);
        let result = parse(&input);
        prop_assert!(result.contains(&text));
    }
}

// ============================================================================
// URL Fuzzing Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// Random protocol schemes should not produce dangerous output
    #[test]
    fn random_protocols_safe(proto in "[a-zA-Z]{1,15}", path in "[a-zA-Z0-9/]{0,30}") {
        let input = format!("[url={}:{}]Click[/url]", proto, path);
        let result = parse(&input);

        let lower_result = result.to_lowercase();
        let dangerous_protocols = ["javascript:", "vbscript:", "data:"];

        for dp in dangerous_protocols {
            prop_assert!(
                !lower_result.contains(&format!("href=\"{}", dp)),
                "Dangerous protocol {} leaked: {}", dp, result
            );
        }
    }

    /// URL with random junk should not crash or produce dangerous output
    #[test]
    fn url_with_junk(junk in "[^\\]]{0,50}") {
        let input = format!("[url=http://x.com/{}]Click[/url]", junk);
        let result = parse(&input);
        // Should not contain unescaped dangerous content
        prop_assert!(!result.contains("<script>"));
    }

    /// Nested URL tags should not produce dangerous output
    #[test]
    fn nested_url_safe(inner in "[a-zA-Z:/.]{0,30}") {
        let input = format!("[url=[url={}]inner[/url]]outer[/url]", inner);
        let result = parse(&input);
        prop_assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Nested URL bypass: {}", result
        );
    }
}

// ============================================================================
// Color/Style Injection Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Color values with injection attempts should be blocked
    #[test]
    fn color_injection_blocked(payload in "[a-zA-Z0-9;:()]{0,30}") {
        let input = format!("[color=red;{}]text[/color]", payload);
        let result = parse(&input);
        // Should not have expression() or javascript:
        prop_assert!(
            !result.to_lowercase().contains("expression(") &&
            !result.to_lowercase().contains("javascript:"),
            "Color injection leaked: {}", result
        );
    }

    /// Size values with injection attempts should be blocked
    #[test]
    fn size_injection_blocked(payload in "[a-zA-Z0-9;:()\"]{0,30}") {
        let input = format!("[size=12{}]text[/size]", payload);
        let result = parse(&input);
        // Should not have onclick or other handlers
        let has_handler = result.to_lowercase().contains(" onclick=") ||
            result.to_lowercase().contains(" onerror=");
        prop_assert!(!has_handler, "Size injection leaked: {}", result);
    }

    /// Font values with injection attempts should be blocked
    #[test]
    fn font_injection_blocked(payload in "[a-zA-Z0-9;:()\"]{0,30}") {
        let input = format!("[font=Arial{}]text[/font]", payload);
        let result = parse(&input);
        let has_handler = result.to_lowercase().contains(" onclick=") ||
            result.to_lowercase().contains(" onerror=");
        prop_assert!(!has_handler, "Font injection leaked: {}", result);
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if a pattern appears inside an HTML tag (between < and >)
fn is_in_html_tag(html: &str, pattern: &str) -> bool {
    let lower = html.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    for (i, _) in lower.match_indices(&pattern_lower) {
        let before = &html[..i];
        let after = &html[i..];

        let last_open = before.rfind('<');
        let last_close = before.rfind('>');

        if let Some(open_pos) = last_open {
            if last_close.map_or(true, |close_pos| open_pos > close_pos) {
                if after.contains('>') {
                    return true;
                }
            }
        }
    }
    false
}
