//! HTML renderer for BBCode AST.
//!
//! This module converts a parsed BBCode AST into HTML using zero-copy
//! techniques where possible. The `cow-utils` crate is used for efficient
//! string manipulation that avoids allocation when unnecessary.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

use crate::ast::{Document, Node, TagNode};
use crate::tags::TagRegistry;

/// Configuration for the HTML renderer.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// CSS class prefix for BBCode elements.
    pub class_prefix: Cow<'static, str>,

    /// Whether to add rel="nofollow" to links.
    pub nofollow_links: bool,

    /// Whether to add target="_blank" to links.
    pub open_links_in_new_tab: bool,

    /// Whether to sanitize text content (escape HTML).
    pub sanitize: bool,

    /// Whether to convert line breaks to <br>.
    pub convert_linebreaks: bool,

    /// Custom smilies/emoji mapping.
    pub smilies: HashMap<String, String>,

    /// Allowed URL schemes for links and images.
    pub allowed_schemes: Vec<String>,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            class_prefix: Cow::Borrowed("bbcode"),
            nofollow_links: true,
            open_links_in_new_tab: false,
            sanitize: true,
            convert_linebreaks: true,
            smilies: HashMap::new(),
            allowed_schemes: vec![
                "http".to_string(),
                "https".to_string(),
                "mailto".to_string(),
            ],
        }
    }
}

/// The HTML renderer.
pub struct Renderer {
    config: RenderConfig,
    registry: TagRegistry,
}

impl Renderer {
    /// Creates a new renderer with default settings.
    pub fn new() -> Self {
        Self {
            config: RenderConfig::default(),
            registry: TagRegistry::new(),
        }
    }

    /// Creates a new renderer with custom configuration.
    pub fn with_config(config: RenderConfig) -> Self {
        Self {
            config,
            registry: TagRegistry::new(),
        }
    }

    /// Renders a document to HTML.
    pub fn render(&self, doc: &Document) -> String {
        let mut output = String::new();
        for node in doc.iter() {
            self.render_node(node, &mut output);
        }
        output
    }

    /// Renders a single node to HTML.
    fn render_node(&self, node: &Node, output: &mut String) {
        match node {
            Node::Text(text) => {
                self.render_text(text, output);
            }
            Node::LineBreak => {
                if self.config.convert_linebreaks {
                    output.push_str("<br />");
                } else {
                    output.push('\n');
                }
            }
            Node::AutoUrl(url) => {
                self.render_auto_url(url, output);
            }
            Node::Tag(tag) => {
                self.render_tag(tag, output);
            }
        }
    }

    /// Renders text content with HTML escaping.
    fn render_text(&self, text: &str, output: &mut String) {
        if self.config.sanitize {
            output.push_str(&escape_html(text));
        } else {
            output.push_str(text);
        }
    }

    /// Renders an auto-detected URL.
    fn render_auto_url(&self, url: &str, output: &mut String) {
        let safe_url = escape_html(url);
        write!(
            output,
            "<a class=\"{}-url\" href=\"{}\"{}{}>{}</a>",
            self.config.class_prefix,
            safe_url,
            if self.config.nofollow_links {
                " rel=\"nofollow\""
            } else {
                ""
            },
            if self.config.open_links_in_new_tab {
                " target=\"_blank\""
            } else {
                ""
            },
            safe_url
        )
        .unwrap();
    }

