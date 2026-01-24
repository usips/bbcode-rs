//! Comprehensive integration tests for the BBCode parser.
//!
//! These tests verify the full parsing and rendering pipeline
//! with realistic BBCode content.

use bbcode::parse;

// ============================================================================
// phpBB Compatibility Tests
// ============================================================================

mod phpbb_compat {
    use super::*;

    #[test]
    fn basic_formatting() {
        assert_eq!(parse("[b]Bold[/b]"), "<strong>Bold</strong>");
        assert_eq!(parse("[i]Italic[/i]"), "<em>Italic</em>");
        assert_eq!(parse("[u]Underline[/u]"), "<u>Underline</u>");
    }

    #[test]
    fn url_tag_with_url() {
        let result = parse("[url=https://example.com]Example[/url]");
        assert!(result.contains("href=\"https://example.com\""));
        assert!(result.contains(">Example</a>"));
    }

    #[test]
    fn url_tag_content_only() {
        let result = parse("[url]https://example.com[/url]");
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn email_tag() {
        let result = parse("[email]test@example.com[/email]");
        assert!(result.contains("mailto:test@example.com"));
    }

    #[test]
    fn email_tag_with_text() {
        let result = parse("[email=test@example.com]Contact Us[/email]");
        assert!(result.contains("mailto:test@example.com"));
        assert!(result.contains(">Contact Us</a>"));
    }

    #[test]
    fn size_percentage() {
        let result = parse("[size=150]Large[/size]");
        // phpBB uses percentage for size (50-200)
        assert!(result.contains("font-size"));
    }

    #[test]
    fn color_hex() {
        let result = parse("[color=#FF0000]Red[/color]");
        assert!(result.contains("color: #FF0000"));
    }

    #[test]
    fn color_named() {
        let result = parse("[color=blue]Blue[/color]");
        assert!(result.contains("color: blue"));
    }

    #[test]
    fn quote_simple() {
        let result = parse("[quote]This is a quote[/quote]");
        assert!(result.contains("<blockquote"));
        assert!(result.contains("This is a quote"));
    }

    #[test]
    fn quote_with_author() {
        let result = parse("[quote=\"username\"]This is quoted[/quote]");
        assert!(result.contains("username wrote:"));
    }

    #[test]
    fn code_block() {
        let result = parse("[code]echo 'Hello';[/code]");
        assert!(result.contains("<pre"));
        assert!(result.contains("<code>"));
    }

    #[test]
    fn code_with_language() {
        let result = parse("[code=php]echo 'Hello';[/code]");
        assert!(result.contains("language-php"));
    }

    #[test]
    fn list_unordered() {
        let result = parse("[list][*]Item 1[*]Item 2[/list]");
        assert!(result.contains("<ul"));
        assert!(result.contains("<li>Item 1</li>"));
    }

    #[test]
    fn list_ordered_decimal() {
        let result = parse("[list=1][*]First[*]Second[/list]");
        assert!(result.contains("<ol"));
        assert!(result.contains("type=\"1\""));
    }

    #[test]
    fn list_ordered_alpha() {
        let result = parse("[list=a][*]A[*]B[/list]");
        assert!(result.contains("type=\"a\""));
    }

    #[test]
    fn list_disc() {
        let result = parse("[list=disc][*]Disc item[/list]");
        assert!(result.contains("list-style-type: disc"));
    }

    #[test]
    fn img_simple() {
        let result = parse("[img]https://example.com/image.png[/img]");
        assert!(result.contains("<img"));
        assert!(result.contains("src=\"https://example.com/image.png\""));
    }
}

// ============================================================================
// XenForo Compatibility Tests
// ============================================================================

mod xenforo_compat {
    use super::*;

    #[test]
    fn basic_formatting() {
        assert_eq!(parse("[B]Bold[/B]"), "<strong>Bold</strong>");
        assert_eq!(parse("[I]Italic[/I]"), "<em>Italic</em>");
        assert_eq!(parse("[U]Underline[/U]"), "<u>Underline</u>");
        assert_eq!(parse("[S]Strike[/S]"), "<s>Strike</s>");
    }

    #[test]
    fn size_xenforo_scale() {
        // XenForo uses 1-7 scale
        let result = parse("[size=4]Normal[/size]");
        assert!(result.contains("font-size: 15px"));

        let result = parse("[size=7]Huge[/size]");
        assert!(result.contains("font-size: 26px"));
    }

    #[test]
    fn left_align() {
        let result = parse("[LEFT]Left aligned[/LEFT]");
        assert!(result.contains("text-align: left"));
    }

    #[test]
    fn center_align() {
        let result = parse("[CENTER]Centered[/CENTER]");
        assert!(result.contains("text-align: center"));
    }

    #[test]
    fn right_align() {
        let result = parse("[RIGHT]Right aligned[/RIGHT]");
        assert!(result.contains("text-align: right"));
    }

    #[test]
    fn justify_align() {
        let result = parse("[JUSTIFY]Justified text[/JUSTIFY]");
        assert!(result.contains("text-align: justify"));
    }

    #[test]
    fn indent() {
        let result = parse("[INDENT]Indented[/INDENT]");
        assert!(result.contains("margin-left: 20px"));
    }

    #[test]
    fn indent_level() {
        let result = parse("[INDENT=3]Deeply indented[/INDENT]");
        assert!(result.contains("margin-left: 60px"));
    }

    #[test]
    fn heading() {
        let result = parse("[HEADING=1]Title[/HEADING]");
        assert!(result.contains("<h2"));
    }

    #[test]
    fn heading_level_3() {
        let result = parse("[HEADING=3]Subheading[/HEADING]");
        assert!(result.contains("<h4"));
    }

    #[test]
    fn spoiler() {
        let result = parse("[SPOILER]Hidden content[/SPOILER]");
        assert!(result.contains("<details"));
        assert!(result.contains("<summary>"));
    }

    #[test]
    fn spoiler_with_title() {
        let result = parse("[SPOILER=\"Click to reveal\"]Secret[/SPOILER]");
        assert!(result.contains("Click to reveal"));
    }

    #[test]
    fn ispoiler() {
        let result = parse("This is [ISPOILER]hidden[/ISPOILER] inline");
        assert!(result.contains("bbcode-ispoiler"));
    }

    #[test]
    fn icode() {
        let result = parse("Use [ICODE]console.log()[/ICODE] for debugging");
        assert!(result.contains("<code"));
        assert!(result.contains("console.log()"));
    }

    #[test]
    fn plain() {
        let result = parse("[PLAIN][B]Not bold[/B][/PLAIN]");
        assert!(!result.contains("<strong>"));
        assert!(result.contains("[B]Not bold[/B]"));
    }

    #[test]
    fn table() {
        let result = parse("[TABLE][TR][TD]Cell 1[/TD][TD]Cell 2[/TD][/TR][/TABLE]");
        assert!(result.contains("<table"));
        assert!(result.contains("<tr>"));
        assert!(result.contains("<td>"));
    }

    #[test]
    fn table_with_header() {
        let result = parse("[TABLE][TR][TH]Header[/TH][/TR][TR][TD]Data[/TD][/TR][/TABLE]");
        assert!(result.contains("<th>Header</th>"));
        assert!(result.contains("<td>Data</td>"));
    }

    #[test]
    fn hr() {
        let result = parse("Before[HR]After");
        assert!(result.contains("<hr />"));
    }
}

// ============================================================================
// Nesting Tests
// ============================================================================

mod nesting {
    use super::*;

    #[test]
    fn simple_nesting() {
        assert_eq!(
            parse("[b][i]Bold and Italic[/i][/b]"),
            "<strong><em>Bold and Italic</em></strong>"
        );
    }

    #[test]
    fn triple_nesting() {
        assert_eq!(
            parse("[b][i][u]All three[/u][/i][/b]"),
            "<strong><em><u>All three</u></em></strong>"
        );
    }

    #[test]
    fn quote_with_formatting() {
        let result = parse("[quote][b]Bold in quote[/b][/quote]");
        assert!(result.contains("<blockquote"));
        assert!(result.contains("<strong>Bold in quote</strong>"));
    }

    #[test]
    fn list_with_formatting() {
        let result = parse("[list][*][b]Bold item[/b][*][i]Italic item[/i][/list]");
        assert!(result.contains("<strong>Bold item</strong>"));
        assert!(result.contains("<em>Italic item</em>"));
    }

    #[test]
    fn url_with_formatting() {
        let result = parse("[url=https://example.com][b]Bold Link[/b][/url]");
        assert!(result.contains("<strong>Bold Link</strong>"));
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn deep_nesting_10_levels() {
        let mut input = String::new();
        for _ in 0..10 {
            input.push_str("[b]");
        }
        input.push_str("deep");
        for _ in 0..10 {
            input.push_str("[/b]");
        }

        let result = parse(&input);
        assert!(result.contains("deep"));
        // Count <strong> tags
        let count = result.matches("<strong>").count();
        assert_eq!(count, 10);
    }
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn empty_input() {
        assert_eq!(parse(""), "");
    }

    #[test]
    fn whitespace_only() {
        assert_eq!(parse("   "), "   ");
    }

    #[test]
    fn unclosed_tag() {
        let result = parse("[b]Bold without close");
        assert!(result.contains("<strong>"));
        assert!(result.contains("Bold without close"));
    }

    #[test]
    fn unmatched_close_tag() {
        let result = parse("text[/b]more");
        assert!(result.contains("[/b]"));
    }

    #[test]
    fn empty_tag() {
        assert_eq!(parse("[b][/b]"), "<strong></strong>");
    }

    #[test]
    fn empty_brackets() {
        let result = parse("[]");
        assert!(result.contains("[]"));
    }

    #[test]
    fn single_open_bracket() {
        let result = parse("[");
        assert_eq!(result, "[");
    }

    #[test]
    fn single_close_bracket() {
        let result = parse("]");
        assert_eq!(result, "]");
    }

    #[test]
    fn nested_brackets() {
        let result = parse("[[b]]text[[/b]]");
        // Should handle this gracefully
        assert!(result.contains("text"));
    }

    #[test]
    fn unknown_tag() {
        let result = parse("[foo]text[/foo]");
        assert!(result.contains("[foo]"));
        assert!(result.contains("[/foo]"));
    }

    #[test]
    fn tag_with_number() {
        let result = parse("[h1]Heading[/h1]");
        // h1 is not a standard BBCode tag
        assert!(result.contains("[h1]"));
    }

    #[test]
    fn mismatched_tags() {
        let result = parse("[b][i]text[/b][/i]");
        // Should handle gracefully
        assert!(result.contains("text"));
    }

    #[test]
    fn interleaved_tags() {
        let result = parse("[b]bold[i]both[/b]italic[/i]");
        // At least one tag should be processed
        assert!(result.contains("bold") || result.contains("both") || result.contains("italic"));
        // Parser handles mismatched tags gracefully
        assert!(result.contains("<strong>") || result.contains("<em>") || result.contains("[b]"));
    }

    #[test]
    fn very_long_content() {
        let long_text = "a".repeat(100000);
        let result = parse(&format!("[b]{}[/b]", long_text));
        assert!(result.contains(&long_text));
    }

    #[test]
    fn many_short_tags() {
        let input = "[b]x[/b]".repeat(1000);
        let result = parse(&input);
        let count = result.matches("<strong>").count();
        assert_eq!(count, 1000);
    }

    #[test]
    fn special_characters_in_content() {
        assert_eq!(parse("5 > 3 and 3 < 5"), "5 &gt; 3 and 3 &lt; 5");
    }

    #[test]
    fn ampersand_in_content() {
        assert_eq!(parse("AT&T"), "AT&amp;T");
    }

    #[test]
    fn quotes_in_content() {
        assert_eq!(parse("He said \"Hello\""), "He said &quot;Hello&quot;");
    }
}

// ============================================================================
// URL Handling Tests
// ============================================================================

mod url_handling {
    use super::*;

