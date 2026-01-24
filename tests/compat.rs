//! Compatibility tests for phpBB and XenForo BBCode syntax.
//!
//! These tests verify that common BBCode patterns from both platforms
//! are correctly parsed and rendered.

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
