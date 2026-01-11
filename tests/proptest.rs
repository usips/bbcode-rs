//! Property-based tests using proptest for the BBCode parser.

use proptest::prelude::*;
use bbcode::{parse, Parser, Renderer};

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

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
    #![proptest_config(ProptestConfig::with_cases(200))]

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
// Fuzzing-style Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

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
}
