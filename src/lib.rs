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