    /// Renders a tag node.
    fn render_tag(&self, tag: &TagNode, output: &mut String) {
        // If broken, render as raw text
        if tag.broken {
            self.render_text(&tag.raw_open, output);
            for child in &tag.children {
                self.render_node(child, output);
            }
            if !tag.raw_close.is_empty() {
                self.render_text(&tag.raw_close, output);
            }
            return;
        }

        // Look up tag definition
        let _tag_def = self.registry.get(&tag.name);

        match &*tag.name {
            // Basic formatting
            "b" | "bold" => self.render_simple_tag(tag, "strong", output),
            "i" | "italic" => self.render_simple_tag(tag, "em", output),
            "u" | "underline" => self.render_simple_tag(tag, "u", output),
            "s" | "strike" | "strikethrough" => self.render_simple_tag(tag, "s", output),
            "sub" => self.render_simple_tag(tag, "sub", output),
            "sup" => self.render_simple_tag(tag, "sup", output),

            // Color and font
            "color" | "colour" => self.render_color(tag, output),
            "font" => self.render_font(tag, output),
            "size" => self.render_size(tag, output),

            // Links
            "url" | "link" => self.render_url(tag, output),
            "email" | "mail" => self.render_email(tag, output),

            // Images
            "img" | "image" => self.render_img(tag, output),

            // Block elements
            "quote" => self.render_quote(tag, output),
            "code" => self.render_code(tag, output),
            "icode" | "c" | "inline" => self.render_icode(tag, output),
            "php" => self.render_code_with_lang(tag, "php", output),
            "html" => self.render_code_with_lang(tag, "html", output),
            "plain" | "noparse" | "nobbc" => self.render_plain(tag, output),

            // Lists
            "list" => self.render_list(tag, output),
            "*" | "li" => self.render_list_item(tag, output),

            // Alignment
            "left" => self.render_align(tag, "left", output),
            "center" => self.render_align(tag, "center", output),
            "right" => self.render_align(tag, "right", output),
            "justify" => self.render_align(tag, "justify", output),
            "indent" => self.render_indent(tag, output),

            // Headings
            "heading" | "h" => self.render_heading(tag, output),

            // Special
            "hr" => output.push_str("<hr />"),
            "br" => output.push_str("<br />"),
            "spoiler" => self.render_spoiler(tag, output),
            "ispoiler" => self.render_ispoiler(tag, output),
            "user" | "member" => self.render_user(tag, output),

            // Tables
            "table" => self.render_table(tag, output),
            "tr" => self.render_table_row(tag, output),
            "th" => self.render_table_header(tag, output),
            "td" => self.render_table_cell(tag, output),

            // Unknown tag - render as text
            _ => {
                self.render_text(&tag.raw_open, output);
                for child in &tag.children {
                    self.render_node(child, output);
                }
                if !tag.raw_close.is_empty() {
                    self.render_text(&tag.raw_close, output);
                }
            }
        }
    }

    /// Renders a simple tag like <strong>, <em>, etc.
    fn render_simple_tag(&self, tag: &TagNode, html_tag: &str, output: &mut String) {
        write!(output, "<{}>", html_tag).unwrap();
        self.render_children(tag, output);
        write!(output, "</{}>", html_tag).unwrap();
    }

    /// Renders all children of a tag.
    fn render_children(&self, tag: &TagNode, output: &mut String) {
        for child in &tag.children {
            self.render_node(child, output);
        }
    }