    #[test]
    fn url_http() {
        let result = parse("[url=http://example.com]Link[/url]");
        assert!(result.contains("href=\"http://example.com\""));
    }

    #[test]
    fn url_https() {
        let result = parse("[url=https://example.com]Link[/url]");
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn url_with_path() {
        let result = parse("[url=https://example.com/path/to/page]Link[/url]");
        assert!(result.contains("href=\"https://example.com/path/to/page\""));
    }

    #[test]
    fn url_with_query() {
        let result = parse("[url=https://example.com?q=test]Link[/url]");
        assert!(result.contains("href=\"https://example.com?q=test\""));
    }

    #[test]
    fn url_with_fragment() {
        let result = parse("[url=https://example.com#section]Link[/url]");
        assert!(result.contains("href=\"https://example.com#section\""));
    }

    #[test]
    fn auto_url_https() {
        let result = parse("Visit https://example.com today!");
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn auto_url_http() {
        let result = parse("Check http://example.com out");
        assert!(result.contains("href=\"http://example.com\""));
    }

    #[test]
    fn url_javascript_blocked() {
        let result = parse("[url=javascript:alert('xss')]Click[/url]");
        assert!(!result.contains("href=\"javascript"));
    }

    #[test]
    fn url_data_blocked() {
        let result = parse("[url=data:text/html,<script>alert('xss')</script>]Click[/url]");
        assert!(!result.contains("href=\"data:"));
    }

    #[test]
    fn url_vbscript_blocked() {
        let result = parse("[url=vbscript:msgbox('xss')]Click[/url]");
        assert!(!result.contains("href=\"vbscript"));
    }

    #[test]
    fn url_nofollow() {
        let result = parse("[url=https://example.com]Link[/url]");
        assert!(result.contains("rel=\"nofollow\""));
    }

    #[test]
    fn nested_url_forbidden() {
        let result = parse("[url=http://a.com][url=http://b.com]Inner[/url][/url]");
        // Inner URL should be rejected
        // The outer should work
        assert!(result.contains("href=\"http://a.com\""));
    }
}

// ============================================================================
// Image Tests
// ============================================================================

mod images {
    use super::*;

    #[test]
    fn img_basic() {
        let result = parse("[img]https://example.com/image.png[/img]");
        assert!(result.contains("<img"));
        assert!(result.contains("src=\"https://example.com/image.png\""));
    }

    #[test]
    fn img_with_dimensions() {
        let result = parse("[img=100x200]https://example.com/image.png[/img]");
        assert!(result.contains("width=\"100\""));
        assert!(result.contains("height=\"200\""));
    }

    #[test]
    fn img_http() {
        let result = parse("[img]http://example.com/image.png[/img]");
        assert!(result.contains("<img"));
    }

    #[test]
    fn img_javascript_blocked() {
        let result = parse("[img]javascript:alert('xss')[/img]");
        assert!(!result.contains("<img"));
    }

    #[test]
    fn img_data_blocked() {
        let result = parse("[img]data:image/png;base64,xxx[/img]");
        assert!(!result.contains("<img"));
    }

    #[test]
    fn img_empty() {
        let result = parse("[img][/img]");
        assert!(!result.contains("<img"));
    }
}

// ============================================================================
// Code Block Tests
// ============================================================================

mod code_blocks {
    use super::*;

    #[test]
    fn code_preserves_bbcode() {
        let result = parse("[code][b]Bold[/b][/code]");
        assert!(result.contains("[b]Bold[/b]"));
        assert!(!result.contains("<strong>"));
    }

    #[test]
    fn code_preserves_html() {
        let result = parse("[code]<div>HTML</div>[/code]");
        assert!(result.contains("&lt;div&gt;HTML&lt;/div&gt;"));
    }

    #[test]
    fn code_preserves_whitespace() {
        let result = parse("[code]  indented\n    more[/code]");
        assert!(result.contains("  indented\n    more"));
    }

    #[test]
    fn icode_inline() {
        let result = parse("Use [icode]foo[/icode] here");
        assert!(result.contains("<code"));
        assert!(result.contains("foo"));
    }

    #[test]
    fn php_code() {
        let result = parse("[php]<?php echo 'test'; ?>[/php]");
        assert!(result.contains("language-php"));
    }

    #[test]
    fn html_code() {
        let result = parse("[html]<div>Test</div>[/html]");
        assert!(result.contains("language-html"));
    }

    #[test]
    fn code_case_insensitive_close() {
        let result = parse("[code]test[/CODE]");
        assert!(result.contains("test"));
        assert!(result.contains("<pre"));
    }
}

// ============================================================================
// Unicode Tests
// ============================================================================

mod unicode {
    use super::*;

    #[test]
    fn japanese() {
        let result = parse("[b]私は猫です[/b]");
        assert!(result.contains("<strong>私は猫です</strong>"));
    }

    #[test]
    fn chinese() {
        let result = parse("[i]你好世界[/i]");
        assert!(result.contains("<em>你好世界</em>"));
    }

    #[test]
    fn russian() {
        let result = parse("[b]Привет мир[/b]");
        assert!(result.contains("<strong>Привет мир</strong>"));
    }

    #[test]
    fn arabic() {
        let result = parse("[b]مرحبا بالعالم[/b]");
        assert!(result.contains("<strong>مرحبا بالعالم</strong>"));
    }

    #[test]
    fn emoji() {
        let result = parse("[b]🔥🎉🚀[/b]");
        assert!(result.contains("<strong>🔥🎉🚀</strong>"));
    }

    #[test]
    fn mixed_scripts() {
        let result = parse("English 日本語 Русский 🎉");
        assert!(result.contains("English 日本語 Русский 🎉"));
    }

    #[test]
    fn unicode_in_url() {
        let result = parse("[url=https://example.com/путь]Link[/url]");
        assert!(result.contains("href=\"https://example.com/путь\""));
    }

    #[test]
    fn unicode_in_quote_author() {
        let result = parse("[quote=\"日本人\"]Quote[/quote]");
        assert!(result.contains("日本人"));
    }
}

// ============================================================================
// Security Tests - Comprehensive HTML Injection Test Suite
// ============================================================================

mod security {
    use super::*;

    /// Helper to check if output contains a dangerous HTML event handler attribute.
    /// Returns true if the output contains an actual HTML attribute like ` onclick="`
    /// but NOT when it's just escaped text like `&quot;onclick=` or inside [brackets].
    fn has_dangerous_event_handler(output: &str, handler: &str) -> bool {
        let pattern = format!(" {}=", handler);
        if !output.contains(&pattern) {
            return false;
        }
        // Check if it's actually in an HTML attribute context
        // Look for patterns like: <tag ... onclick="..." or attribute="..." onclick="
        // vs safe patterns like: [tag ... onclick=...] or &quot;onclick=

        // If the pattern appears after a < and before a >, it's dangerous
        for (i, _) in output.match_indices(&pattern) {
            let before = &output[..i];
            let after = &output[i..];

            // Check if we're inside an HTML tag (after < and before >)
            let last_open = before.rfind('<');
            let last_close = before.rfind('>');

            // If last < is after last > (or no >), we might be inside a tag
            if let Some(open_pos) = last_open {
                if last_close.map_or(true, |close_pos| open_pos > close_pos) {
                    // We're after a < without a closing >
                    // Check if there's a > coming after the handler
                    if after.contains('>') {
                        return true; // Dangerous: inside an HTML tag
                    }
                }
            }
        }
        false
    }

    /// Check if output contains dangerous CSS patterns in actual HTML style attributes.
    /// Returns false if CSS patterns only appear in text/BBCode context.
    fn has_dangerous_css(output: &str) -> bool {
        let lower = output.to_lowercase();

        // Check for style=" pattern (HTML style attribute)
        // We need to find style= inside an HTML tag context
        for (i, _) in lower.match_indices("style=") {
            let before = &output[..i];
            let after = &output[i..];

            // Check if we're inside an HTML tag (after < and before >)
            let last_open = before.rfind('<');
            let last_close = before.rfind('>');

            // If last < is after last > (or no >), we're inside a tag
            if let Some(open_pos) = last_open {
                if last_close.map_or(true, |close_pos| open_pos > close_pos) {
                    // We're inside an HTML tag - check for dangerous CSS
                    let style_content = after;
                    if style_content.contains("expression(")
                        || style_content.contains("javascript:")
                        || style_content.contains("behavior:")
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    // ========================================================================
    // SECTION 1: PROTOCOL HANDLER BYPASSES
    // Goal: Execute JavaScript via the href or src attribute.
    // ========================================================================

    mod protocol_handler_bypasses {
        use super::*;

        // --- Basic Protocol Injection ---

        #[test]
        fn javascript_protocol_in_url() {
            let result = parse("[url=javascript:alert(1)]Click Me[/url]");
            assert!(
                !result.contains("href=\"javascript:"),
                "Must not contain javascript: href"
            );
            assert!(
                !result.contains("href='javascript:"),
                "Must not contain javascript: href (single quote)"
            );
        }

        #[test]
        fn javascript_protocol_in_img() {
            let result = parse("[img]javascript:alert(1)[/img]");
            assert!(
                !result.contains("src=\"javascript:"),
                "Must not contain javascript: src"
            );
            assert!(!result.contains("<img"), "Should not render img tag at all");
        }

        // --- Case Sensitivity Bypasses ---

        #[test]
        fn javascript_mixed_case() {
            let result = parse("[url=JaVaScRiPt:alert(1)]Click Me[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Case-insensitive check failed"
            );
            assert!(
                !result.to_lowercase().contains("href='javascript:"),
                "Case-insensitive check failed"
            );
        }

        #[test]
        fn javascript_uppercase() {
            let result = parse("[url=JAVASCRIPT:alert(1)]Click Me[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Uppercase bypass"
            );
        }

        #[test]
        fn vbscript_mixed_case() {
            let result = parse("[url=VbScRiPt:msgbox(1)]Click Me[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"vbscript:"),
                "VBScript case-insensitive check"
            );
        }

        // --- Whitespace/Control Character Obfuscation ---

        #[test]
        fn javascript_with_space() {
            // Note: "java script" with a space is not the same as "javascript"
            let result = parse("[url=java script:alert(1)]Click Me[/url]");
            // Even though this isn't valid javascript:, verify no href is produced
            // or if it is, it's not executable
            assert!(!result.to_lowercase().contains("href=\"javascript:"));
        }

        #[test]
        fn javascript_with_tab() {
            let result = parse("[url=java\tscript:alert(1)]Click Me[/url]");
            // Tab character should not allow bypass
            assert!(!result.to_lowercase().contains("href=\"javascript:"));
            // Check the tab doesn't get collapsed
            assert!(!result.contains("href=\"javascript:"));
        }

        #[test]
        fn javascript_with_newline() {
            let result = parse("[url=java\nscript:alert(1)]Click Me[/url]");
            assert!(!result.to_lowercase().contains("href=\"javascript:"));
        }

        #[test]
        fn javascript_with_html_entity_null() {
            let result = parse("[url=javascript&#00;:alert(1)]Click Me[/url]");
            assert!(!result.to_lowercase().contains("href=\"javascript:"));
            // Also check that the entity itself doesn't execute
            assert!(
                !result.contains("javascript&#00;:"),
                "HTML entities in scheme"
            );
        }

        #[test]
        fn javascript_with_html_entity_colon() {
            // &#58; is the colon character
            let result = parse("[url=javascript&#58;alert(1)]Click Me[/url]");
            // The scheme detection should still work
            assert!(!result.to_lowercase().contains("href=\"javascript:"));
        }

        // --- Dangerous Protocols (Legacy & Modern) ---

        #[test]
        fn vbscript_protocol() {
            let result = parse("[url=vbscript:msgbox(1)]Click Me[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"vbscript:"),
                "VBScript blocked"
            );
        }

        #[test]
        fn data_protocol_base64_script() {
            let result = parse(
                "[url=data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==]Click Me[/url]",
            );
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "data: URL blocked"
            );
        }

        #[test]
        fn data_protocol_in_img() {
            let result =
                parse("[img]data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==[/img]");
            assert!(
                !result.to_lowercase().contains("src=\"data:"),
                "data: URL blocked in img"
            );
        }

        #[test]
        fn livescript_protocol() {
            // Legacy Netscape protocol
            let result = parse("[url=livescript:alert(1)]Click Me[/url]");
            // Should either be blocked or not rendered
            assert!(
                !result.contains("<a") || !result.to_lowercase().contains("href=\"livescript:")
            );
        }

        #[test]
        fn mocha_protocol() {
            // Legacy Netscape protocol
            let result = parse("[url=mocha:alert(1)]Click Me[/url]");
            assert!(!result.contains("<a") || !result.to_lowercase().contains("href=\"mocha:"));
        }
    }

    // ========================================================================
    // SECTION 2: ATTRIBUTE ESCAPING & BREAKOUTS
    // Goal: Break out of the HTML attribute to inject new attributes.
    // ========================================================================

    mod attribute_breakouts {
        use super::*;

        // --- Double Quote Breakout ---

        #[test]
        fn double_quote_onclick_breakout() {
            let result = parse(r#"[url=" onclick="alert(1)"]Click Me[/url]"#);
            // Should either reject the tag (render as text) or escape the dangerous content
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "onclick injection blocked. Output: {}",
                result
            );
        }

        #[test]
        fn double_quote_onmouseover_breakout() {
            let result = parse(r#"[url=" onmouseover="alert(1)]Click Me[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onmouseover"),
                "onmouseover injection blocked. Output: {}",
                result
            );
        }

        // --- Single Quote Breakout ---

        #[test]
        fn single_quote_onclick_breakout() {
            let result = parse("[url=' onclick='alert(1)']Click Me[/url]");
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "onclick via single quote blocked. Output: {}",
                result
            );
        }

        // --- No-Quote Injection ---

        #[test]
        fn space_onclick_injection() {
            let result = parse("[url=http://google.com onclick=alert(1)]Click Me[/url]");
            // The onclick should either:
            // 1. Be in raw BBCode text (safe - not interpreted as HTML)
            // 2. Not appear at all
            // It should NEVER appear as an HTML attribute like: <a onclick=
            assert!(
                !result.contains("<a ") || !result.contains(" onclick="),
                "onclick injection via space must not appear in HTML anchor tag"
            );
        }

        // --- Attribute Confusion ---

        #[test]
        fn multiple_equals_confusion() {
            let result = parse("[url=foo=bar onclick=alert(1)]Click Me[/url]");
            assert!(
                !result.contains(" onclick="),
                "onclick injection via multiple = blocked"
            );
        }

        #[test]
        fn query_string_breakout() {
            let result =
                parse(r#"[url=http://site.com?q=123" onmouseover="alert(1)]Click Me[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onmouseover"),
                "onmouseover injection via query string blocked. Output: {}",
                result
            );
        }

        // --- Tag Closure Breakout ---

        #[test]
        fn close_tag_script_injection() {
            let result = parse(r#"[url="></a><script>alert(1)</script><a href="]Click Me[/url]"#);
            assert!(!result.contains("<script>"), "script tag injection blocked");
            assert!(
                !result.contains("</a><script>"),
                "tag closure breakout blocked"
            );
        }

        #[test]
        fn adjacent_script_injection() {
            let result = parse("[url=http://site.com]Link[/url] <script>alert(1)</script> [url=http://example.com]Link2[/url]");
            assert!(
                !result.contains("<script>"),
                "Adjacent script tag must be escaped"
            );
            assert!(
                result.contains("&lt;script&gt;"),
                "Script tag should be HTML escaped"
            );
        }

        #[test]
        fn empty_url_injection() {
            let result = parse("[url=]<script>alert(1)</script>[/url]");
            assert!(!result.contains("<script>"), "Script in content blocked");
        }
    }

    // ========================================================================
    // SECTION 3: CSS & STYLE INJECTION
    // Goal: Execute JS via CSS properties or load external resources.
    // ========================================================================

    mod css_injection {
        use super::*;

        // --- Expression Vector (IE Legacy) ---

        #[test]
        fn css_expression_injection() {
            // [style] is not a supported tag, so it should be rendered as text
            let result = parse("[style=width:expression(alert(1))]Text[/style]");
            // Should not have expression in a style= attribute context
            assert!(
                !has_dangerous_css(&result),
                "CSS expression blocked. Output: {}",
                result
            );
        }

        #[test]
        fn size_expression_injection() {
            let result = parse("[size=10;width:expression(alert(1))]Text[/size]");
            // Size should reject invalid values - rendered as text is safe
            assert!(
                !has_dangerous_css(&result),
                "CSS expression in size blocked. Output: {}",
                result
            );
        }

        // --- Background Image Vector ---

        #[test]
        fn style_background_javascript() {
            let result = parse("[style=background-image:url(javascript:alert(1))]Text[/style]");
            // [style] is not a supported tag, rendered as text is safe
            assert!(
                !has_dangerous_css(&result),
                "javascript: in background blocked. Output: {}",
                result
            );
        }

        #[test]
        fn quote_style_background_javascript() {
            let result =
                parse(r#"[quote style="background:url('javascript:alert(1')"]Text[/quote]"#);
            // Quote tag should not allow style attribute injection
            // The style= should not become an actual HTML attribute
            assert!(
                !has_dangerous_css(&result),
                "javascript: in quote style blocked. Output: {}",
                result
            );
        }

        // --- Behavior Vector (IE Legacy) ---

        #[test]
        fn style_behavior_injection() {
            let result = parse("[style=behavior:url(http://site.com/xss.htc)]Text[/style]");
            // [style] is not a supported tag, rendered as text is safe
            assert!(
                !has_dangerous_css(&result),
                "CSS behavior blocked. Output: {}",
                result
            );
        }

        // --- Breaking out of Style Attribute ---

        #[test]
        fn color_with_semicolon() {
            let result = parse("[color=red;]Text[/color]");
            // Should either reject or sanitize the semicolon
            // Valid scenario: reject entirely, or only use "red"
            if result.contains("color:") {
                // If it rendered, check no injection
                assert!(!result.contains("color: red;]") && !result.contains("red;]"));
            }
        }

        #[test]
        fn color_with_onmouseover() {
            let result = parse(r#"[color=red" onmouseover="alert(1)]Text[/color]"#);
            // Invalid color should be rejected, rendered as text is safe
            assert!(
                !has_dangerous_event_handler(&result, "onmouseover"),
                "onmouseover in color blocked. Output: {}",
                result
            );
        }

        #[test]
        fn color_with_semicolon_event() {
            let result = parse("[color=red;onmouseover=alert(1)]Text[/color]");
            // Invalid color value should be rejected, rendered as text is safe
            assert!(
                !has_dangerous_event_handler(&result, "onmouseover"),
                "onmouseover via semicolon blocked. Output: {}",
                result
            );
        }

        #[test]
        fn color_expression_injection() {
            let result = parse("[color=expression(alert(1))]Text[/color]");
            // Invalid color, rendered as text is safe
            assert!(
                !has_dangerous_css(&result),
                "expression in color blocked. Output: {}",
                result
            );
        }

        #[test]
        fn color_url_injection() {
            let result = parse("[color=red;background:url(javascript:alert(1))]Text[/color]");
            // Invalid color value, rendered as text is safe
            assert!(
                !has_dangerous_css(&result),
                "javascript: in color blocked. Output: {}",
                result
            );
        }
    }

    // ========================================================================
    // SECTION 4: IMAGE TAG VECTORS
    // Goal: Use the image source or error handlers to execute code.
    // ========================================================================

    mod image_vectors {
        use super::*;

        // --- OnError Handler ---

        #[test]
        fn img_onerror_injection() {
            let result = parse(r#"[img]http://site.com/nonexistent.jpg" onerror="alert(1)[/img]"#);
            // URL with quote should be rejected, rendered as text is safe
            assert!(
                !has_dangerous_event_handler(&result, "onerror"),
                "onerror injection blocked. Output: {}",
                result
            );
        }

        #[test]
        fn img_onmouseover_injection() {
            let result = parse(r#"[img]http://site.com/image.jpg" onmouseover="alert(1)[/img]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onmouseover"),
                "onmouseover injection blocked. Output: {}",
                result
            );
        }

        #[test]
        fn img_onload_injection() {
            let result = parse(r#"[img]http://site.com/image.jpg" onload="alert(1)[/img]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onload"),
                "onload injection blocked. Output: {}",
                result
            );
        }

        // --- Dynamic Source ---

        #[test]
        fn img_dynsrc_injection() {
            let result = parse("[img]dynsrc=javascript:alert(1)[/img]");
            // Invalid URL scheme, should be rendered as text or rejected
            // Not dangerous if not in src= attribute
            assert!(
                !result.contains("src=\"javascript:") && !result.contains("src=\"dynsrc"),
                "dynsrc javascript blocked. Output: {}",
                result
            );
        }

        #[test]
        fn img_lowsrc_injection() {
            let result = parse("[img]lowsrc=javascript:alert(1)[/img]");
            // Invalid URL, should be rendered as text or rejected
            assert!(
                !result.contains("src=\"javascript:") && !result.contains("src=\"lowsrc"),
                "lowsrc javascript blocked. Output: {}",
                result
            );
        }

        // --- Additional Image Vectors ---

        #[test]
        fn img_with_html_breakout() {
            let result =
                parse(r#"[img]http://x.com/x.jpg"><script>alert(1)</script><img src="[/img]"#);
            assert!(
                !result.contains("<script>"),
                "script injection via img blocked"
            );
        }

        #[test]
        fn img_svg_with_script() {
            let result = parse("[img]http://evil.com/image.svg#<script>alert(1)</script>[/img]");
            assert!(
                !result.contains("<script>"),
                "script in SVG fragment blocked"
            );
        }
    }

    // ========================================================================
    // SECTION 5: NESTING & PARSER LOGIC ERRORS
    // Goal: Confuse the parser into producing broken HTML.
    // ========================================================================

    mod nesting_logic_errors {
        use super::*;

        // --- Split/Interleaved Tags ---

        #[test]
        fn interleaved_url_bold() {
            let result = parse("[url=http://example.com][b]Link[/url][/b]");
            // Parser should handle gracefully - not produce broken HTML
            // The key is that whatever is produced is valid/safe
            // Check for balanced tags or text rendering
            let a_opens = result.matches("<a ").count();
            let a_closes = result.matches("</a>").count();
            assert!(
                a_opens == a_closes,
                "Should have balanced anchor tags. Output: {}",
                result
            );
        }

        #[test]
        fn interleaved_with_quote_breakout() {
            let result = parse(r#"[url="][b]Link[/url][/b]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "No event handlers"
            );
            assert!(!result.contains("<script>"), "No script injection");
        }

        // --- Nested Attributes ---

        #[test]
        fn nested_url_in_url_option() {
            let result = parse("[url=[url=javascript:alert(1)]Link[/url]]Link[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Nested javascript: blocked"
            );
        }

        #[test]
        fn bbcode_in_url_option() {
            let result = parse("[url=[b]bold[/b]]Link[/url]");
            // Should not produce broken HTML
            assert!(!result.contains("[b]</a>"), "Malformed HTML avoided");
        }

        // --- Recursive DOS Check ---

        #[test]
        fn deeply_nested_quotes() {
            let nested = "[quote]".repeat(100) + "Content" + &"[/quote]".repeat(100);
            let result = parse(&nested);
            // Should not crash and should produce reasonable output
            // Either limits nesting or handles it
            assert!(!result.is_empty(), "Should handle deep nesting");
        }

        #[test]
        fn deeply_nested_formatting() {
            let nested = "[b][i][u]".repeat(50) + "Text" + &"[/u][/i][/b]".repeat(50);
            let result = parse(&nested);
            assert!(!result.is_empty(), "Should handle deep formatting nesting");
        }

        // --- Unclosed Tags causing "Tag Soup" ---

        #[test]
        fn unclosed_url_with_newline() {
            let result = parse("[url=http://site.com\n[b]\n[i]");
            // Should not produce an open <a> tag without closing
            // Count opens vs closes
            let a_opens = result.matches("<a ").count();
            let a_closes = result.matches("</a>").count();
            assert!(
                a_opens == a_closes || result.contains("[url="),
                "Unclosed tags handled correctly"
            );
        }

        #[test]
        fn unclosed_tags_soup() {
            let result = parse("[b][i][u]text without closing");
            // Parser should handle this gracefully
            assert!(result.contains("text without closing"));
        }

        #[test]
        fn mismatched_closing_order() {
            let result = parse("[b][i]text[/b][/i]");
            // Should handle mismatched closing tags
            assert!(result.contains("text"));
        }
    }

    // ========================================================================
    // SECTION 6: ENCODING & OBFUSCATION
    // Goal: Bypass filters using various encodings.
    // ========================================================================

    mod encoding_obfuscation {
        use super::*;

        // --- HTML Entity Encoding ---

        #[test]
        fn html_entity_javascript() {
            // &#106;&#97;&#118;&#97;&#115;&#99;&#114;&#105;&#112;&#116;&#58;&#97;&#108;&#101;&#114;&#116;&#40;&#49;&#41;
            // = javascript:alert(1)
            let result = parse("[url=&#106;&#97;&#118;&#97;&#115;&#99;&#114;&#105;&#112;&#116;&#58;&#97;&#108;&#101;&#114;&#116;&#40;&#49;&#41;]Click Me[/url]");
            // The HTML entities should either be:
            // 1. Not decoded and thus not match javascript:
            // 2. Decoded and blocked
            assert!(
                !result.contains("href=\"javascript:"),
                "HTML entity bypass blocked"
            );
        }

        #[test]
        fn html_entity_hex_javascript() {
            // &#x6A;&#x61;&#x76;&#x61;&#x73;&#x63;&#x72;&#x69;&#x70;&#x74;&#x3A;
            // = javascript:
            let result = parse("[url=&#x6A;&#x61;&#x76;&#x61;&#x73;&#x63;&#x72;&#x69;&#x70;&#x74;&#x3A;alert(1)]Click Me[/url]");
            assert!(
                !result.contains("href=\"javascript:"),
                "Hex HTML entity bypass blocked"
            );
        }

        // --- URL Encoding ---

        #[test]
        fn url_encoded_javascript() {
            // %6A%61%76%61%73%63%72%69%70%74%3A%61%6C%65%72%74%28%31%29
            // = javascript:alert(1)
            let result = parse(
                "[url=%6A%61%76%61%73%63%72%69%70%74%3A%61%6C%65%72%74%28%31%29]Click Me[/url]",
            );
            // Should not decode and execute
            assert!(
                !result.contains("href=\"javascript:"),
                "URL encoded bypass blocked"
            );
        }

        #[test]
        fn double_url_encoded_javascript() {
            // Double encoding: %256A for 'j'
            let result = parse(
                "[url=%256A%2561%2576%2561%2573%2563%2572%2569%2570%2574%253A]Click Me[/url]",
            );
            assert!(
                !result.contains("href=\"javascript:"),
                "Double URL encoded bypass blocked"
            );
        }

        // --- Mixed Encoding ---

        #[test]
        fn mixed_encoding_tab() {
            // java&#09;script:alert(1) - HTML entity for tab
            let result = parse("[url=java&#09;script:alert(1)]Click Me[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Mixed encoding bypass blocked"
            );
        }

        #[test]
        fn mixed_encoding_newline() {
            // java&#10;script:alert(1) - HTML entity for newline
            let result = parse("[url=java&#10;script:alert(1)]Click Me[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Mixed encoding newline bypass blocked"
            );
        }

        // --- Unicode Obfuscation ---

        #[test]
        fn unicode_javascript() {
            // Using unicode escape sequences
            let result = parse("[url=\u{006A}avascript:alert(1)]Click Me[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Unicode bypass blocked"
            );
        }

        #[test]
        fn unicode_homograph_attack() {
            // Using lookalike characters (Cyrillic 'а' instead of Latin 'a')
            let result = parse("[url=jаvаscript:alert(1)]Click Me[/url]");
            // This should either not match "javascript" pattern or be rejected
            // The key is it shouldn't execute as JavaScript
            if result.contains("<a") {
                // If it rendered as a link, ensure it's not dangerous
                assert!(!result.to_lowercase().contains("href=\"javascript:"));
            }
        }
    }

    // ========================================================================
    // SECTION 7: POLYGLOTS
    // Goal: Complex strings designed to break multiple contexts.
    // ========================================================================

    mod polyglots {
        use super::*;

        #[test]
        fn javascript_url_with_img() {
            let result = parse("[url=javascript:alert(1)][img]http://site.com[/img][/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "JavaScript URL with img blocked"
            );
        }

        #[test]
        fn url_content_onclick() {
            let result = parse(r#"[url]http://site.com" onclick="alert(1)[/url]"#);
            // URL with quote should be rejected, rendered as text is safe
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "onclick in URL content blocked. Output: {}",
                result
            );
        }

        #[test]
        fn img_onerror_cookie() {
            let result = parse(
                r#"[img]http://url.to.file.which/not.exist" onerror=alert(document.cookie);[/img]"#,
            );
            // URL with quote should be rejected
            assert!(
                !has_dangerous_event_handler(&result, "onerror"),
                "onerror cookie steal blocked. Output: {}",
                result
            );
        }

        #[test]
        fn email_onclick() {
            let result = parse(r#"[email]test@test.com" onclick="alert(1)[/email]"#);
            // Email with quote should be rejected
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "onclick in email blocked. Output: {}",
                result
            );
        }

        #[test]
        fn multi_context_polyglot() {
            let polyglot = r#"[url=javascript:alert(1)//"][img]x" onerror=alert(1)//[/img][/url]"#;
            let result = parse(polyglot);
            assert!(
                !result.contains("href=\"javascript:"),
                "JavaScript in polyglot blocked"
            );
            assert!(
                !has_dangerous_event_handler(&result, "onerror"),
                "onerror in polyglot blocked. Output: {}",
                result
            );
        }

        #[test]
        fn svg_polyglot() {
            let result = parse(r#"[img]x"><svg onload=alert(1)>[/img]"#);
            assert!(!result.contains("<svg"), "SVG injection blocked");
            assert!(
                !has_dangerous_event_handler(&result, "onload"),
                "onload injection blocked. Output: {}",
                result
            );
        }

        #[test]
        fn quote_with_script() {
            let result = parse(r#"[quote="</blockquote><script>alert(1)</script>"]text[/quote]"#);
            assert!(
                !result.contains("<script>"),
                "Script in quote attribution blocked"
            );
        }

        #[test]
        fn comprehensive_xss_polyglot() {
            // A comprehensive XSS polyglot
            let polyglot = r#"jaVasCript:/*-/*`/*\`/*'/*"/**/(/* */oNcLiCk=alert() )//%0D%0A%0d%0a//</stYle/</titLe/</teXtarEa/</scRipt/--!>\x3csVg/<sVg/oNloAd=alert()//>\x3e"#;
            let result = parse(&format!("[url={}]Click[/url]", polyglot));
            assert!(!result.contains(" onclick="), "Polyglot onclick blocked");
            assert!(!result.contains(" onload="), "Polyglot onload blocked");
            assert!(!result.contains("<svg"), "Polyglot SVG blocked");
        }
    }

    // ========================================================================
    // SECTION 8: ADDITIONAL EDGE CASES
    // ========================================================================

    mod additional_edge_cases {
        use super::*;

        // --- HTML Tag Injection ---

        #[test]
        fn raw_script_tag() {
            let result = parse("<script>alert('xss')</script>");
            assert!(!result.contains("<script>"), "Raw script tag escaped");
            assert!(result.contains("&lt;script&gt;"), "Script tag HTML escaped");
        }

        #[test]
        fn raw_img_onerror() {
            let result = parse(r#"<img src=x onerror=alert(1)>"#);
            assert!(!result.contains("<img"), "Raw img tag escaped");
            assert!(result.contains("&lt;img"), "Img tag HTML escaped");
        }

        #[test]
        fn raw_svg_onload() {
            let result = parse("<svg onload=alert(1)>");
            assert!(!result.contains("<svg"), "SVG tag escaped");
        }

        #[test]
        fn raw_body_onload() {
            let result = parse("<body onload=alert(1)>");
            assert!(!result.contains("<body"), "Body tag escaped");
        }

        #[test]
        fn raw_iframe() {
            let result = parse("<iframe src=\"javascript:alert(1)\"></iframe>");
            assert!(!result.contains("<iframe"), "Iframe escaped");
        }

        #[test]
        fn raw_object_tag() {
            let result = parse("<object data=\"javascript:alert(1)\"></object>");
            assert!(!result.contains("<object"), "Object tag escaped");
        }

        #[test]
        fn raw_embed_tag() {
            let result = parse("<embed src=\"javascript:alert(1)\">");
            assert!(!result.contains("<embed"), "Embed tag escaped");
        }

        // --- Event Handler Variations ---

        #[test]
        fn event_handler_case_variations() {
            let handlers = vec![
                "onclick",
                "ONCLICK",
                "OnClick",
                "oNcLiCk",
                "onmouseover",
                "ONMOUSEOVER",
                "OnMouseOver",
                "onerror",
                "ONERROR",
                "OnError",
                "onload",
                "ONLOAD",
                "OnLoad",
                "onfocus",
                "ONFOCUS",
                "OnFocus",
                "onblur",
                "ONBLUR",
                "OnBlur",
            ];

            for handler in handlers {
                let result = parse(&format!(
                    r#"[url=http://x.com" {}=alert(1)]Click[/url]"#,
                    handler
                ));
                // The handler should either:
                // 1. Be in raw BBCode text (safe - not interpreted as HTML)
                // 2. Not appear at all
                // It should NEVER appear as an actual HTML attribute
                let is_raw_bbcode = result.contains("[url=");
                let has_handler_in_anchor = result.contains("<a ")
                    && result
                        .to_lowercase()
                        .contains(&format!(" {}=", handler.to_lowercase()));
                assert!(
                    is_raw_bbcode || !has_handler_in_anchor,
                    "Event handler {} should be blocked: {}",
                    handler,
                    result
                );
            }
        }

        // --- Protocol Variations ---

        #[test]
        fn various_dangerous_protocols() {
            let protocols = vec![
                "javascript:",
                "vbscript:",
                "data:",
                "file:",
                "JAVASCRIPT:",
                "VBSCRIPT:",
                "DATA:",
                "FILE:",
                "JaVaScRiPt:",
                "VbScRiPt:",
                "DaTa:",
                "FiLe:",
            ];

            for proto in protocols {
                let result = parse(&format!("[url={}alert(1)]Click[/url]", proto));
                assert!(
                    !result.contains(&format!("href=\"{}", proto.to_lowercase())),
                    "Protocol {} should be blocked",
                    proto
                );
            }
        }

        // --- Null Byte Injection ---

        #[test]
        fn null_byte_in_url() {
            let result = parse("[url=javascript:\x00alert(1)]Click[/url]");
            assert!(
                !result.contains("href=\"javascript:"),
                "Null byte bypass blocked"
            );
        }

        #[test]
        fn null_byte_in_content() {
            let result = parse("[b]Hello\x00World[/b]");
            // Should handle gracefully
            assert!(result.contains("Hello") || result.contains("World"));
        }

        // --- Size/Font Injection Attempts ---

        #[test]
        fn size_with_event_handler() {
            let result = parse(r#"[size=12" onclick="alert(1)]Text[/size]"#);
            // Invalid size value should be rejected, rendered as text is safe
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "onclick in size blocked. Output: {}",
                result
            );
        }

        #[test]
        fn font_with_event_handler() {
            let result = parse(r#"[font=Arial" onclick="alert(1)]Text[/font]"#);
            // Invalid font value should be rejected, rendered as text is safe
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "onclick in font blocked. Output: {}",
                result
            );
        }

        // --- Quote Tag Abuse ---

        #[test]
        fn quote_author_injection() {
            let result = parse(r#"[quote="User" onclick="alert(1)"]Text[/quote]"#);
            // Quote author should be escaped
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "onclick in quote author blocked. Output: {}",
                result
            );
        }

        #[test]
        fn quote_author_html_injection() {
            let result = parse(r#"[quote="<script>alert(1)</script>"]Text[/quote]"#);
            assert!(
                !result.contains("<script>"),
                "Script in quote author blocked"
            );
        }

        // --- Code Tag Content ---

        #[test]
        fn code_tag_with_html() {
            let result = parse("[code]<script>alert(1)</script>[/code]");
            // Code content should be escaped
            assert!(
                !result.contains("<script>"),
                "HTML in code should be escaped"
            );
            assert!(
                result.contains("&lt;script&gt;") || result.contains("&lt;script"),
                "Script tag should be HTML escaped in code"
            );
        }

        #[test]
        fn code_tag_preserves_content() {
            let result = parse("[code]function test() { alert('hello'); }[/code]");
            assert!(result.contains("function test()"), "Code content preserved");
        }
    }

    // ========================================================================
    // SECTION 9: CONTROL CHARACTER BYPASSES
    // Goal: Use control characters to bypass protocol/handler detection.
    // Based on dcwatson/bbcode issue #16 and OWASP Filter Evasion Cheat Sheet.
    // ========================================================================

    mod control_character_bypasses {
        use super::*;

        #[test]
        fn null_before_javascript() {
            // \x00 before javascript:
            let result = parse("[url=\x00javascript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Null byte before javascript blocked"
            );
        }

        #[test]
        fn soh_before_javascript() {
            // \x01 (Start of Heading) before javascript: - dcwatson/bbcode vulnerability
            let result = parse("[url=\x01javascript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "SOH before javascript blocked"
            );
        }

        #[test]
        fn vertical_tab_in_protocol() {
            // \x0B (vertical tab) in protocol
            let result = parse("[url=java\x0Bscript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Vertical tab bypass blocked"
            );
        }

        #[test]
        fn form_feed_in_protocol() {
            // \x0C (form feed) in protocol
            let result = parse("[url=java\x0Cscript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Form feed bypass blocked"
            );
        }

        #[test]
        fn carriage_return_in_protocol() {
            // \x0D (carriage return) in protocol
            let result = parse("[url=java\x0Dscript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Carriage return bypass blocked"
            );
        }

        #[test]
        fn bell_character_before_javascript() {
            // \x07 (bell) before javascript:
            let result = parse("[url=\x07javascript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Bell character bypass blocked"
            );
        }

        #[test]
        fn backspace_in_protocol() {
            // \x08 (backspace) in protocol
            let result = parse("[url=java\x08script:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Backspace bypass blocked"
            );
        }

        #[test]
        fn multiple_control_chars() {
            // Multiple control characters combined
            let result = parse("[url=\x01\x02\x03javascript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Multiple control chars bypass blocked"
            );
        }

        #[test]
        fn control_char_after_colon() {
            // Control char after the colon
            let result = parse("[url=javascript:\x00alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Control char after colon blocked"
            );
        }

        #[test]
        fn img_control_char_bypass() {
            let result = parse("[img]\x01javascript:alert(1)[/img]");
            assert!(
                !result.contains("<img"),
                "Control char in img src blocked"
            );
        }
    }

    // ========================================================================
    // SECTION 10: ADDITIONAL EVENT HANDLERS (HTML5 & Legacy)
    // Goal: Test comprehensive list of event handlers.
    // ========================================================================

    mod additional_event_handlers {
        use super::*;

        // HTML5 Event Handlers
        #[test]
        fn onanimationstart_injection() {
            let result = parse(r#"[url=http://x.com" onanimationstart="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onanimationstart"),
                "onanimationstart blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onanimationend_injection() {
            let result = parse(r#"[url=http://x.com" onanimationend="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onanimationend"),
                "onanimationend blocked. Output: {}",
                result
            );
        }

        #[test]
        fn ontransitionend_injection() {
            let result = parse(r#"[url=http://x.com" ontransitionend="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "ontransitionend"),
                "ontransitionend blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onwheel_injection() {
            let result = parse(r#"[url=http://x.com" onwheel="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onwheel"),
                "onwheel blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onscroll_injection() {
            let result = parse(r#"[url=http://x.com" onscroll="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onscroll"),
                "onscroll blocked. Output: {}",
                result
            );
        }

        #[test]
        fn oncopy_injection() {
            let result = parse(r#"[url=http://x.com" oncopy="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "oncopy"),
                "oncopy blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onpaste_injection() {
            let result = parse(r#"[url=http://x.com" onpaste="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onpaste"),
                "onpaste blocked. Output: {}",
                result
            );
        }

        #[test]
        fn oncut_injection() {
            let result = parse(r#"[url=http://x.com" oncut="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "oncut"),
                "oncut blocked. Output: {}",
                result
            );
        }

        #[test]
        fn ondrag_injection() {
            let result = parse(r#"[url=http://x.com" ondrag="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "ondrag"),
                "ondrag blocked. Output: {}",
                result
            );
        }

        #[test]
        fn ondrop_injection() {
            let result = parse(r#"[url=http://x.com" ondrop="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "ondrop"),
                "ondrop blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onsearch_injection() {
            let result = parse(r#"[url=http://x.com" onsearch="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onsearch"),
                "onsearch blocked. Output: {}",
                result
            );
        }

        #[test]
        fn oncontextmenu_injection() {
            let result = parse(r#"[url=http://x.com" oncontextmenu="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "oncontextmenu"),
                "oncontextmenu blocked. Output: {}",
                result
            );
        }

        // Legacy event handlers for older browsers
        #[test]
        fn onstart_marquee_injection() {
            // onstart is used by <marquee> element
            let result = parse(r#"[url=http://x.com" onstart="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onstart"),
                "onstart blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onfinish_marquee_injection() {
            // onfinish is used by <marquee> element
            let result = parse(r#"[url=http://x.com" onfinish="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onfinish"),
                "onfinish blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onbounce_marquee_injection() {
            // onbounce is used by <marquee> element
            let result = parse(r#"[url=http://x.com" onbounce="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onbounce"),
                "onbounce blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onbeforeprint_injection() {
            let result = parse(r#"[url=http://x.com" onbeforeprint="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onbeforeprint"),
                "onbeforeprint blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onafterprint_injection() {
            let result = parse(r#"[url=http://x.com" onafterprint="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onafterprint"),
                "onafterprint blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onhashchange_injection() {
            let result = parse(r#"[url=http://x.com" onhashchange="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onhashchange"),
                "onhashchange blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onpopstate_injection() {
            let result = parse(r#"[url=http://x.com" onpopstate="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onpopstate"),
                "onpopstate blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onstorage_injection() {
            let result = parse(r#"[url=http://x.com" onstorage="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onstorage"),
                "onstorage blocked. Output: {}",
                result
            );
        }

        #[test]
        fn ontoggle_injection() {
            let result = parse(r#"[url=http://x.com" ontoggle="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "ontoggle"),
                "ontoggle blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onpointerdown_injection() {
            let result = parse(r#"[url=http://x.com" onpointerdown="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onpointerdown"),
                "onpointerdown blocked. Output: {}",
                result
            );
        }

        #[test]
        fn onpointerup_injection() {
            let result = parse(r#"[url=http://x.com" onpointerup="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onpointerup"),
                "onpointerup blocked. Output: {}",
                result
            );
        }

        #[test]
        fn ontouchstart_injection() {
            let result = parse(r#"[url=http://x.com" ontouchstart="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "ontouchstart"),
                "ontouchstart blocked. Output: {}",
                result
            );
        }

        #[test]
        fn ontouchend_injection() {
            let result = parse(r#"[url=http://x.com" ontouchend="alert(1)]Click[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "ontouchend"),
                "ontouchend blocked. Output: {}",
                result
            );
        }
    }

    // ========================================================================
    // SECTION 11: RAW HTML TAG INJECTION (EXTENDED)
    // Goal: Test more HTML tags that could be used for XSS.
    // ========================================================================

    mod extended_html_injection {
        use super::*;

        // Form-based attacks
        #[test]
        fn raw_form_tag() {
            let result = parse("<form action=\"javascript:alert(1)\"><input type=\"submit\"></form>");
            assert!(!result.contains("<form"), "Form tag escaped");
        }

        #[test]
        fn raw_input_tag() {
            let result = parse(r#"<input onfocus="alert(1)" autofocus>"#);
            assert!(!result.contains("<input"), "Input tag escaped");
        }

        #[test]
        fn raw_button_tag() {
            let result = parse(r#"<button onclick="alert(1)">Click</button>"#);
            assert!(!result.contains("<button"), "Button tag escaped");
        }

        #[test]
        fn raw_textarea_tag() {
            let result = parse(r#"<textarea onfocus="alert(1)">text</textarea>"#);
            assert!(!result.contains("<textarea"), "Textarea tag escaped");
        }

        #[test]
        fn raw_select_tag() {
            let result = parse(r#"<select onfocus="alert(1)"><option>x</option></select>"#);
            assert!(!result.contains("<select"), "Select tag escaped");
        }

        // Metadata tags
        #[test]
        fn raw_meta_refresh() {
            let result = parse(r#"<meta http-equiv="refresh" content="0;url=javascript:alert(1)">"#);
            assert!(!result.contains("<meta"), "Meta tag escaped");
        }

        #[test]
        fn raw_link_tag() {
            let result = parse(r#"<link rel="stylesheet" href="javascript:alert(1)">"#);
            assert!(!result.contains("<link"), "Link tag escaped");
        }

        #[test]
        fn raw_base_tag() {
            let result = parse(r#"<base href="javascript:alert(1)">"#);
            assert!(!result.contains("<base"), "Base tag escaped");
        }

        #[test]
        fn raw_style_tag() {
            let result = parse("<style>*{background:url('javascript:alert(1)')}</style>");
            assert!(!result.contains("<style"), "Style tag escaped");
        }

        // Media tags
        #[test]
        fn raw_video_tag() {
            let result = parse(r#"<video><source onerror="alert(1)"></video>"#);
            assert!(!result.contains("<video"), "Video tag escaped");
        }

        #[test]
        fn raw_audio_tag() {
            let result = parse(r#"<audio src="x" onerror="alert(1)">"#);
            assert!(!result.contains("<audio"), "Audio tag escaped");
        }

        #[test]
        fn raw_source_tag() {
            let result = parse(r#"<source onerror="alert(1)">"#);
            assert!(!result.contains("<source"), "Source tag escaped");
        }

        #[test]
        fn raw_track_tag() {
            let result = parse(r#"<track default src="x" oncuechange="alert(1)">"#);
            assert!(!result.contains("<track"), "Track tag escaped");
        }

        // Legacy/deprecated but still dangerous
        #[test]
        fn raw_marquee_tag() {
            let result = parse(r#"<marquee onstart="alert(1)">text</marquee>"#);
            assert!(!result.contains("<marquee"), "Marquee tag escaped");
        }

        #[test]
        fn raw_bgsound_tag() {
            // Legacy IE tag
            let result = parse(r#"<bgsound src="javascript:alert(1)">"#);
            assert!(!result.contains("<bgsound"), "Bgsound tag escaped");
        }

        #[test]
        fn raw_applet_tag() {
            let result = parse(r#"<applet code="javascript:alert(1)"></applet>"#);
            assert!(!result.contains("<applet"), "Applet tag escaped");
        }

        // Other dangerous tags
        #[test]
        fn raw_math_tag() {
            let result = parse(r#"<math><maction actiontype="statusline">text</maction></math>"#);
            assert!(!result.contains("<math"), "Math tag escaped");
        }

        #[test]
        fn raw_details_tag() {
            let result = parse(r#"<details open ontoggle="alert(1)">text</details>"#);
            assert!(!result.contains("<details") || !result.contains("ontoggle"), "Details/ontoggle escaped");
        }

        #[test]
        fn raw_dialog_tag() {
            let result = parse(r#"<dialog open onclose="alert(1)">text</dialog>"#);
            assert!(!result.contains("<dialog"), "Dialog tag escaped");
        }

        // XML/Namespace attacks
        #[test]
        fn svg_xlink_href() {
            let result = parse(r#"<svg><a xlink:href="javascript:alert(1)"><text>click</text></a></svg>"#);
            assert!(!result.contains("<svg"), "SVG with xlink:href escaped");
        }

        #[test]
        fn svg_animate() {
            let result = parse(r#"<svg><animate onbegin="alert(1)"></animate></svg>"#);
            assert!(!result.contains("<svg"), "SVG animate escaped");
        }

        #[test]
        fn svg_set() {
            let result = parse(r#"<svg><set onbegin="alert(1)"></set></svg>"#);
            assert!(!result.contains("<svg"), "SVG set escaped");
        }

        #[test]
        fn svg_foreignobject() {
            let result = parse(r#"<svg><foreignObject><iframe src="javascript:alert(1)"></iframe></foreignObject></svg>"#);
            assert!(!result.contains("<svg"), "SVG foreignObject escaped");
        }

        // Keygen (deprecated but some browsers support)
        #[test]
        fn raw_keygen_tag() {
            let result = parse(r#"<keygen autofocus onfocus="alert(1)">"#);
            assert!(!result.contains("<keygen"), "Keygen tag escaped");
        }
    }

    // ========================================================================
    // SECTION 12: DATA URI VARIATIONS
    // Goal: Test various data: URI MIME types and encodings.
    // ========================================================================

    mod data_uri_variations {
        use super::*;

        #[test]
        fn data_text_html() {
            let result = parse("[url=data:text/html,<script>alert(1)</script>]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "data:text/html blocked"
            );
        }

        #[test]
        fn data_text_html_base64() {
            // PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg== = <script>alert(1)</script>
            let result = parse("[url=data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "data:text/html;base64 blocked"
            );
        }

        #[test]
        fn data_application_xhtml() {
            let result = parse("[url=data:application/xhtml+xml,<script>alert(1)</script>]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "data:application/xhtml+xml blocked"
            );
        }

        #[test]
        fn data_image_svg() {
            // SVG with embedded script
            let result = parse("[url=data:image/svg+xml,<svg onload='alert(1)'>]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "data:image/svg+xml blocked"
            );
        }

        #[test]
        fn data_image_svg_base64() {
            // Base64 encoded SVG with onload
            let result = parse("[url=data:image/svg+xml;base64,PHN2ZyBvbmxvYWQ9J2FsZXJ0KDEpJz4=]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "data:image/svg+xml;base64 blocked"
            );
        }

        #[test]
        fn data_text_css() {
            let result = parse("[url=data:text/css,.x{background:url(javascript:alert(1))}]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "data:text/css blocked"
            );
        }

        #[test]
        fn data_charset_param() {
            let result = parse("[url=data:text/html;charset=utf-8,<script>alert(1)</script>]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "data: with charset blocked"
            );
        }

        #[test]
        fn data_mixed_case() {
            let result = parse("[url=DaTa:text/html,<script>alert(1)</script>]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "DaTa: case insensitive blocked"
            );
        }

        #[test]
        fn data_in_img() {
            let result = parse("[img]data:image/svg+xml,<svg onload='alert(1)'>[/img]");
            assert!(
                !result.contains("<img"),
                "data: in img src blocked"
            );
        }

        #[test]
        fn data_with_whitespace() {
            let result = parse("[url=data:  text/html,<script>alert(1)</script>]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"data:"),
                "data: with whitespace blocked"
            );
        }
    }

    // ========================================================================
    // SECTION 13: HTML ENTITY ENCODING VARIATIONS
    // Goal: Test various HTML entity formats and edge cases.
    // ========================================================================

    mod html_entity_variations {
        use super::*;

        // Entities without semicolons (some browsers accept these)
        #[test]
        fn javascript_entities_no_semicolon() {
            // &#106 without semicolon for 'j'
            let result = parse("[url=&#106avascript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Entity without semicolon blocked"
            );
        }

        #[test]
        fn hex_entities_no_semicolon() {
            // &#x6A without semicolon for 'j'
            let result = parse("[url=&#x6Aavascript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Hex entity without semicolon blocked"
            );
        }

        // Long numeric entities
        #[test]
        fn padded_numeric_entities() {
            // &#0000106; = 'j' with padding zeros
            let result = parse("[url=&#0000106;avascript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Padded numeric entity blocked"
            );
        }

        #[test]
        fn padded_hex_entities() {
            // &#x00006A; = 'j' with padding zeros
            let result = parse("[url=&#x00006A;avascript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Padded hex entity blocked"
            );
        }

        // Named entities for special chars
        #[test]
        fn named_entity_lt_gt() {
            let result = parse("[url=&lt;script&gt;]Click[/url]");
            // Should not produce executable content
            assert!(!result.contains("<script>"), "Named entities don't produce HTML");
        }

        #[test]
        fn mixed_entity_styles() {
            // Mix of decimal and hex entities
            let result = parse("[url=&#106;&#x61;&#118;&#x61;script:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Mixed entity styles blocked"
            );
        }

        #[test]
        fn uppercase_hex_entities() {
            // &#X6A instead of &#x6a
            let result = parse("[url=&#X6Aavascript:alert(1)]Click[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Uppercase X in hex entity blocked"
            );
        }

        // Entity in event handler name
        #[test]
        fn entity_in_handler_name() {
            let result = parse(r#"[url=http://x.com" &#x6F;nclick="alert(1)]Click[/url]"#);
            // &#x6F; = 'o', so this tries to spell "onclick"
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "Entity in handler name blocked. Output: {}",
                result
            );
        }

        #[test]
        fn double_encoding() {
            // &amp;#106; = &#106; when decoded once
            let result = parse("[url=&amp;#106;avascript:alert(1)]Click[/url]");
            // After one decode: &#106;avascript which could become javascript
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Double encoded entity blocked"
            );
        }
    }

    // ========================================================================
    // SECTION 14: phpBB & FORUM SOFTWARE SPECIFIC EXPLOITS
    // Based on exploit-db.com findings and CVEs.
    // ========================================================================

    mod forum_specific_exploits {
        use super::*;

        // phpBB 2.0.6 - CVE-2004-1315 style attack
        #[test]
        fn phpbb_quote_onclick_breakout() {
            let result = parse(r#"[url=http://www.example.com" onclick="alert('xss')]text[/url]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "phpBB quote breakout blocked. Output: {}",
                result
            );
        }

        // JForum 2.08 - color tag style injection
        #[test]
        fn jforum_color_style_injection() {
            let result = parse(r#"[color=red' style='font-size:50px' /onMouseOver='alert(document.cookie)']test[/color]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onmouseover"),
                "JForum color injection blocked. Output: {}",
                result
            );
            assert!(
                !result.contains("style='font-size:50px'"),
                "Injected style blocked. Output: {}",
                result
            );
        }

        // webSPELL - img onerror
        #[test]
        fn webspell_img_onerror() {
            let result = parse(r#"[img]http://x.jpg" onerror="alert(1)[/img]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onerror"),
                "webSPELL img onerror blocked. Output: {}",
                result
            );
        }

        // SMF (Simple Machines Forum) style
        #[test]
        fn smf_url_breakout() {
            let result = parse(r#"[url=javascript:alert(String.fromCharCode(88,83,83))]XSS[/url]"#);
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "SMF javascript URL blocked"
            );
        }

        // PHP-Fusion style
        #[test]
        fn phpfusion_nested_tags() {
            let result = parse("[url=[img]javascript:alert(1)[/img]]text[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "PHP-Fusion nested tag blocked"
            );
        }

        // Friendica style - multiple tags with injection
        #[test]
        fn friendica_color_injection() {
            let result = parse("[color=\"#000000\" onclick=\"alert(1)\"]test[/color]");
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "Friendica color onclick blocked. Output: {}",
                result
            );
        }

        #[test]
        fn friendica_size_injection() {
            let result = parse("[size=\"30\" onclick=\"alert(1)\"]test[/size]");
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "Friendica size onclick blocked. Output: {}",
                result
            );
        }

        #[test]
        fn friendica_font_injection() {
            let result = parse("[font=\"Arial\" onclick=\"alert(1)\"]test[/font]");
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "Friendica font onclick blocked. Output: {}",
                result
            );
        }

        #[test]
        fn friendica_img_link_injection() {
            let result = parse("[img=200x100]javascript:alert(1)[/img]");
            assert!(
                !result.to_lowercase().contains("src=\"javascript:"),
                "Friendica img javascript blocked"
            );
        }

        #[test]
        fn friendica_url_link_injection() {
            let result = parse("[url=\"javascript:alert(1)\"]Click me[/url]");
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Friendica url javascript blocked"
            );
        }

        // AOblogger/MyBloggie style
        #[test]
        fn aoblogger_script_tag() {
            let result = parse("[url]<script>alert(1)</script>[/url]");
            assert!(!result.contains("<script>"), "Script in URL content blocked");
        }

        // PostBoard style
        #[test]
        fn postboard_onclick_injection() {
            let result = parse(r#"[url=http://x.com onclick=alert(1)]test[/url]"#);
            // Note: no quotes around onclick value
            // Safe if: tag is rejected (rendered as raw BBCode) OR no onclick in HTML
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "PostBoard onclick without quotes blocked. Output: {}",
                result
            );
        }

        // Land Down Under style
        #[test]
        fn ldu_email_onclick() {
            let result = parse(r#"[email]test@test.com onclick=alert(1)[/email]"#);
            // Safe if: email is rejected (rendered as raw BBCode) OR no onclick in HTML
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "LDU email onclick blocked. Output: {}",
                result
            );
        }

        // W-Agora style
        #[test]
        fn wagora_script_src() {
            let result = parse("[url]http://x.com/x.js[/url]<script src=http://evil.com/xss.js></script>");
            assert!(!result.contains("<script"), "W-Agora script injection blocked");
        }

        // eoCMS style
        #[test]
        fn eocms_img_src_injection() {
            let result = parse(r#"[img]http://x/x.gif" onmouseover="alert(1)[/img]"#);
            assert!(
                !has_dangerous_event_handler(&result, "onmouseover"),
                "eoCMS img onmouseover blocked. Output: {}",
                result
            );
        }
    }

    // ========================================================================
    // SECTION 15: ADDITIONAL PROTOCOL SCHEMES
    // Goal: Block additional dangerous URL protocols.
    // ========================================================================

    mod additional_protocols {
        use super::*;

        #[test]
        fn file_protocol() {
            let result = parse("[url=file:///etc/passwd]Click[/url]");
            assert!(
                !result.contains("href=\"file:"),
                "file: protocol blocked"
            );
        }

        #[test]
        fn ftp_protocol() {
            // FTP is often blocked in high-security contexts
            let result = parse("[url=ftp://evil.com/malware.exe]Download[/url]");
            // This may or may not be blocked depending on allowed_schemes config
            // But it should never contain javascript:
            assert!(!result.contains("javascript:"));
        }

        #[test]
        fn telnet_protocol() {
            let result = parse("[url=telnet://evil.com]Connect[/url]");
            assert!(
                !result.contains("href=\"telnet:"),
                "telnet: protocol blocked"
            );
        }

        #[test]
        fn ms_its_protocol() {
            // IE-specific protocol
            let result = parse("[url=ms-its:mhtml:file://c:\\foo.mht!http://www.example.com/chm.htm::evilscript.chm]Click[/url]");
            assert!(
                !result.contains("href=\"ms-its:"),
                "ms-its: protocol blocked"
            );
        }

        #[test]
        fn mhtml_protocol() {
            let result = parse("[url=mhtml:file://C:/foo.mhtml]Click[/url]");
            assert!(
                !result.contains("href=\"mhtml:"),
                "mhtml: protocol blocked"
            );
        }

        #[test]
        fn jar_protocol() {
            // Java archive protocol - can be used for XSS
            let result = parse("[url=jar:https://example.com/evil.jar!/attack.html]Click[/url]");
            assert!(
                !result.contains("href=\"jar:"),
                "jar: protocol blocked"
            );
        }

        #[test]
        fn about_protocol() {
            let result = parse("[url=about:blank]Click[/url]");
            // about: can sometimes be used for XSS in certain contexts
            // Main check is it doesn't contain javascript
            assert!(!result.to_lowercase().contains("javascript:"));
        }

        #[test]
        fn view_source_protocol() {
            let result = parse("[url=view-source:javascript:alert(1)]Click[/url]");
            assert!(
                !result.contains("href=\"view-source:"),
                "view-source: protocol blocked"
            );
        }

        #[test]
        fn res_protocol() {
            // Windows resource protocol
            let result = parse("[url=res://ieframe.dll/acr_error.htm#javascript:alert(1)]Click[/url]");
            assert!(
                !result.contains("href=\"res:"),
                "res: protocol blocked"
            );
        }

        #[test]
        fn blob_protocol() {
            let result = parse("[url=blob:https://example.com/12345678-1234-1234-1234-123456789012]Click[/url]");
            assert!(
                !result.contains("href=\"blob:"),
                "blob: protocol blocked"
            );
        }
    }

    // ========================================================================
    // SECTION 16: REGRESSION TESTS
    // ========================================================================

    mod regression {
        use super::*;

        #[test]
        fn legitimate_urls_still_work() {
            let result = parse("[url=https://example.com]Link[/url]");
            assert!(
                result.contains("href=\"https://example.com\""),
                "HTTPS URLs work"
            );
        }

        #[test]
        fn legitimate_http_urls_work() {
            let result = parse("[url=http://example.com]Link[/url]");
            assert!(
                result.contains("href=\"http://example.com\""),
                "HTTP URLs work"
            );
        }

        #[test]
        fn legitimate_mailto_works() {
            let result = parse("[email]user@example.com[/email]");
            assert!(
                result.contains("mailto:user@example.com"),
                "mailto links work"
            );
        }

        #[test]
        fn legitimate_images_work() {
            let result = parse("[img]https://example.com/image.png[/img]");
            assert!(
                result.contains("src=\"https://example.com/image.png\""),
                "HTTPS images work"
            );
        }

        #[test]
        fn legitimate_colors_work() {
            let result = parse("[color=red]Text[/color]");
            assert!(result.contains("color: red"), "Named colors work");
        }

        #[test]
        fn legitimate_hex_colors_work() {
            let result = parse("[color=#ff0000]Text[/color]");
            assert!(result.contains("color: #ff0000"), "Hex colors work");
        }

        #[test]
        fn legitimate_formatting_works() {
            let result = parse("[b]Bold[/b] [i]Italic[/i] [u]Underline[/u]");
            assert!(result.contains("<strong>Bold</strong>"), "Bold works");
            assert!(result.contains("<em>Italic</em>"), "Italic works");
            assert!(result.contains("<u>Underline</u>"), "Underline works");
        }
    }

    // ========================================================================
    // SECTION 17: XENFORO-SPECIFIC SECURITY PATTERNS
    // Based on XF\BbCode\Renderer\Html.php security validations.
    // ========================================================================

    mod xenforo_security_patterns {
        use super::*;

        // --- URL Validation (from XF Html.php:1685) ---
        // XenForo blocks: data:, https?://data, javascript:, about:

        #[test]
        fn xf_https_data_url_disguise() {
            // XenForo specifically blocks https://data URLs (disguised data URLs)
            let result = parse("[url=https://data:text/html,<script>alert(1)</script>]Click[/url]");
            // Should not create a dangerous link
            assert!(
                !result.contains("<script>"),
                "https://data disguise blocked"
            );
        }

        #[test]
        fn xf_http_data_url_disguise() {
            let result = parse("[url=http://data:text/html,<script>alert(1)</script>]Click[/url]");
            assert!(
                !result.contains("<script>"),
                "http://data disguise blocked"
            );
        }

        #[test]
        fn xf_newline_in_url_blocked() {
            // XenForo blocks URLs containing newlines (line 1680-1683)
            let result = parse("[url=https://example.com\n/path]Click[/url]");
            // Either reject the URL or sanitize it
            if result.contains("href=") {
                assert!(
                    !result.contains("href=\"https://example.com\n"),
                    "Newline in URL should be blocked or sanitized"
                );
            }
        }

        // --- CSS System Colors Blacklist (from XF Html.php:1309-1360) ---
        // XenForo blocks these to prevent UI confusion/spoofing

        #[test]
        fn xf_transparent_color_blocked() {
            let result = parse("[color=transparent]Hidden text[/color]");
            // transparent should be rejected - renders as text
            assert!(
                !result.contains("color: transparent"),
                "transparent color should be blocked"
            );
        }

        #[test]
        fn xf_system_color_activeborder() {
            let result = parse("[color=activeborder]Text[/color]");
            // System colors should be rejected
            assert!(
                !result.contains("color: activeborder"),
                "activeborder system color blocked"
            );
        }

        #[test]
        fn xf_system_color_buttonface() {
            let result = parse("[color=buttonface]Text[/color]");
            assert!(
                !result.contains("color: buttonface"),
                "buttonface system color blocked"
            );
        }

        #[test]
        fn xf_system_color_window() {
            let result = parse("[color=window]Text[/color]");
            assert!(
                !result.contains("color: window"),
                "window system color blocked"
            );
        }

        #[test]
        fn xf_system_color_highlight() {
            let result = parse("[color=highlight]Text[/color]");
            assert!(
                !result.contains("color: highlight"),
                "highlight system color blocked"
            );
        }

        #[test]
        fn xf_system_color_menu() {
            let result = parse("[color=menu]Text[/color]");
            assert!(
                !result.contains("color: menu"),
                "menu system color blocked"
            );
        }

        #[test]
        fn xf_system_color_canvas() {
            let result = parse("[color=canvas]Text[/color]");
            // Canvas is a CSS4 system color
            assert!(
                !result.contains("color: canvas"),
                "canvas system color blocked"
            );
        }

        #[test]
        fn xf_system_color_case_insensitive() {
            let result = parse("[color=TRANSPARENT]Text[/color]");
            assert!(
                !result.to_lowercase().contains("color: transparent"),
                "transparent blocked case-insensitively"
            );
        }

        // --- Font Validation (from XF Html.php:1372) ---

        #[test]
        fn xf_font_inherit_blocked() {
            // XenForo blocks "inherit" as font-family to prevent style inheritance attacks
            let result = parse("[font=inherit]Text[/font]");
            // Should either block entirely or not use inherit
            if result.contains("font-family:") {
                assert!(
                    !result.contains("font-family: inherit")
                        && !result.contains("font-family: 'inherit'")
                        && !result.contains("font-family:inherit"),
                    "inherit keyword should be blocked in font-family"
                );
            }
        }

        #[test]
        fn xf_font_with_quotes_escape() {
            // XenForo wraps font names in quotes after escaping
            let result = parse("[font=Arial]Text[/font]");
            if result.contains("font-family:") {
                // Should be escaped properly, often wrapped in quotes
                assert!(
                    result.contains("font-family:") && !result.contains("<script"),
                    "Font should be safely rendered"
                );
            }
        }

        // --- Size Validation (from XF Html.php:1392) ---

        #[test]
        fn xf_size_non_numeric_rejected() {
            // XenForo uses intval() which returns 0 for non-numeric
            let result = parse("[size=abc]Text[/size]");
            // Non-numeric sizes should be rejected or rendered as text
            // If rendered, it shouldn't be "abc"
            assert!(
                !result.contains("font-size: abc"),
                "Non-numeric size rejected"
            );
        }

        #[test]
        fn xf_size_negative_handled() {
            let result = parse("[size=-5]Text[/size]");
            // Negative sizes should be rejected or clamped
            assert!(
                !result.contains("font-size: -"),
                "Negative size handled safely"
            );
        }

        #[test]
        fn xf_size_zero_handled() {
            let result = parse("[size=0]Text[/size]");
            // Zero size should be rejected (would make text invisible)
            assert!(
                !result.contains("font-size: 0px") && !result.contains("font-size: 0;"),
                "Zero size should not render invisible text"
            );
        }

        #[test]
        fn xf_size_extremely_large() {
            let result = parse("[size=9999]Text[/size]");
            // Very large sizes should be clamped or rejected
            if result.contains("font-size:") {
                // If it renders, check the size is reasonable
                assert!(
                    !result.contains("font-size: 9999"),
                    "Extremely large size should be clamped"
                );
            }
        }
    }

    // ========================================================================
    // SECTION 18: PHPBB/S9E-SPECIFIC SECURITY PATTERNS
    // Based on s9e\TextFormatter repository.xml and phpBB validations.
    // ========================================================================

    mod phpbb_security_patterns {
        use super::*;

        // --- RANGE Validation (s9e repository.xml SIZE tag) ---
        // phpBB uses RANGE=8,36 for font sizes

        #[test]
        fn phpbb_size_below_minimum() {
            let result = parse("[size=1]Tiny[/size]");
            // Size 1 in phpBB scale should still work, but very small
            // Main check: doesn't cause issues
            assert!(result.contains("Tiny"), "Content preserved");
        }

        #[test]
        fn phpbb_size_percentage_style() {
            // phpBB uses percentage-based sizing (50-200%)
            let result = parse("[size=150]Large[/size]");
            // Should either render as percentage or convert appropriately
            if result.contains("font-size:") {
                assert!(
                    !result.contains("<script>"),
                    "No script injection via size"
                );
            }
        }

        // --- CHOICE Validation (s9e ALIGN tag) ---

        #[test]
        fn phpbb_align_invalid_choice() {
            // Only left, right, center, justify should be valid
            // Invalid values should be rejected
            let result = parse("[align=diagonal]Text[/align]");
            // [align] is not a supported tag in our lib, should render as text
            assert!(
                result.contains("[align=diagonal]") || !result.contains("text-align: diagonal"),
                "Invalid align choice rejected"
            );
        }

        // --- HASHMAP Validation (s9e LIST tag) ---

        #[test]
        fn phpbb_list_type_validation() {
            // Valid types: 1, a, A, i, I (mapped to decimal, lower-alpha, etc.)
            let result = parse("[list=1][*]Item[/list]");
            assert!(result.contains("<ol"), "Decimal list works");
        }

        #[test]
        fn phpbb_list_invalid_type() {
            let result = parse("[list=invalid][*]Item[/list]");
            // Invalid list type should fallback to default or be rejected
            // Should not cause errors
            assert!(result.contains("Item"), "Content preserved with invalid list type");
        }

        // --- URL Validation (s9e URL tag) ---

        #[test]
        fn phpbb_url_scheme_whitelist() {
            // phpBB only allows configured schemes
            let result = parse("[url=gopher://old.server.com]Old Link[/url]");
            // Gopher protocol should be blocked (not in whitelist)
            assert!(
                !result.contains("href=\"gopher:"),
                "Gopher protocol blocked"
            );
        }

        // --- IMG Validation ---

        #[test]
        fn phpbb_img_with_dimensions_xss() {
            // Testing dimension parameters for XSS
            let result = parse("[img=100x100\" onerror=\"alert(1)]https://example.com/img.png[/img]");
            assert!(
                !result.contains(" onerror="),
                "XSS via img dimensions blocked"
            );
        }

        // --- Quote Attribute Injection ---

        #[test]
        fn phpbb_quote_attribute_injection() {
            // Testing s9e quote tag attribute injection
            let result = parse(r#"[quote author="test" onclick="alert(1)"]Text[/quote]"#);
            // The onclick should not become an HTML attribute
            assert!(
                !result.contains(" onclick=\"alert"),
                "Quote attribute injection blocked"
            );
        }

        // --- Noparse/Plain Tag Security ---

        #[test]
        fn phpbb_noparse_preserves_dangerous_text() {
            // Noparse should preserve but escape dangerous content
            let result = parse("[noparse]<script>alert(1)</script>[/noparse]");
            assert!(
                !result.contains("<script>alert"),
                "Script in noparse escaped"
            );
            // Content should be escaped
            assert!(
                result.contains("&lt;script&gt;") || result.contains("[noparse]"),
                "Noparse content properly handled"
            );
        }

        // --- Table Attribute Injection ---

        #[test]
        fn phpbb_td_colspan_xss() {
            // Testing colspan parameter for XSS (s9e allows UINT for colspan)
            let result = parse(r#"[table][tr][td colspan="1" onclick="alert(1)"]Cell[/td][/tr][/table]"#);
            assert!(
                !result.contains(" onclick="),
                "TD colspan XSS blocked"
            );
        }

        #[test]
        fn phpbb_td_align_injection() {
            // s9e CHOICE filter for align should prevent injection
            let result = parse(r#"[table][tr][td align="left; onclick: alert(1)"]Cell[/td][/tr][/table]"#);
            // The malicious align value should be rejected or sanitized
            assert!(
                !result.contains("onclick:"),
                "TD align injection blocked"
            );
        }

        // --- EMAIL Tag Injection ---

        #[test]
        fn phpbb_email_javascript_injection() {
            let result = parse("[email]javascript:alert(1)[/email]");
            assert!(
                !result.contains("href=\"javascript:"),
                "JavaScript in email blocked"
            );
        }

        #[test]
        fn phpbb_email_attribute_injection() {
            let result = parse(r#"[email=test@example.com" onclick="alert(1)]Contact[/email]"#);
            // The email validation should fail due to the quote, rendering as text
            // Check that onclick doesn't appear as an actual HTML attribute
            assert!(
                !has_dangerous_event_handler(&result, "onclick"),
                "Email attribute injection blocked. Output: {}",
                result
            );
        }
    }

    // ========================================================================
    // SECTION 19: COMBINED PLATFORM SECURITY TESTS
    // Tests that verify security across both XenForo and phpBB patterns.
    // ========================================================================

    mod combined_platform_security {
        use super::*;

        #[test]
        fn both_platforms_block_javascript_urls() {
            let variants = vec![
                "javascript:alert(1)",
                "JAVASCRIPT:alert(1)",
                "JaVaScRiPt:alert(1)",
                " javascript:alert(1)",
                "\tjavascript:alert(1)",
                "\njavascript:alert(1)",
            ];
            for variant in variants {
                let result = parse(&format!("[url={}]Click[/url]", variant));
                assert!(
                    !result.to_lowercase().contains("href=\"javascript:"),
                    "JavaScript URL variant blocked: {}",
                    variant
                );
            }
        }

        #[test]
        fn both_platforms_block_data_urls() {
            let variants = vec![
                "data:text/html,<script>alert(1)</script>",
                "DATA:text/html,<script>alert(1)</script>",
                "data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==",
            ];
            for variant in variants {
                let result = parse(&format!("[img]{}[/img]", variant));
                assert!(
                    !result.contains("<img") || !result.to_lowercase().contains("src=\"data:"),
                    "Data URL variant blocked in img: {}",
                    variant
                );
            }
        }

        #[test]
        fn both_platforms_escape_html_in_content() {
            let dangerous_content = "<script>alert('xss')</script>";
            let result = parse(&format!("[b]{}[/b]", dangerous_content));
            assert!(
                !result.contains("<script>"),
                "Script tag escaped in content"
            );
            assert!(
                result.contains("&lt;script&gt;"),
                "Script tag HTML-escaped"
            );
        }

        #[test]
        fn both_platforms_handle_nested_attack() {
            // Combining multiple attack vectors
            let input = r#"[quote="<script>"][url=javascript:alert(1)][img]data:text/html,<script>[/img][/url][/quote]"#;
            let result = parse(input);
            assert!(!result.contains("<script>"), "No script tags");
            assert!(
                !result.contains("href=\"javascript:"),
                "No javascript URLs"
            );
            assert!(
                !result.contains("src=\"data:") || !result.contains("<img"),
                "No data URLs in img"
            );
        }

        #[test]
        fn both_platforms_handle_encoding_attacks() {
            // HTML entity encoded javascript
            let result = parse("[url=&#106;&#97;&#118;&#97;&#115;&#99;&#114;&#105;&#112;&#116;&#58;alert(1)]Click[/url]");
            // Should not decode and execute
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "HTML entity encoded javascript blocked"
            );
        }

        #[test]
        fn valid_content_still_works() {
            // Ensure security doesn't break legitimate content
            let input = r#"[b]Hello[/b] [i]World[/i]
[url=https://example.com]Link[/url]
[img]https://example.com/image.png[/img]
[quote="User"]Some quote[/quote]
[code]console.log("test");[/code]
[color=red]Red text[/color]
[size=4]Sized text[/size]"#;
            let result = parse(input);

            assert!(result.contains("<strong>Hello</strong>"), "Bold works");
            assert!(result.contains("<em>World</em>"), "Italic works");
            assert!(result.contains("href=\"https://example.com\""), "URL works");
            assert!(result.contains("src=\"https://example.com/image.png\""), "Img works");
            assert!(result.contains("<blockquote"), "Quote works");
            assert!(result.contains("<code>"), "Code works");
            assert!(result.contains("color:"), "Color works");
            assert!(result.contains("font-size:"), "Size works");
        }
    }
}

// ============================================================================
// Realistic Forum Post Tests
// ============================================================================

mod realistic {
    use super::*;

    #[test]
    fn forum_post_with_quote() {
        let input = r#"[quote="PreviousUser"]I think this is a great idea![/quote]

I agree with this. Here are my thoughts:

[list=1]
[*]First point
[*]Second point
[*]Third point
[/list]

For more info, check [url=https://example.com]this link[/url].

Thanks!"#;

        let result = parse(input);
        assert!(result.contains("<blockquote"));
        assert!(result.contains("PreviousUser wrote:"));
        assert!(result.contains("<ol"));
        // List items may contain linebreaks
        assert!(result.contains("First point"));
        assert!(result.contains("Second point"));
        assert!(result.contains("Third point"));
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn code_example_post() {
        let input = r#"Here's how to do it in Rust:

[code=rust]
fn main() {
    println!("Hello, world!");
}
[/code]

You can also use [icode]println![/icode] macro for debugging.

[b]Note:[/b] Don't forget to handle errors!"#;

        let result = parse(input);
        assert!(result.contains("language-rust"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("<code"));
        // Check the Note is present and bolded
        assert!(result.contains("Note:"), "Result should contain 'Note:'");
        assert!(
            result.contains("<strong>"),
            "Result should contain <strong> tags"
        );
    }

    #[test]
    fn signature() {
        let input = r#"[center][size=2][color=gray]
[img]https://example.com/sig.png[/img]
Member since 2020 | [url=https://example.com]My Website[/url]
[/color][/size][/center]"#;

        let result = parse(input);
        assert!(result.contains("text-align: center"));
        assert!(result.contains("<img"));
        assert!(result.contains("color: gray"));
    }

    #[test]
    fn announcement() {
        let input = r#"[center][b][size=6][color=red]⚠️ IMPORTANT ANNOUNCEMENT ⚠️[/color][/size][/b][/center]

[spoiler="Click to read details"]
This is an important update to our community guidelines.

Please review the following:
[list]
[*]Rule 1: Be respectful
[*]Rule 2: No spam
[*]Rule 3: Have fun!
[/list]
[/spoiler]

[right][size=2]Posted by Admin[/size][/right]"#;

        let result = parse(input);
        assert!(result.contains("⚠️ IMPORTANT ANNOUNCEMENT ⚠️"));
        assert!(result.contains("<details"));
        assert!(result.contains("<strong>"));
    }

    #[test]
    fn table_comparison() {
        let input = r#"[table]
[tr][th]Feature[/th][th]Free[/th][th]Pro[/th][/tr]
[tr][td]Basic Access[/td][td]✓[/td][td]✓[/td][/tr]
[tr][td]Premium Content[/td][td]✗[/td][td]✓[/td][/tr]
[/table]"#;

        let result = parse(input);
        assert!(result.contains("<table"));
        assert!(result.contains("<th>Feature</th>"));
        assert!(result.contains("<td>✓</td>"));
    }
}
