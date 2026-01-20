//! # BBCode Parser
//!
//! A zero-copy BBCode parser supporting phpBB and XenForo syntax.
//!
//! ## Features
//!
//! - **Zero-copy parsing**: Uses `winnow` for efficient parsing without string allocation
//! - **Full BBCode support**: Supports all standard phpBB and XenForo BBCode tags
//! - **HTML rendering**: Converts BBCode to safe, escaped HTML
//! - **Customizable**: Configurable tag registry and renderer settings
//! - **Safe**: XSS protection and URL validation
//!
//! ## Quick Start
//!
//! ```rust
//! use bbcode::parse;
//!
//! let html = parse("[b]Hello[/b] [i]World[/i]!");
//! assert_eq!(html, "<strong>Hello</strong> <em>World</em>!");
//! ```
//!
//! ## Supported Tags
//!
//! ### Basic Formatting
//! - `[b]`, `[i]`, `[u]`, `[s]` - Bold, italic, underline, strikethrough
//! - `[color=...]`, `[font=...]`, `[size=...]` - Color, font, and size
//! - `[sub]`, `[sup]` - Subscript and superscript
//!
//! ### Links and Images
//! - `[url]`, `[url=...]` - Links
//! - `[email]`, `[email=...]` - Email links
//! - `[img]` - Images
//!
//! ### Block Elements
//! - `[quote]`, `[quote=...]` - Quotations
//! - `[code]`, `[code=lang]` - Code blocks
//! - `[icode]` - Inline code
//! - `[list]`, `[*]` - Lists
//!
//! ### Alignment
//! - `[left]`, `[center]`, `[right]`, `[justify]` - Text alignment
//! - `[indent]` - Indentation
//! - `[heading=N]` - Headings
//!
//! ### Tables
//! - `[table]`, `[tr]`, `[td]`, `[th]` - Tables
//!
//! ### Special
//! - `[spoiler]`, `[ispoiler]` - Spoiler tags
//! - `[hr]`, `[br]` - Horizontal rule and line break
//! - `[plain]` - Disable BBCode parsing
//!
//! ## Advanced Usage
//!
//! For more control, use the parser and renderer separately:
//!
//! ```rust
//! use bbcode::{Parser, Renderer, ParserConfig, RenderConfig};
//!
//! // Custom parser config
//! let config = ParserConfig {
//!     max_depth: 10,
//!     auto_link: true,
//!     ..Default::default()
//! };
//!
//! let parser = Parser::with_config(config);
//! let doc = parser.parse("[b]Hello[/b]");
//!
//! // Custom renderer config
//! let render_config = RenderConfig {
//!     class_prefix: "my-bbcode".into(),
//!     nofollow_links: true,
//!     ..Default::default()
//! };
//!
//! let renderer = Renderer::with_config(render_config);
//! let html = renderer.render(&doc);
//! ```

pub mod ast;
pub mod error;
pub mod parser;
pub mod renderer;
pub mod tags;
pub mod tokenizer;

// Re-exports for convenience
pub use ast::{Document, Node, TagNode, TagOption, TagType};
pub use error::{ParseError, RenderError};
pub use parser::{Parser, ParserConfig};
pub use renderer::{escape_html, CustomTagHandler, RenderConfig, RenderContext, Renderer};
pub use tags::{CustomTagDef, ResolvedTag, TagDef, TagRegistry, STANDARD_TAGS};
pub use tokenizer::{tokenize, Token};

/// Parses BBCode and renders it to HTML with default settings.
///
/// This is a convenience function that combines parsing and rendering
/// in a single call using default configurations.
///
/// # Example
///
/// ```rust
/// use bbcode::parse;
///
/// let html = parse("[b]Bold[/b] text");
/// assert_eq!(html, "<strong>Bold</strong> text");
/// ```
pub fn parse(input: &str) -> String {
    let parser = Parser::new();
    let doc = parser.parse(input);
    let renderer = Renderer::new();
    renderer.render(&doc)
}