    /// Gets the inner text of a tag (for verbatim content).
    fn get_inner_text<'a>(&self, tag: &TagNode<'a>) -> Cow<'a, str> {
        tag.inner_text()
    }

    // ============================================================================
    // Specific tag renderers
    // ============================================================================

    fn render_color(&self, tag: &TagNode, output: &mut String) {
        if let Some(color) = tag.option.as_scalar() {
            if is_valid_color(color) {
                write!(
                    output,
                    "<span class=\"{}-color\" style=\"color: {};\">",
                    self.config.class_prefix,
                    escape_html(color)
                )
                .unwrap();
                self.render_children(tag, output);
                output.push_str("</span>");
                return;
            }
        }
        // Invalid color, render as text
        self.render_as_text(tag, output);
    }

    fn render_font(&self, tag: &TagNode, output: &mut String) {
        if let Some(font) = tag.option.as_scalar() {
            if is_valid_font(font) {
                write!(
                    output,
                    "<span class=\"{}-font\" style=\"font-family: {};\">",
                    self.config.class_prefix,
                    escape_html(font)
                )
                .unwrap();
                self.render_children(tag, output);
                output.push_str("</span>");
                return;
            }
        }
        self.render_as_text(tag, output);
    }

    fn render_size(&self, tag: &TagNode, output: &mut String) {
        if let Some(size) = tag.option.as_scalar() {
            if let Some(css_size) = parse_size(size) {
                write!(
                    output,
                    "<span class=\"{}-size\" style=\"font-size: {};\">",
                    self.config.class_prefix, css_size
                )
                .unwrap();
                self.render_children(tag, output);
                output.push_str("</span>");
                return;
            }
        }
        self.render_as_text(tag, output);
    }

    fn render_url(&self, tag: &TagNode, output: &mut String) {
        // URL can be in option or content
        let url = if let Some(opt) = tag.option.as_scalar() {
            opt.clone()
        } else {
            tag.inner_text()
        };

        if !is_valid_url(&url, &self.config.allowed_schemes) {
            self.render_as_text(tag, output);
            return;
        }

        write!(
            output,
            "<a class=\"{}-url\" href=\"{}\"",
            self.config.class_prefix,
            escape_html(&url)
        )
        .unwrap();

        if self.config.nofollow_links {
            output.push_str(" rel=\"nofollow\"");
        }
        if self.config.open_links_in_new_tab {
            output.push_str(" target=\"_blank\"");
        }
        output.push('>');

        if tag.option.is_scalar() {
            self.render_children(tag, output);
        } else {
            // URL is the content, display it
            self.render_text(&url, output);
        }

        output.push_str("</a>");
    }

    fn render_email(&self, tag: &TagNode, output: &mut String) {
        let email = if let Some(opt) = tag.option.as_scalar() {
            opt.clone()
        } else {
            tag.inner_text()
        };

        // Email validation - must contain @ and no dangerous characters
        // Block quotes, angle brackets, and event handler patterns
        if !email.contains('@')
            || email.contains('<')
            || email.contains('>')
            || email.contains('"')
            || email.contains('\'')
        {
            self.render_as_text(tag, output);
            return;
        }

        // Also check for event handler injection in email
        let lower = email.to_ascii_lowercase();
        if lower.contains("onclick=")
            || lower.contains("onerror=")
            || lower.contains("onmouseover=")
            || lower.contains("onload=")
            || lower.contains("onfocus=")
        {
            self.render_as_text(tag, output);
            return;
        }

        write!(
            output,
            "<a class=\"{}-email\" href=\"mailto:{}\">",
            self.config.class_prefix,
            escape_html(&email)
        )
        .unwrap();

        if tag.option.is_scalar() {
            self.render_children(tag, output);
        } else {
            self.render_text(&email, output);
        }

        output.push_str("</a>");
    }

    fn render_img(&self, tag: &TagNode, output: &mut String) {
        let url = tag.inner_text();

        if url.is_empty() || !is_valid_url(&url, &self.config.allowed_schemes) {
            self.render_as_text(tag, output);
            return;
        }

        write!(
            output,
            "<img class=\"{}-img\" src=\"{}\"",
            self.config.class_prefix,
            escape_html(&url)
        )
        .unwrap();

        // Handle dimensions from option
        if let Some(opt) = tag.option.as_scalar() {
            if let Some((width, height)) = parse_dimensions(opt) {
                write!(output, " width=\"{}\" height=\"{}\"", width, height).unwrap();
            }
        } else if let Some(map) = tag.option.as_map() {
            if let Some(width) = map.get("width") {
                write!(output, " width=\"{}\"", escape_html(width)).unwrap();
            }
            if let Some(height) = map.get("height") {
                write!(output, " height=\"{}\"", escape_html(height)).unwrap();
            }
            if let Some(alt) = map.get("alt") {
                write!(output, " alt=\"{}\"", escape_html(alt)).unwrap();
            }
        }

        output.push_str(" />");
    }

    fn render_quote(&self, tag: &TagNode, output: &mut String) {
        write!(
            output,
            "<blockquote class=\"{}-quote\"",
            self.config.class_prefix
        )
        .unwrap();

        // Handle quote author
        if let Some(author) = tag.option.as_scalar() {
            write!(output, " data-author=\"{}\"", escape_html(author)).unwrap();
        }

        output.push('>');

        // If author is present, add a header
        if let Some(author) = tag.option.as_scalar() {
            write!(
                output,
                "<div class=\"{}-quote-author\">{} wrote:</div>",
                self.config.class_prefix,
                escape_html(author)
            )
            .unwrap();
        }

        write!(
            output,
            "<div class=\"{}-quote-content\">",
            self.config.class_prefix
        )
        .unwrap();
        self.render_children(tag, output);
        output.push_str("</div></blockquote>");
    }

    fn render_code(&self, tag: &TagNode, output: &mut String) {
        let lang = tag.option.as_scalar();
        let content = self.get_inner_text(tag);

        write!(output, "<pre class=\"{}-code\"", self.config.class_prefix).unwrap();

        if let Some(lang) = lang {
            write!(output, " data-language=\"{}\"", escape_html(lang)).unwrap();
        }

        output.push_str("><code");

        if let Some(lang) = lang {
            write!(output, " class=\"language-{}\"", escape_html(lang)).unwrap();
        }

        output.push('>');
        output.push_str(&escape_html(&content));
        output.push_str("</code></pre>");
    }

    fn render_code_with_lang(&self, tag: &TagNode, lang: &str, output: &mut String) {
        let content = self.get_inner_text(tag);

        write!(
            output,
            "<pre class=\"{}-code\" data-language=\"{}\"><code class=\"language-{}\">",
            self.config.class_prefix, lang, lang
        )
        .unwrap();
        output.push_str(&escape_html(&content));
        output.push_str("</code></pre>");
    }

    fn render_icode(&self, tag: &TagNode, output: &mut String) {
        let content = self.get_inner_text(tag);
        write!(
            output,
            "<code class=\"{}-icode\">",
            self.config.class_prefix
        )
        .unwrap();
        output.push_str(&escape_html(&content));
        output.push_str("</code>");
    }

    fn render_plain(&self, tag: &TagNode, output: &mut String) {
        let content = self.get_inner_text(tag);
        output.push_str(&escape_html(&content));
    }

    fn render_list(&self, tag: &TagNode, output: &mut String) {
        let is_ordered = tag
            .option
            .as_scalar()
            .map(|s| s == "1" || s == "a" || s == "A" || s == "i" || s == "I")
            .unwrap_or(false);

        let list_tag = if is_ordered { "ol" } else { "ul" };

        write!(
            output,
            "<{} class=\"{}-list\"",
            list_tag, self.config.class_prefix
        )
        .unwrap();

        // Handle list type
        if let Some(list_type) = tag.option.as_scalar() {
            match list_type.as_ref() {
                "1" => output.push_str(" type=\"1\""),
                "a" => output.push_str(" type=\"a\""),
                "A" => output.push_str(" type=\"A\""),
                "i" => output.push_str(" type=\"i\""),
                "I" => output.push_str(" type=\"I\""),
                "disc" => output.push_str(" style=\"list-style-type: disc;\""),
                "circle" => output.push_str(" style=\"list-style-type: circle;\""),
                "square" => output.push_str(" style=\"list-style-type: square;\""),
                _ => {}
            }
        }

        output.push('>');
        self.render_children(tag, output);
        write!(output, "</{}>", list_tag).unwrap();
    }

    fn render_list_item(&self, tag: &TagNode, output: &mut String) {
        output.push_str("<li>");
        self.render_children(tag, output);
        output.push_str("</li>");
    }

    fn render_align(&self, tag: &TagNode, align: &str, output: &mut String) {
        write!(
            output,
            "<div class=\"{}-align\" style=\"text-align: {};\">",
            self.config.class_prefix, align
        )
        .unwrap();
        self.render_children(tag, output);
        output.push_str("</div>");
    }

    fn render_indent(&self, tag: &TagNode, output: &mut String) {
        let level: u8 = tag
            .option
            .as_scalar()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1)
            .min(5);

        let margin = level as u32 * 20;

        write!(
            output,
            "<div class=\"{}-indent\" style=\"margin-left: {}px;\">",
            self.config.class_prefix, margin
        )
        .unwrap();
        self.render_children(tag, output);
        output.push_str("</div>");
    }

    fn render_heading(&self, tag: &TagNode, output: &mut String) {
        let level: u8 = tag
            .option
            .as_scalar()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1)
            .clamp(1, 6);

        // XenForo maps 1->h2, 2->h3, 3->h4
        let html_level = (level + 1).min(6);

        write!(
            output,
            "<h{} class=\"{}-heading\">",
            html_level, self.config.class_prefix
        )
        .unwrap();
        self.render_children(tag, output);
        write!(output, "</h{}>", html_level).unwrap();
    }

    fn render_spoiler(&self, tag: &TagNode, output: &mut String) {
        write!(
            output,
            "<details class=\"{}-spoiler\"><summary>",
            self.config.class_prefix
        )
        .unwrap();

        if let Some(title) = tag.option.as_scalar() {
            output.push_str(&escape_html(title));
        } else {
            output.push_str("Spoiler");
        }

        output.push_str("</summary><div class=\"spoiler-content\">");
        self.render_children(tag, output);
        output.push_str("</div></details>");
    }

    fn render_ispoiler(&self, tag: &TagNode, output: &mut String) {
        write!(
            output,
            "<span class=\"{}-ispoiler\" onclick=\"this.classList.toggle('revealed')\">",
            self.config.class_prefix
        )
        .unwrap();
        self.render_children(tag, output);
        output.push_str("</span>");
    }

    fn render_user(&self, tag: &TagNode, output: &mut String) {
        let user_id = tag.option.as_scalar();
        let username = tag.inner_text();

        if let Some(id) = user_id {
            write!(
                output,
                "<a class=\"{}-user\" data-user-id=\"{}\" href=\"#\">@{}</a>",
                self.config.class_prefix,
                escape_html(id),
                escape_html(&username)
            )
            .unwrap();
        } else {
            write!(
                output,
                "<span class=\"{}-user\">@{}</span>",
                self.config.class_prefix,
                escape_html(&username)
            )
            .unwrap();
        }
    }

    fn render_table(&self, tag: &TagNode, output: &mut String) {
        write!(
            output,
            "<table class=\"{}-table\"",
            self.config.class_prefix
        )
        .unwrap();

        if let Some(map) = tag.option.as_map() {
            if let Some(width) = map.get("width") {
                write!(output, " style=\"width: {};\"", escape_html(width)).unwrap();
            }
        }

        output.push('>');
        self.render_children(tag, output);
        output.push_str("</table>");
    }

    fn render_table_row(&self, tag: &TagNode, output: &mut String) {
        output.push_str("<tr>");
        self.render_children(tag, output);
        output.push_str("</tr>");
    }

    fn render_table_header(&self, tag: &TagNode, output: &mut String) {
        output.push_str("<th");

        if let Some(map) = tag.option.as_map() {
            if let Some(width) = map.get("width") {
                write!(output, " style=\"width: {};\"", escape_html(width)).unwrap();
            }
        }

        output.push('>');
        self.render_children(tag, output);
        output.push_str("</th>");
    }

    fn render_table_cell(&self, tag: &TagNode, output: &mut String) {
        output.push_str("<td");

        if let Some(map) = tag.option.as_map() {
            if let Some(width) = map.get("width") {
                write!(output, " style=\"width: {};\"", escape_html(width)).unwrap();
            }
        }

        output.push('>');
        self.render_children(tag, output);
        output.push_str("</td>");
    }

    /// Renders a tag as plain text (for invalid/broken tags).
    fn render_as_text(&self, tag: &TagNode, output: &mut String) {
        self.render_text(&tag.raw_open, output);
        for child in &tag.children {
            self.render_node(child, output);
        }
        if !tag.raw_close.is_empty() {
            self.render_text(&tag.raw_close, output);
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Escapes HTML special characters.
///
/// This uses cow-utils for efficient zero-copy when no escaping is needed.
pub fn escape_html(input: &str) -> Cow<'_, str> {
    // Check if any escaping is needed
    if !input
        .bytes()
        .any(|b| matches!(b, b'<' | b'>' | b'&' | b'"' | b'\''))
    {
        return Cow::Borrowed(input);
    }

    // Need to escape
    let mut result = String::with_capacity(input.len() + input.len() / 4);
    for c in input.chars() {
        match c {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Validates a color value.
fn is_valid_color(color: &str) -> bool {
    // Hex color
    if color.starts_with('#') {
        let hex = &color[1..];
        return (hex.len() == 3 || hex.len() == 6) && hex.chars().all(|c| c.is_ascii_hexdigit());
    }

    // RGB/RGBA
    if color.starts_with("rgb") {
        return color
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '(' | ')' | ',' | ' ' | '.'));
    }

    // Named color (simplified validation)
    VALID_COLORS.contains(&color.to_ascii_lowercase().as_str())
}

/// Validates a font family name.
fn is_valid_font(font: &str) -> bool {
    // Only allow alphanumeric, spaces, and hyphens
    font.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_')
}

/// Parses a size value into CSS.
fn parse_size(size: &str) -> Option<String> {
    // Size can be:
    // - A number 1-7 (XenForo style)
    // - A percentage (phpBB style: 50-200)
    // - A pixel value like "12px"

    if let Ok(n) = size.parse::<u8>() {
        if (1..=7).contains(&n) {
            // XenForo sizes: 1=9px, 2=10px, 3=12px, 4=15px, 5=18px, 6=22px, 7=26px
            let px = match n {
                1 => 9,
                2 => 10,
                3 => 12,
                4 => 15,
                5 => 18,
                6 => 22,
                7 => 26,
                _ => 12,
            };
            return Some(format!("{}px", px));
        } else if (8..=200).contains(&n) {
            // Pixel value or percentage
            if n <= 100 {
                return Some(format!("{}%", n));
            } else {
                return Some(format!("{}px", n.min(36)));
            }
        }
    }

    if size.ends_with("px") {
        if let Ok(n) = size[..size.len() - 2].parse::<u8>() {
            if (8..=36).contains(&n) {
                return Some(size.to_string());
            }
        }
    }

    if size.ends_with('%') {
        if let Ok(n) = size[..size.len() - 1].parse::<u16>() {
            if (50..=200).contains(&n) {
                return Some(size.to_string());
            }
        }
    }

    None
}

/// Validates a URL for safe rendering.
/// Rejects dangerous protocols and attribute-breaking characters.
fn is_valid_url(url: &str, allowed_schemes: &[String]) -> bool {
    // Must not be empty
    if url.is_empty() {
        return false;
    }

    // Check for dangerous patterns in lowercase
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:")
    {
        return false;
    }

    // Block attribute injection attempts - quotes and angle brackets
    // These could break out of HTML attributes
    if url.contains('"') || url.contains('\'') || url.contains('<') || url.contains('>') {
        return false;
    }

    // Block event handler injection attempts (case-insensitive)
    // Check for patterns like: onclick=, onerror=, onmouseover=, etc.
    let lower_for_events = lower.replace(char::is_whitespace, "");
    if lower_for_events.contains("onclick=")
        || lower_for_events.contains("onerror=")
        || lower_for_events.contains("onmouseover=")
        || lower_for_events.contains("onload=")
        || lower_for_events.contains("onfocus=")
        || lower_for_events.contains("onblur=")
        || lower_for_events.contains("onmousedown=")
        || lower_for_events.contains("onmouseup=")
        || lower_for_events.contains("onmouseenter=")
        || lower_for_events.contains("onmouseleave=")
        || lower_for_events.contains("onkeydown=")
        || lower_for_events.contains("onkeyup=")
        || lower_for_events.contains("onkeypress=")
        || lower_for_events.contains("onchange=")
        || lower_for_events.contains("oninput=")
        || lower_for_events.contains("onsubmit=")
    {
        return false;
    }

    // Check scheme
    if let Some(colon_pos) = url.find(':') {
        let scheme = &url[..colon_pos].to_ascii_lowercase();
        if !allowed_schemes.iter().any(|s| s == scheme) {
            return false;
        }
    }

    true
}

/// Parses image dimensions from option like "100x200" or "100".
fn parse_dimensions(opt: &str) -> Option<(u32, u32)> {
    if let Some(x_pos) = opt.find(|c| c == 'x' || c == 'X') {
        let width: u32 = opt[..x_pos].parse().ok()?;
        let height: u32 = opt[x_pos + 1..].parse().ok()?;
        Some((width.min(2000), height.min(2000)))
    } else if let Ok(size) = opt.parse::<u32>() {
        Some((size.min(2000), size.min(2000)))
    } else {
        None
    }
}

/// List of valid CSS color names.
static VALID_COLORS: &[&str] = &[
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgrey",
    "darkgreen",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "grey",
    "green",
    "greenyellow",
    "honeydew",
    "hotpink",
    "indianred",
    "indigo",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgrey",
    "lightgreen",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "rebeccapurple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "transparent",
    "turquoise",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn render(input: &str) -> String {
        let parser = Parser::new();
        let doc = parser.parse(input);
        let renderer = Renderer::new();
        renderer.render(&doc)
    }

    // ==================== Basic Rendering Tests ====================

    #[test]
    fn render_plain_text() {
        assert_eq!(render("Hello, world!"), "Hello, world!");
    }

    #[test]
    fn render_text_with_html_entities() {
        assert_eq!(
            render("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn render_quotes_escaped() {
        assert_eq!(render("He said \"Hello\""), "He said &quot;Hello&quot;");
    }

    #[test]
    fn render_ampersand() {
        assert_eq!(render("A & B"), "A &amp; B");
    }

    // ==================== Basic Formatting Tests ====================

    #[test]
    fn render_bold() {
        assert_eq!(render("[b]Bold[/b]"), "<strong>Bold</strong>");
    }

    #[test]
    fn render_italic() {
        assert_eq!(render("[i]Italic[/i]"), "<em>Italic</em>");
    }

    #[test]
    fn render_underline() {
        assert_eq!(render("[u]Underline[/u]"), "<u>Underline</u>");
    }

    #[test]
    fn render_strikethrough() {
        assert_eq!(render("[s]Strike[/s]"), "<s>Strike</s>");
    }

    #[test]
    fn render_nested_formatting() {
        assert_eq!(
            render("[b][i]Bold Italic[/i][/b]"),
            "<strong><em>Bold Italic</em></strong>"
        );
    }

    #[test]
    fn render_sub_sup() {
        assert_eq!(render("H[sub]2[/sub]O"), "H<sub>2</sub>O");
        assert_eq!(render("x[sup]2[/sup]"), "x<sup>2</sup>");
    }

    // ==================== Color Tests ====================

    #[test]
    fn render_color_named() {
        let result = render("[color=red]Red text[/color]");
        assert!(result.contains("color: red;"));
        assert!(result.contains("Red text"));
    }

    #[test]
    fn render_color_hex() {
        let result = render("[color=#ff0000]Red text[/color]");
        assert!(result.contains("color: #ff0000;"));
    }

    #[test]
    fn render_color_hex_short() {
        let result = render("[color=#f00]Red text[/color]");
        assert!(result.contains("color: #f00;"));
    }

    #[test]
    fn render_color_invalid() {
        let result = render("[color=notacolor]text[/color]");
        // Should render as raw text
        assert!(result.contains("[color=notacolor]"));
    }

    // ==================== Size Tests ====================

    #[test]
    fn render_size_numeric() {
        let result = render("[size=4]Large[/size]");
        assert!(result.contains("font-size: 15px;"));
    }

    #[test]
    fn render_size_pixels() {
        let result = render("[size=20px]Large[/size]");
        assert!(result.contains("font-size: 20px;"));
    }

    #[test]
    fn render_size_percent() {
        let result = render("[size=150%]Large[/size]");
        assert!(result.contains("font-size: 150%;"));
    }

    // ==================== URL Tests ====================

    #[test]
    fn render_url_with_option() {
        let result = render("[url=https://example.com]Click here[/url]");
        assert!(result.contains("href=\"https://example.com\""));
        assert!(result.contains("Click here"));
        assert!(result.contains("rel=\"nofollow\""));
    }

    #[test]
    fn render_url_without_option() {
        let result = render("[url]https://example.com[/url]");
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn render_url_xss_prevention() {
        let result = render("[url=javascript:alert('xss')]Click[/url]");
        // Should be rendered as text, not as link
        assert!(!result.contains("href=\"javascript"));
    }

    #[test]
    fn render_auto_url() {
        let result = render("Visit https://example.com today!");
        assert!(result.contains("href=\"https://example.com\""));
    }

    // ==================== Email Tests ====================

    #[test]
    fn render_email_with_option() {
        let result = render("[email=test@example.com]Contact[/email]");
        assert!(result.contains("href=\"mailto:test@example.com\""));
        assert!(result.contains("Contact"));
    }

    #[test]
    fn render_email_without_option() {
        let result = render("[email]test@example.com[/email]");
        assert!(result.contains("href=\"mailto:test@example.com\""));
    }

    // ==================== Image Tests ====================

    #[test]
    fn render_img() {
        let result = render("[img]https://example.com/image.png[/img]");
        assert!(result.contains("src=\"https://example.com/image.png\""));
        assert!(result.contains("<img"));
    }

    #[test]
    fn render_img_with_dimensions() {
        let result = render("[img=100x200]https://example.com/image.png[/img]");
        assert!(result.contains("width=\"100\""));
        assert!(result.contains("height=\"200\""));
    }

    #[test]
    fn render_img_xss_prevention() {
        let result = render("[img]javascript:alert('xss')[/img]");
        assert!(!result.contains("<img"));
    }

    // ==================== Quote Tests ====================

    #[test]
    fn render_quote() {
        let result = render("[quote]Quoted text[/quote]");
        assert!(result.contains("<blockquote"));
        assert!(result.contains("Quoted text"));
    }

    #[test]
    fn render_quote_with_author() {
        let result = render("[quote=\"John\"]Quoted text[/quote]");
        assert!(result.contains("<blockquote"));
        assert!(result.contains("John wrote:"));
        assert!(result.contains("Quoted text"));
    }

    // ==================== Code Tests ====================

    #[test]
    fn render_code() {
        let result = render("[code]function test() {}[/code]");
        assert!(result.contains("<pre"));
        assert!(result.contains("<code"));
        assert!(result.contains("function test() {}"));
    }

    #[test]
    fn render_code_with_language() {
        let result = render("[code=javascript]function test() {}[/code]");
        assert!(result.contains("data-language=\"javascript\""));
        assert!(result.contains("language-javascript"));
    }

    #[test]
    fn render_code_escapes_html() {
        let result = render("[code]<script>alert('xss')</script>[/code]");
        assert!(result.contains("&lt;script&gt;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn render_icode() {
        let result = render("Use [icode]console.log()[/icode] to debug");
        assert!(result.contains("<code"));
        assert!(result.contains("console.log()"));
    }

    // ==================== List Tests ====================

    #[test]
    fn render_unordered_list() {
        let result = render("[list][*]One[*]Two[/list]");
        assert!(result.contains("<ul"));
        assert!(result.contains("<li>One</li>"));
        assert!(result.contains("<li>Two</li>"));
    }

    #[test]
    fn render_ordered_list() {
        let result = render("[list=1][*]One[*]Two[/list]");
        assert!(result.contains("<ol"));
        assert!(result.contains("type=\"1\""));
    }

    #[test]
    fn render_list_alpha() {
        let result = render("[list=a][*]One[*]Two[/list]");
        assert!(result.contains("type=\"a\""));
    }

    // ==================== Alignment Tests ====================

    #[test]
    fn render_center() {
        let result = render("[center]Centered[/center]");
        assert!(result.contains("text-align: center;"));
    }

    #[test]
    fn render_right() {
        let result = render("[right]Right aligned[/right]");
        assert!(result.contains("text-align: right;"));
    }

    // ==================== Heading Tests ====================

    #[test]
    fn render_heading() {
        let result = render("[heading=1]Title[/heading]");
        assert!(result.contains("<h2"));
        assert!(result.contains("Title"));
    }

    #[test]
    fn render_heading_level_3() {
        let result = render("[heading=3]Subtitle[/heading]");
        assert!(result.contains("<h4"));
    }

    // ==================== Spoiler Tests ====================

    #[test]
    fn render_spoiler() {
        let result = render("[spoiler]Hidden content[/spoiler]");
        assert!(result.contains("<details"));
        assert!(result.contains("<summary>"));
        assert!(result.contains("Hidden content"));
    }

    #[test]
    fn render_spoiler_with_title() {
        let result = render("[spoiler=Click to reveal]Hidden[/spoiler]");
        assert!(result.contains("Click to reveal"));
    }

    #[test]
    fn render_ispoiler() {
        let result = render("This is [ispoiler]hidden[/ispoiler] text");
        assert!(result.contains("bbcode-ispoiler"));
        assert!(result.contains("hidden"));
    }

    // ==================== Table Tests ====================

    #[test]
    fn render_table() {
        let result = render("[table][tr][td]Cell[/td][/tr][/table]");
        assert!(result.contains("<table"));
        assert!(result.contains("<tr>"));
        assert!(result.contains("<td>"));
        assert!(result.contains("Cell"));
    }

    // ==================== Self-Closing Tag Tests ====================

    #[test]
    fn render_hr() {
        let result = render("Before[hr]After");
        assert!(result.contains("<hr />"));
    }

    // ==================== Line Break Tests ====================

    #[test]
    fn render_linebreaks() {
        let result = render("Line 1\nLine 2");
        assert!(result.contains("<br />"));
    }

    // ==================== Unknown Tag Tests ====================

    #[test]
    fn render_unknown_tag() {
        let result = render("[unknown]text[/unknown]");
        assert!(result.contains("[unknown]"));
        assert!(result.contains("[/unknown]"));
    }

    // ==================== Helper Function Tests ====================

    #[test]
    fn escape_html_no_special_chars() {
        let result = escape_html("Hello world");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(&*result, "Hello world");
    }

    #[test]
    fn escape_html_with_special_chars() {
        let result = escape_html("<script>&\"'");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(&*result, "&lt;script&gt;&amp;&quot;&#x27;");
    }

    #[test]
    fn is_valid_color_test() {
        assert!(is_valid_color("red"));
        assert!(is_valid_color("blue"));
        assert!(is_valid_color("#ff0000"));
        assert!(is_valid_color("#f00"));
        assert!(is_valid_color("rgb(255, 0, 0)"));

        assert!(!is_valid_color("notacolor"));
        assert!(!is_valid_color("#gggggg"));
        assert!(!is_valid_color(""));
    }

    #[test]
    fn is_valid_font_test() {
        assert!(is_valid_font("Arial"));
        assert!(is_valid_font("Times New Roman"));
        assert!(is_valid_font("courier-new"));

        assert!(!is_valid_font("font<script>"));
        assert!(!is_valid_font("font;color:red"));
    }

    #[test]
    fn parse_size_test() {
        assert_eq!(parse_size("4"), Some("15px".to_string()));
        assert_eq!(parse_size("20px"), Some("20px".to_string()));
        assert_eq!(parse_size("150%"), Some("150%".to_string()));

        assert!(parse_size("999px").is_none());
        assert!(parse_size("abc").is_none());
    }

    #[test]
    fn is_valid_url_test() {
        let schemes = vec!["http".to_string(), "https".to_string()];

        assert!(is_valid_url("https://example.com", &schemes));
        assert!(is_valid_url("http://example.com", &schemes));

        assert!(!is_valid_url("javascript:alert('xss')", &schemes));
        assert!(!is_valid_url("data:text/html,<script>", &schemes));
        assert!(!is_valid_url("vbscript:alert", &schemes));
    }

    #[test]
    fn parse_dimensions_test() {
        assert_eq!(parse_dimensions("100x200"), Some((100, 200)));
        assert_eq!(parse_dimensions("100X200"), Some((100, 200)));
        assert_eq!(parse_dimensions("100"), Some((100, 100)));

        assert!(parse_dimensions("abc").is_none());
        assert!(parse_dimensions("100x").is_none());
    }

    // ==================== Complex Rendering Tests ====================

    #[test]
    fn render_complex_document() {
        let input = r#"[quote="Admin"]
Hello [b]everyone[/b]!

Check the [url=https://example.com]documentation[/url].

[code=rust]
fn main() {
    println!("Hello!");
}
[/code]
[/quote]"#;

        let result = render(input);

        assert!(result.contains("<blockquote"));
        assert!(result.contains("<strong>everyone</strong>"));
        assert!(result.contains("href=\"https://example.com\""));
        assert!(result.contains("language-rust"));
    }

    #[test]
    fn render_preserves_order() {
        let result = render("A[b]B[/b]C[i]D[/i]E");
        // Check the content appears in order
        let a_pos = result.find('A').unwrap();
        let b_pos = result.find('B').unwrap();
        let c_pos = result.find('C').unwrap();
        let d_pos = result.find('D').unwrap();
        let e_pos = result.find('E').unwrap();

        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
        assert!(c_pos < d_pos);
        assert!(d_pos < e_pos);
    }
}