/// Parses BBCode and renders it to HTML with custom configurations.
///
/// # Example
///
/// ```rust
/// use bbcode::{parse_with_config, ParserConfig, RenderConfig};
///
/// let parser_config = ParserConfig::default();
/// let render_config = RenderConfig {
///     class_prefix: "forum".into(),
///     ..Default::default()
/// };
///
/// let html = parse_with_config("[b]Bold[/b]", &parser_config, &render_config);
/// ```
pub fn parse_with_config(
    input: &str,
    parser_config: &ParserConfig,
    render_config: &RenderConfig,
) -> String {
    let parser = Parser::with_config(parser_config.clone());
    let doc = parser.parse(input);
    let renderer = Renderer::with_config(render_config.clone());
    renderer.render(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Integration Tests - Full Pipeline
    // ============================================================================

    #[test]
    fn test_parse_empty() {
        assert_eq!(parse(""), "");
    }

    #[test]
    fn test_parse_plain_text() {
        assert_eq!(parse("Hello, world!"), "Hello, world!");
    }

    #[test]
    fn test_parse_bold() {
        assert_eq!(parse("[b]Bold[/b]"), "<strong>Bold</strong>");
    }

    #[test]
    fn test_parse_italic() {
        assert_eq!(parse("[i]Italic[/i]"), "<em>Italic</em>");
    }

    #[test]
    fn test_parse_underline() {
        assert_eq!(parse("[u]Underline[/u]"), "<u>Underline</u>");
    }

    #[test]
    fn test_parse_strikethrough() {
        assert_eq!(parse("[s]Strike[/s]"), "<s>Strike</s>");
    }

    #[test]
    fn test_parse_nested() {
        assert_eq!(
            parse("[b][i]Bold Italic[/i][/b]"),
            "<strong><em>Bold Italic</em></strong>"
        );
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(parse("[B]Bold[/B]"), "<strong>Bold</strong>");
        assert_eq!(parse("[B]Bold[/b]"), "<strong>Bold</strong>");
        assert_eq!(parse("[b]Bold[/B]"), "<strong>Bold</strong>");
    }

    #[test]
    fn test_html_escaping() {
        assert_eq!(
            parse("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_parse_linebreaks() {
        assert_eq!(parse("Line 1\nLine 2"), "Line 1<br />Line 2");
    }

    #[test]
    fn test_parse_hr() {
        assert_eq!(parse("Before[hr]After"), "Before<hr />After");
    }

    // ============================================================================
    // URL Tests
    // ============================================================================

    #[test]
    fn test_url_with_option() {
        let result = parse("[url=https://example.com]Link[/url]");
        assert!(result.contains("href=\"https://example.com\""));
        assert!(result.contains(">Link</a>"));
    }

    #[test]
    fn test_url_without_option() {
        let result = parse("[url]https://example.com[/url]");
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn test_url_auto_detection() {
        let result = parse("Visit https://example.com today!");
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn test_url_javascript_blocked() {
        let result = parse("[url=javascript:alert('xss')]Click[/url]");
        // Should NOT create a link
        assert!(!result.contains("href=\"javascript"));
    }

    // ============================================================================
    // Image Tests
    // ============================================================================

    #[test]
    fn test_img() {
        let result = parse("[img]https://example.com/image.png[/img]");
        assert!(result.contains("<img"));
        assert!(result.contains("src=\"https://example.com/image.png\""));
    }

    #[test]
    fn test_img_javascript_blocked() {
        let result = parse("[img]javascript:alert('xss')[/img]");
        assert!(!result.contains("<img"));
    }

    // ============================================================================
    // Quote Tests
    // ============================================================================

    #[test]
    fn test_quote() {
        let result = parse("[quote]Quoted text[/quote]");
        assert!(result.contains("<blockquote"));
        assert!(result.contains("Quoted text"));
    }

    #[test]
    fn test_quote_with_author() {
        let result = parse("[quote=\"John Doe\"]Quote[/quote]");
        assert!(result.contains("John Doe wrote:"));
    }

    // ============================================================================
    // Code Tests
    // ============================================================================

    #[test]
    fn test_code_block() {
        let result = parse("[code]function test() {}[/code]");
        assert!(result.contains("<pre"));
        assert!(result.contains("<code>"));
    }

    #[test]
    fn test_code_preserves_bbcode() {
        let result = parse("[code][b]Not bold[/b][/code]");
        // BBCode inside code should be escaped, not parsed
        assert!(result.contains("[b]Not bold[/b]"));
    }

    #[test]
    fn test_code_escapes_html() {
        let result = parse("[code]<script>alert('xss')</script>[/code]");
        assert!(result.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_icode() {
        let result = parse("Use [icode]console.log()[/icode] to debug");
        assert!(result.contains("<code"));
        assert!(result.contains("console.log()"));
    }

    // ============================================================================
    // List Tests
    // ============================================================================

    #[test]
    fn test_unordered_list() {
        let result = parse("[list][*]Item 1[*]Item 2[/list]");
        assert!(result.contains("<ul"));
        assert!(result.contains("<li>Item 1</li>"));
        assert!(result.contains("<li>Item 2</li>"));
    }

    #[test]
    fn test_ordered_list() {
        let result = parse("[list=1][*]First[*]Second[/list]");
        assert!(result.contains("<ol"));
    }

    // ============================================================================
    // Color Tests
    // ============================================================================

    #[test]
    fn test_color_named() {
        let result = parse("[color=red]Red text[/color]");
        assert!(result.contains("color: red;"));
    }

    #[test]
    fn test_color_hex() {
        let result = parse("[color=#ff0000]Red text[/color]");
        assert!(result.contains("color: #ff0000;"));
    }

    #[test]
    fn test_color_invalid() {
        let result = parse("[color=invalid]text[/color]");
        // Invalid color should render as text
        assert!(result.contains("[color=invalid]"));
    }

    // ============================================================================
    // Size Tests
    // ============================================================================

    #[test]
    fn test_size_xenforo_style() {
        let result = parse("[size=4]Large[/size]");
        assert!(result.contains("font-size:"));
    }

    // ============================================================================
    // Alignment Tests
    // ============================================================================

    #[test]
    fn test_center() {
        let result = parse("[center]Centered[/center]");
        assert!(result.contains("text-align: center;"));
    }

    #[test]
    fn test_right() {
        let result = parse("[right]Right[/right]");
        assert!(result.contains("text-align: right;"));
    }

    // ============================================================================
    // Spoiler Tests
    // ============================================================================

    #[test]
    fn test_spoiler() {
        let result = parse("[spoiler]Hidden[/spoiler]");
        assert!(result.contains("<details"));
        assert!(result.contains("<summary>"));
    }

    #[test]
    fn test_spoiler_with_title() {
        let result = parse("[spoiler=Reveal]Hidden[/spoiler]");
        assert!(result.contains("Reveal"));
    }

    // ============================================================================
    // Table Tests
    // ============================================================================

    #[test]
    fn test_table() {
        let result = parse("[table][tr][td]Cell 1[/td][td]Cell 2[/td][/tr][/table]");
        assert!(result.contains("<table"));
        assert!(result.contains("<tr>"));
        assert!(result.contains("<td>"));
        assert!(result.contains("Cell 1"));
        assert!(result.contains("Cell 2"));
    }

    // ============================================================================
    // Plain/NoParse Tests
    // ============================================================================

    #[test]
    fn test_plain() {
        let result = parse("[plain][b]Not Bold[/b][/plain]");
        assert!(!result.contains("<strong>"));
        assert!(result.contains("[b]Not Bold[/b]"));
    }

    #[test]
    fn test_noparse() {
        let result = parse("[noparse][i]Not Italic[/i][/noparse]");
        assert!(!result.contains("<em>"));
    }

    #[test]
    fn test_nobbc_alias() {
        let result = parse("[nobbc][u]Not Underlined[/u][/nobbc]");
        assert!(!result.contains("<u>"));
        assert!(result.contains("[u]Not Underlined[/u]"));
    }

    #[test]
    fn test_plain_with_multiple_tags() {
        let result = parse("[plain][b]bold[/b] [i]italic[/i] [u]underline[/u][/plain]");
        assert!(!result.contains("<strong>"));
        assert!(!result.contains("<em>"));
        assert!(!result.contains("<u>"));
        assert!(result.contains("[b]bold[/b]"));
        assert!(result.contains("[i]italic[/i]"));
        assert!(result.contains("[u]underline[/u]"));
    }

    #[test]
    fn test_plain_with_url() {
        let result = parse("[plain][url=https://example.com]link[/url][/plain]");
        assert!(!result.contains("<a"));
        assert!(result.contains("[url=https://example.com]link[/url]"));
    }

    #[test]
    fn test_plain_with_image() {
        let result = parse("[plain][img]https://example.com/image.png[/img][/plain]");
        assert!(!result.contains("<img"));
        assert!(result.contains("[img]https://example.com/image.png[/img]"));
    }

    #[test]
    fn test_plain_with_color() {
        let result = parse("[plain][color=red]Red Text[/color][/plain]");
        assert!(!result.contains("color:"));
        assert!(result.contains("[color=red]Red Text[/color]"));
    }

    #[test]
    fn test_plain_with_size() {
        let result = parse("[plain][size=20]Big Text[/size][/plain]");
        assert!(!result.contains("font-size"));
        assert!(result.contains("[size=20]Big Text[/size]"));
    }

    #[test]
    fn test_plain_with_quote() {
        let result = parse("[plain][quote]Not a quote[/quote][/plain]");
        assert!(!result.contains("<blockquote"));
        assert!(result.contains("[quote]Not a quote[/quote]"));
    }

    #[test]
    fn test_plain_with_code() {
        let result = parse("[plain][code]function test() {}[/code][/plain]");
        assert!(!result.contains("<pre"));
        assert!(result.contains("[code]function test() {}[/code]"));
    }

    #[test]
    fn test_plain_with_list() {
        let result = parse("[plain][list][*]item1[*]item2[/list][/plain]");
        assert!(!result.contains("<ul"));
        assert!(!result.contains("<li"));
        assert!(result.contains("[list]"));
        assert!(result.contains("[*]item1"));
    }

    #[test]
    fn test_plain_with_table() {
        let result = parse("[plain][table][tr][td]cell[/td][/tr][/table][/plain]");
        assert!(!result.contains("<table"));
        assert!(result.contains("[table]"));
        assert!(result.contains("[td]cell[/td]"));
    }

    #[test]
    fn test_plain_escapes_html() {
        let result = parse("[plain]<script>alert('xss')</script>[/plain]");
        assert!(!result.contains("<script>"));
        assert!(result.contains("&lt;script&gt;"));
        assert!(result.contains("&lt;/script&gt;"));
    }

    #[test]
    fn test_plain_with_special_chars() {
        let result = parse("[plain]& < > \" '[/plain]");
        assert!(result.contains("&amp;"));
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(result.contains("&quot;"));
    }

    #[test]
    fn test_plain_with_newlines() {
        let result = parse("[plain]line1\nline2\nline3[/plain]");
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
        assert!(result.contains("line3"));
        // Should convert newlines since convert_newlines is true
        assert!(result.contains("<br>") || result.contains("\n"));
    }

    #[test]
    fn test_plain_preserves_whitespace() {
        let result = parse("[plain]   spaces   [/plain]");
        assert!(result.contains("spaces"));
    }

    #[test]
    fn test_plain_empty() {
        let result = parse("[plain][/plain]");
        // Empty plain tag should produce empty or minimal output
        // Just ensure parsing succeeds without panic
        let _ = result;
    }

    #[test]
    fn test_plain_nested_plain_tags() {
        let result = parse("[plain][plain]inner[/plain][/plain]");
        // Should treat inner [plain] as literal text
        assert!(result.contains("[plain]inner[/plain]"));
    }

    #[test]
    fn test_plain_with_unclosed_tags_inside() {
        let result = parse("[plain][b]unclosed[/plain]");
        assert!(result.contains("[b]unclosed"));
        assert!(!result.contains("<strong>"));
    }

    #[test]
    fn test_plain_disables_auto_link() {
        let result = parse("[plain]https://example.com[/plain]");
        // Should not auto-convert URL to link since stop_auto_link is true
        assert!(!result.contains("<a"));
        assert!(result.contains("https://example.com"));
    }

    #[test]
    fn test_plain_with_spoiler() {
        let result = parse("[plain][spoiler]Hidden[/spoiler][/plain]");
        // Should not render as HTML spoiler (which uses <details>)
        assert!(!result.contains("<details"));
        assert!(result.contains("[spoiler]Hidden[/spoiler]"));
    }

    #[test]
    fn test_plain_with_heading() {
        let result = parse("[plain][heading=1]Title[/heading][/plain]");
        assert!(!result.contains("<h1"));
        assert!(result.contains("[heading=1]Title[/heading]"));
    }

    #[test]
    fn test_plain_inside_other_tags() {
        let result = parse("[b]bold [plain][i]not italic[/i][/plain] bold[/b]");
        assert!(result.contains("<strong>"));
        assert!(!result.contains("<em>"));
        assert!(result.contains("[i]not italic[/i]"));
    }

    #[test]
    fn test_noparse_comprehensive() {
        let input = "[noparse][b]bold[/b] [url=test]link[/url] <script>test</script>[/noparse]";
        let result = parse(input);
        assert!(!result.contains("<strong>"));
        assert!(!result.contains("<a"));
        assert!(!result.contains("<script>"));
        assert!(result.contains("[b]bold[/b]"));
        assert!(result.contains("[url=test]link[/url]"));
        assert!(result.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_plain_case_insensitive() {
        let result = parse("[PLAIN][b]test[/b][/PLAIN]");
        assert!(result.contains("[b]test[/b]"));
        assert!(!result.contains("<strong>"));
    }

    #[test]
    fn test_plain_mixed_case_close() {
        let result = parse("[plain][b]test[/b][/PlAiN]");
        assert!(result.contains("[b]test[/b]"));
        assert!(!result.contains("<strong>"));
    }

    #[test]
    fn test_plain_unclosed() {
        // When [plain] is unclosed, the content after it gets parsed normally
        // This is because verbatim tags need explicit close tags to work
        let result = parse("[plain][b]bold");
        // The [b] tag will be parsed since [plain] wasn't properly closed
        // But [b] itself is unclosed, so it will be auto-closed
        assert!(result.contains("bold"));
    }

    #[test]
    fn test_plain_unclosed_with_following_content() {
        // When [plain] is unclosed, content after gets parsed as normal BBCode
        let result = parse("[plain]text [b]bold[/b]");
        // Without a [/plain], the content is parsed normally
        // so [b] will create <strong>
        assert!(result.contains("bold"));
    }

    #[test]
    fn test_plain_nested_in_plain() {
        // Inner [plain] should be treated as literal text, not as a tag
        let result = parse("[plain]outer [plain]inner[/plain] still outer[/plain]");
        // The first [/plain] closes the outer tag since inner [plain] is literal
        assert!(result.contains("[plain]inner"));
    }

    #[test]
    fn test_plain_inside_bold() {
        // Plain inside another tag should work properly
        let result = parse("[b]bold [plain][i]not italic[/i][/plain] bold[/b]");
        assert!(result.contains("<strong>"));
        assert!(!result.contains("<em>"));
        assert!(result.contains("[i]not italic[/i]"));
    }

    #[test]
    fn test_plain_mismatched_closing() {
        // [plain] closed by [/noparse] should work (same tag, different alias)
        let result = parse("[plain]content[/noparse]");
        assert!(result.contains("content"));
    }

    #[test]
    fn test_multiple_plain_tags() {
        // Multiple separate plain sections
        let result = parse("[plain]first[/plain] normal [plain]second[/plain]");
        assert!(result.contains("first"));
        assert!(result.contains("second"));
        assert!(result.contains("normal"));
    }

    #[test]
    fn test_plain_does_not_auto_close_on_other_tags() {
        // Plain should only close with [/plain], [/noparse], or [/nobbc]
        let result = parse("[plain][b]text[/b][/plain]");
        // [/b] should not close the [plain] tag
        assert!(result.contains("[/b]"));
    }

    // ============================================================================
    // Unicode Tests
    // ============================================================================

    #[test]
    fn test_unicode_japanese() {
        assert_eq!(parse("私は猫です"), "私は猫です");
    }

    #[test]
    fn test_unicode_cyrillic() {
        assert_eq!(parse("Привет мир"), "Привет мир");
    }

    #[test]
    fn test_unicode_emoji() {
        assert_eq!(parse("🔥🎉"), "🔥🎉");
    }

    #[test]
    fn test_unicode_in_tags() {
        let result = parse("[b]私は猫です[/b]");
        assert!(result.contains("<strong>私は猫です</strong>"));
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_empty_tag() {
        let result = parse("[b][/b]");
        assert_eq!(result, "<strong></strong>");
    }

    // ============================================================================
    // Auto-Closing and Nesting Tests
    // ============================================================================
    //
    // BBCode parser auto-closing behavior (XenForo-compatible):
    //
    // 1. UNCLOSED TAGS: Tags that aren't explicitly closed are auto-closed at
    //    the end of input in reverse order (innermost first).
    //    Example: [b][i]text → <strong><em>text</em></strong>
    //
    // 2. PROPER NESTING: When tags are closed in the correct order, they nest
    //    properly in the HTML output.
    //    Example: [b][i]text[/i][/b] → <strong><em>text</em></strong>
    //
    // 3. XENFORO AUTO-CLOSING: When tags are closed out of order, the parser
    //    auto-closes intervening tags first, then closes the requested tag.
    //    This maintains proper nesting structure.
    //    Example: [b][i]text[/b][/i] → <strong><em>text</em></strong>[/i]
    //    (The [/b] auto-closes [i], then closes [b]; [/i] has no match → literal)
    //    Example: [b][i][u]text[/b] → <strong><em><u>text</u></em></strong>
    //    (The [/b] auto-closes [u] and [i], then closes [b])
    //
    // 4. VERBATIM TAGS: Tags like [plain], [code], [icode] require explicit
    //    closing to work. If unclosed, their content is parsed normally.
    //    Example: [plain][b]text     → <strong>text</strong>
    //    Example: [plain][b]text[/plain] → [b]text
    //
    // ============================================================================

    #[test]
    fn test_unclosed_tag() {
        let result = parse("[b]Bold without close");
        assert!(result.contains("<strong>"));
        assert!(result.contains("Bold without close"));
    }

    #[test]
    fn test_unmatched_close() {
        let result = parse("text[/b]more");
        assert!(result.contains("[/b]"));
    }

    #[test]
    fn test_multiple_unclosed_nested_tags() {
        // Multiple nested tags, all unclosed
        let result = parse("[b][i][u]text");
        // Auto-closes in proper nested order
        assert_eq!(result, "<strong><em><u>text</u></em></strong>");
    }

    #[test]
    fn test_mismatched_closing_order() {
        // Tags closed in wrong order: [b][i]text[/b][/i]
        let result = parse("[b][i]text[/b][/i]");
        // XenForo behavior: [/b] auto-closes [i] first, then closes [b]
        // Result: <strong><em>text</em></strong>
        // The final [/i] has no matching open tag, so appears as literal text
        assert_eq!(result, "<strong><em>text</em></strong>[/i]");
    }

    #[test]
    fn test_intervening_tag_auto_closes() {
        // When closing [b], the intervening [i] should auto-close
        let parser = Parser::new();
        let doc = parser.parse("[b][i]t[/b]");

        println!("AST:");
        for (idx, node) in doc.nodes.iter().enumerate() {
            println!("  Node {}: {:?}", idx, node);
        }

        let renderer = Renderer::new();
        let result = renderer.render(&doc);
        println!("Rendered: '{}'", result);

        // XenForo behavior: should be <strong><em>t</em></strong>
        // The [i] auto-closes before [b] closes
        assert_eq!(result, "<strong><em>t</em></strong>");
    }

    #[test]
    fn test_mismatched_closing_order_triple() {
        // Three tags with mismatched closing: [b][i][u]text[/b][/u][/i]
        let result = parse("[b][i][u]text[/b][/u][/i]");
        // XenForo behavior: [/b] auto-closes [u] and [i], then closes [b]
        // Result: <strong><em><u>text</u></em></strong>
        // The [/u] and [/i] at the end have no matching tags
        assert_eq!(result, "<strong><em><u>text</u></em></strong>[/u][/i]");
    }

    #[test]
    fn test_xenforo_nested_auto_close_single() {
        // Single intervening tag
        let result = parse("[b][i]text[/b]");
        assert_eq!(result, "<strong><em>text</em></strong>");
    }

    #[test]
    fn test_xenforo_nested_auto_close_double() {
        // Two intervening tags
        let result = parse("[b][i][u]text[/b]");
        assert_eq!(result, "<strong><em><u>text</u></em></strong>");
    }

    #[test]
    fn test_xenforo_nested_auto_close_with_content_between() {
        // Content between tags
        let result = parse("[b]bold [i]italic[/b]");
        assert_eq!(result, "<strong>bold <em>italic</em></strong>");
    }

    #[test]
    fn test_xenforo_partial_overlap() {
        // [b][i]text1[/b] text2 [/i]
        // [/b] auto-closes [i], then [/i] has no match
        let result = parse("[b][i]text1[/b] text2 [/i]");
        assert_eq!(result, "<strong><em>text1</em></strong> text2 [/i]");
    }

    #[test]
    fn test_properly_nested_closing() {
        // Properly nested and closed
        let result = parse("[b][i][u]text[/u][/i][/b]");
        // Should produce perfectly nested HTML
        assert_eq!(result, "<strong><em><u>text</u></em></strong>");
    }

    #[test]
    fn test_partially_closed_nested() {
        // Some tags closed, some not
        let result = parse("[b][i]text[/i] more");
        assert!(result.contains("<strong>"));
        assert!(result.contains("<em>"));
        assert!(result.contains("text"));
        assert!(result.contains("more"));
    }

    #[test]
    fn test_deeply_nested_unclosed() {
        // Deep nesting without any close tags
        let result = parse("[b][i][u][s][color=red]deep");
        assert!(result.contains("deep"));
        // All tags should auto-close
    }

    #[test]
    fn test_interleaved_tags() {
        // Properly nested tags with proper closing
        let result = parse("[b][i]text[/i][/b]");
        assert!(result.contains("<strong>"));
        assert!(result.contains("<em>"));
        assert!(result.contains("text"));
    }

    #[test]
    fn test_unclosed_block_tag() {
        // Block-level tags like [quote] should also auto-close
        let result = parse("[quote]quoted text");
        assert!(result.contains("quoted text"));
    }

    #[test]
    fn test_mixed_block_and_inline_unclosed() {
        // Mix of block and inline tags, all unclosed
        let result = parse("[quote][b]bold quote");
        assert!(result.contains("bold quote"));
    }

    #[test]
    fn test_auto_close_preserves_content() {
        // Ensure content is never lost during auto-closing
        let result = parse("[b]start [i]middle [u]end");
        assert!(result.contains("start"));
        assert!(result.contains("middle"));
        assert!(result.contains("end"));
    }

    #[test]
    fn test_close_tag_after_unclosed_siblings() {
        // [b][i]text1[/b] text2 [i]text3
        // First [i] is unclosed, then [b] closes, then another [i]
        let result = parse("[b][i]text1[/b] text2 [i]text3");
        assert!(result.contains("text1"));
        assert!(result.contains("text2"));
        assert!(result.contains("text3"));
    }

    #[test]
    fn test_unknown_tag() {
        let result = parse("[unknown]text[/unknown]");
        assert!(result.contains("[unknown]"));
        assert!(result.contains("[/unknown]"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut input = String::new();
        for _ in 0..20 {
            input.push_str("[b]");
        }
        input.push_str("deep");
        for _ in 0..20 {
            input.push_str("[/b]");
        }

        let result = parse(&input);
        assert!(result.contains("deep"));
    }

    #[test]
    fn test_many_tags() {
        let input = "[b]a[/b]".repeat(100);
        let result = parse(&input);
        assert!(result.contains("<strong>a</strong>"));
    }

    #[test]
    fn test_mismatched_nesting() {
        // [b][i]text[/b][/i] - when tags are mismatched, parser handles gracefully
        // The parser will close unclosed tags and treat unmatched close tags as text
        let result = parse("[b][i]text[/b][/i]");
        // At minimum, some tags should be processed
        assert!(result.contains("text"));
        // One of the tags should render
        assert!(result.contains("<em>") || result.contains("<strong>"));
    }

    // ============================================================================
    // Complex Documents
    // ============================================================================

    #[test]
    fn test_forum_post() {
        let input = r#"[quote="Admin"]
Welcome to our forum!
[/quote]

Here are the rules:
[list=1]
[*]Be respectful
[*]No spam
[*]Have fun!
[/list]

For more info, visit [url=https://example.com]our website[/url].

[code=rust]
fn main() {
    println!("Hello, world!");
}
[/code]"#;

        let result = parse(input);

        assert!(result.contains("<blockquote"));
        assert!(result.contains("Admin"));
        assert!(result.contains("<ol"));
        // List items may contain linebreaks, so just check item content is present
        assert!(result.contains("Be respectful"));
        assert!(result.contains("No spam"));
        assert!(result.contains("Have fun!"));
        assert!(result.contains("href=\"https://example.com\""));
        assert!(result.contains("language-rust"));
    }

    #[test]
    fn test_signature() {
        let input = r#"[center]
[img]https://example.com/sig.png[/img]
[size=2][color=gray]Member since 2020[/color][/size]
[/center]"#;

        let result = parse(input);
        assert!(result.contains("text-align: center"));
        assert!(result.contains("<img"));
    }

    // ============================================================================
    // Security Tests
    // ============================================================================

    #[test]
    fn test_xss_in_text() {
        let result = parse("<script>alert('xss')</script>");
        assert!(!result.contains("<script>"));
        assert!(result.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_xss_in_url() {
        let result = parse("[url=javascript:alert('xss')]Click[/url]");
        // The javascript: URL should not appear in an href attribute
        assert!(!result.contains("href=\"javascript:"));
    }

    #[test]
    fn test_xss_in_img() {
        let result = parse("[img]javascript:alert('xss')[/img]");
        // The javascript: URL should not appear in a src attribute
        assert!(!result.contains("src=\"javascript:"));
        // The img tag should not be rendered at all
        assert!(!result.contains("<img"));
    }

    #[test]
    fn test_xss_in_color() {
        let result = parse("[color=red;onclick=alert('xss')]text[/color]");
        // Invalid color should be rejected
        assert!(result.contains("[color="));
    }

    #[test]
    fn test_xss_data_url() {
        let result = parse("[img]data:text/html,<script>alert('xss')</script>[/img]");
        // The data: URL should not appear in a src attribute
        assert!(!result.contains("src=\"data:"));
        // The img tag should not be rendered at all
        assert!(!result.contains("<img"));
    }

    // ============================================================================
    // Parser Config Tests
    // ============================================================================

    #[test]
    fn test_max_depth_limit() {
        let config = ParserConfig {
            max_depth: 3,
            ..Default::default()
        };

        let parser = Parser::with_config(config);
        let input = "[b][b][b][b][b]deep[/b][/b][/b][/b][/b]";
        let doc = parser.parse(input);

        // Should parse without panic
        let renderer = Renderer::new();
        let _result = renderer.render(&doc);
    }

    // ============================================================================
    // Renderer Config Tests
    // ============================================================================

    #[test]
    fn test_custom_class_prefix() {
        let render_config = RenderConfig {
            class_prefix: "forum".into(),
            ..Default::default()
        };

        let parser = Parser::new();
        let doc = parser.parse("[color=red]text[/color]");
        let renderer = Renderer::with_config(render_config);
        let result = renderer.render(&doc);

        assert!(result.contains("forum-color"));
    }

    #[test]
    fn test_target_blank() {
        let render_config = RenderConfig {
            open_links_in_new_tab: true,
            ..Default::default()
        };

        let parser = Parser::new();
        let doc = parser.parse("[url=https://example.com]Link[/url]");
        let renderer = Renderer::with_config(render_config);
        let result = renderer.render(&doc);

        assert!(result.contains("target=\"_blank\""));
    }

    #[test]
    fn test_nofollow_disabled() {
        let render_config = RenderConfig {
            nofollow_links: false,
            ..Default::default()
        };

        let parser = Parser::new();
        let doc = parser.parse("[url=https://example.com]Link[/url]");
        let renderer = Renderer::with_config(render_config);
        let result = renderer.render(&doc);

        assert!(!result.contains("rel=\"nofollow\""));
    }
}
