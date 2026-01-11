//! # Custom Attachment Handler Example
//!
//! This example demonstrates how to implement a custom BBCode tag handler for
//! attachments, similar to XenForo's `[attach]` tag.
//!
//! ## Features
//!
//! - Custom tag registration with `CustomTagDef`
//! - Pre-fetching attachment data before rendering (batch DB simulation)
//! - XenForo-compatible option parsing (`=full`, `type=full`, `width=`, `height=`)
//! - Support for images, videos, and audio files
//! - Proper fallback for missing/invalid attachments
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example attach
//! ```

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::{Arc, Mutex};

use bbcode::{
    escape_html, CustomTagDef, CustomTagHandler, Parser, RenderConfig, RenderContext, Renderer,
    TagNode, TagType,
};

// ============================================================================
// Simulated Database Schema (matching XenForo's xf_attachment_data)
// ============================================================================

/// Simulated xf_attachment_data row.
#[derive(Debug, Clone)]
pub struct AttachmentData {
    pub data_id: u64,
    pub user_id: u64,
    pub upload_date: u64,
    pub filename: String,
    pub file_size: u64,
    pub file_hash: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumbnail_width: Option<u32>,
    pub thumbnail_height: Option<u32>,
}

/// Simulated xf_attachment row.
#[derive(Debug, Clone)]
pub struct Attachment {
    pub attachment_id: u64,
    pub data_id: u64,
    pub content_type: String,
    pub content_id: u64,
    pub attach_date: u64,
    pub view_count: u64,
}

/// Combined attachment info for rendering.
#[derive(Debug, Clone)]
pub struct AttachmentInfo {
    pub attachment: Attachment,
    pub data: AttachmentData,
}

impl AttachmentInfo {
    /// Returns true if this is a video attachment.
    pub fn is_video(&self) -> bool {
        let ext = self.data.filename.rsplit('.').next().unwrap_or("");
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "mp4" | "webm" | "mov" | "avi" | "mkv"
        )
    }

    /// Returns true if this is an audio attachment.
    pub fn is_audio(&self) -> bool {
        let ext = self.data.filename.rsplit('.').next().unwrap_or("");
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "mp3" | "wav" | "ogg" | "flac" | "m4a"
        )
    }

    /// Returns true if this is an image attachment.
    pub fn is_image(&self) -> bool {
        let ext = self.data.filename.rsplit('.').next().unwrap_or("");
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp"
        )
    }

    /// Returns true if this attachment has a thumbnail.
    pub fn has_thumbnail(&self) -> bool {
        self.data.thumbnail_width.is_some() && self.data.thumbnail_height.is_some()
    }

    /// Returns the direct URL to the attachment.
    pub fn direct_url(&self) -> String {
        format!(
            "/attachments/{}.{}",
            self.attachment.attachment_id,
            self.data.filename.rsplit('.').next().unwrap_or("bin")
        )
    }

    /// Returns the thumbnail URL.
    pub fn thumbnail_url(&self) -> String {
        format!("/attachments/{}-thumb.jpg", self.attachment.attachment_id)
    }

    /// Returns the canonical URL to view the attachment.
    pub fn view_url(&self) -> String {
        format!("/attachments/{}/", self.attachment.attachment_id)
    }
}

// ============================================================================
// Simulated Database
// ============================================================================

/// Simulated database with attachment data.
pub struct AttachmentDatabase {
    attachments: HashMap<u64, Attachment>,
    attachment_data: HashMap<u64, AttachmentData>,
}

impl AttachmentDatabase {
    /// Creates a new database with sample data from the XenForo schema.
    pub fn new() -> Self {
        let mut db = Self {
            attachments: HashMap::new(),
            attachment_data: HashMap::new(),
        };

        // Sample data from xf_attachment_data (matching the user's query results)
        let data_rows = vec![
            AttachmentData {
                data_id: 1,
                user_id: 126,
                upload_date: 1360004492,
                filename: "Erykah Badu Album Cover.jpg".into(),
                file_size: 123517,
                file_hash: "fffc082aeb783ab459d80f7ec2e6fb8f".into(),
                width: Some(600),
                height: Some(600),
                thumbnail_width: Some(250),
                thumbnail_height: Some(250),
            },
            AttachmentData {
                data_id: 2,
                user_id: 22,
                upload_date: 1360108502,
                filename: "DSCN0656.jpg".into(),
                file_size: 86429,
                file_hash: "96090a938e0d8c112f68ee8b4ba16b44".into(),
                width: Some(1280),
                height: Some(960),
                thumbnail_width: Some(250),
                thumbnail_height: Some(188),
            },
            AttachmentData {
                data_id: 3,
                user_id: 22,
                upload_date: 1360108961,
                filename: "DSCN0657.jpg".into(),
                file_size: 112486,
                file_hash: "424b5b39b3d0ba1b6d97155bb0c88b46".into(),
                width: Some(1280),
                height: Some(1245),
                thumbnail_width: Some(250),
                thumbnail_height: Some(244),
            },
            AttachmentData {
                data_id: 4,
                user_id: 33,
                upload_date: 1360124325,
                filename: "hulkacwc.JPG".into(),
                file_size: 36847,
                file_hash: "ae258d4fbce2fb584b7a1bd624e5da9f".into(),
                width: Some(438),
                height: Some(575),
                thumbnail_width: Some(191),
                thumbnail_height: Some(250),
            },
            AttachmentData {
                data_id: 5,
                user_id: 33,
                upload_date: 1360124793,
                filename: "SchuComic9Page19-C.jpg".into(),
                file_size: 152400,
                file_hash: "d428a0cd2b876345152111ba2946a182".into(),
                width: Some(784),
                height: Some(1000),
                thumbnail_width: Some(196),
                thumbnail_height: Some(250),
            },
            // Add a video attachment
            AttachmentData {
                data_id: 100,
                user_id: 1,
                upload_date: 1700000000,
                filename: "funny_cat.mp4".into(),
                file_size: 5_000_000,
                file_hash: "abc123".into(),
                width: Some(1920),
                height: Some(1080),
                thumbnail_width: Some(320),
                thumbnail_height: Some(180),
            },
            // Add an audio attachment
            AttachmentData {
                data_id: 101,
                user_id: 1,
                upload_date: 1700000001,
                filename: "podcast_episode.mp3".into(),
                file_size: 10_000_000,
                file_hash: "def456".into(),
                width: None,
                height: None,
                thumbnail_width: None,
                thumbnail_height: None,
            },
            // Add an image without thumbnail
            AttachmentData {
                data_id: 102,
                user_id: 1,
                upload_date: 1700000002,
                filename: "document.pdf".into(),
                file_size: 500_000,
                file_hash: "ghi789".into(),
                width: None,
                height: None,
                thumbnail_width: None,
                thumbnail_height: None,
            },
        ];

        // Sample data from xf_attachment
        let attachment_rows = vec![
            Attachment {
                attachment_id: 1,
                data_id: 1,
                content_type: "post".into(),
                content_id: 2267,
                attach_date: 1360004492,
                view_count: 829,
            },
            Attachment {
                attachment_id: 2,
                data_id: 2,
                content_type: "post".into(),
                content_id: 11081,
                attach_date: 1360108502,
                view_count: 793,
            },
            Attachment {
                attachment_id: 3,
                data_id: 3,
                content_type: "post".into(),
                content_id: 11081,
                attach_date: 1360108961,
                view_count: 872,
            },
            Attachment {
                attachment_id: 4,
                data_id: 4,
                content_type: "post".into(),
                content_id: 945,
                attach_date: 1360124325,
                view_count: 2139,
            },
            Attachment {
                attachment_id: 5,
                data_id: 5,
                content_type: "post".into(),
                content_id: 1576,
                attach_date: 1360124793,
                view_count: 4614,
            },
            Attachment {
                attachment_id: 100,
                data_id: 100,
                content_type: "post".into(),
                content_id: 99999,
                attach_date: 1700000000,
                view_count: 100,
            },
            Attachment {
                attachment_id: 101,
                data_id: 101,
                content_type: "post".into(),
                content_id: 99999,
                attach_date: 1700000001,
                view_count: 50,
            },
            Attachment {
                attachment_id: 102,
                data_id: 102,
                content_type: "post".into(),
                content_id: 99999,
                attach_date: 1700000002,
                view_count: 10,
            },
        ];

        for data in data_rows {
            db.attachment_data.insert(data.data_id, data);
        }

        for attachment in attachment_rows {
            db.attachments.insert(attachment.attachment_id, attachment);
        }

        db
    }

    /// Looks up an attachment by ID.
    pub fn get(&self, attachment_id: u64) -> Option<AttachmentInfo> {
        let attachment = self.attachments.get(&attachment_id)?;
        let data = self.attachment_data.get(&attachment.data_id)?;
        Some(AttachmentInfo {
            attachment: attachment.clone(),
            data: data.clone(),
        })
    }

    /// Batch lookup of multiple attachment IDs.
    pub fn get_many(&self, ids: &[u64]) -> HashMap<u64, AttachmentInfo> {
        ids.iter()
            .filter_map(|&id| self.get(id).map(|info| (id, info)))
            .collect()
    }
}

// ============================================================================
// Attachment Display Options (XenForo-compatible)
// ============================================================================

/// Display options parsed from the attach tag.
#[derive(Debug, Clone, Default)]
pub struct AttachDisplayOptions {
    /// Display as full size image (vs thumbnail).
    pub full: bool,
    /// Explicit width (e.g., "100px", "50%").
    pub width: Option<String>,
    /// Explicit height (e.g., "200px", "auto").
    pub height: Option<String>,
    /// Alt text for the image.
    pub alt: Option<String>,
    /// Alignment (left, right, center).
    pub align: Option<String>,
}

impl AttachDisplayOptions {
    /// Parses display options from the tag's option.
    ///
    /// Supports XenForo formats:
    /// - `[attach=full]` - scalar "full"
    /// - `[attach type=full]` - map with type key
    /// - `[attach width=100px height=200px]` - explicit dimensions
    /// - `[attach=full width=100px]` - scalar + map combined (treated as map)
    pub fn from_tag(tag: &TagNode) -> Self {
        let mut opts = Self::default();

        match &tag.option {
            bbcode::TagOption::None => {}
            bbcode::TagOption::Scalar(s) => {
                // [attach=full] or [attach=thumb]
                if s.eq_ignore_ascii_case("full") {
                    opts.full = true;
                }
            }
            bbcode::TagOption::Map(map) => {
                // [attach type=full width=100px height=200px alt="description"]
                for (key, value) in map {
                    let key_lower = key.to_ascii_lowercase();
                    match key_lower.as_str() {
                        "type" => {
                            if value.eq_ignore_ascii_case("full") {
                                opts.full = true;
                            }
                        }
                        "width" => {
                            if Self::is_valid_dimension(value) {
                                opts.width = Some(value.to_string());
                            }
                        }
                        "height" => {
                            if Self::is_valid_dimension(value) {
                                opts.height = Some(value.to_string());
                            }
                        }
                        "alt" | "title" => {
                            opts.alt = Some(value.to_string());
                        }
                        "align" => {
                            if matches!(
                                value.to_ascii_lowercase().as_str(),
                                "left" | "right" | "center"
                            ) {
                                opts.align = Some(value.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        opts
    }

    /// Validates a dimension value (e.g., "100px", "50%", "auto").
    fn is_valid_dimension(s: &str) -> bool {
        if s == "auto" {
            return true;
        }
        // Must be digits followed by optional unit
        let s = s.trim();
        if s.is_empty() {
            return false;
        }

        // Check for percentage
        if s.ends_with('%') {
            return s[..s.len() - 1]
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.');
        }

        // Check for pixels
        let s = s.strip_suffix("px").unwrap_or(s);
        s.chars().all(|c| c.is_ascii_digit() || c == '.')
    }

    /// Returns the CSS style string for dimensions.
    pub fn style_string(&self) -> String {
        let mut styles = Vec::new();

        if let Some(ref w) = self.width {
            styles.push(format!("width: {}", w));
        }
        if let Some(ref h) = self.height {
            styles.push(format!("height: {}", h));
        }

        styles.join("; ")
    }

    /// Returns the CSS class for alignment.
    pub fn align_class(&self) -> &str {
        match self.align.as_deref() {
            Some("left") => "bbImage--left",
            Some("right") => "bbImage--right",
            Some("center") => "bbImage--center",
            _ => "",
        }
    }
}

// ============================================================================
// Attachment Handler
// ============================================================================

/// Custom handler for the [attach] BBCode tag.
///
/// This handler:
/// 1. Collects all attachment IDs during the collection phase
/// 2. Batch-fetches attachment data from the "database"
/// 3. Renders attachments with proper HTML based on type
pub struct AttachHandler {
    /// Database reference for lookups.
    db: Arc<AttachmentDatabase>,
    /// Collected attachment IDs (for batch fetching).
    collected_ids: Mutex<Vec<u64>>,
    /// Fetched attachment data (populated after prepare()).
    fetched: Mutex<HashMap<u64, AttachmentInfo>>,
    /// Whether user can view attachments.
    can_view: bool,
}

impl AttachHandler {
    /// Creates a new attachment handler with the given database.
    pub fn new(db: Arc<AttachmentDatabase>) -> Self {
        Self {
            db,
            collected_ids: Mutex::new(Vec::new()),
            fetched: Mutex::new(HashMap::new()),
            can_view: true,
        }
    }

    /// Creates a handler where user cannot view attachments.
    pub fn new_restricted(db: Arc<AttachmentDatabase>) -> Self {
        Self {
            db,
            collected_ids: Mutex::new(Vec::new()),
            fetched: Mutex::new(HashMap::new()),
            can_view: false,
        }
    }

    /// Parses the attachment ID from the tag content.
    fn parse_id(tag: &TagNode) -> Option<u64> {
        let content = tag.inner_text();
        content.trim().parse().ok()
    }

    /// Renders an image attachment.
    fn render_image(
        &self,
        info: &AttachmentInfo,
        opts: &AttachDisplayOptions,
        ctx: &RenderContext,
        output: &mut String,
    ) {
        let (src, intrinsic_width, intrinsic_height) = if opts.full {
            (info.direct_url(), info.data.width, info.data.height)
        } else if info.has_thumbnail() {
            (
                info.thumbnail_url(),
                info.data.thumbnail_width,
                info.data.thumbnail_height,
            )
        } else {
            (info.direct_url(), info.data.width, info.data.height)
        };

        let alt = opts.alt.as_deref().unwrap_or(&info.data.filename);
        let align_class = opts.align_class();
        let style = opts.style_string();

        // Build the img tag
        write!(output, "<img class=\"{}-attach", ctx.class_prefix).unwrap();
        if !align_class.is_empty() {
            write!(output, " {}", align_class).unwrap();
        }
        output.push('"');

        output.push_str(" src=\"");
        output.push_str(&escape_html(&src));
        output.push('"');

        output.push_str(" alt=\"");
        output.push_str(&escape_html(alt));
        output.push('"');

        // Add intrinsic dimensions if available and no explicit dimensions
        if opts.width.is_none() && opts.height.is_none() {
            if let (Some(w), Some(h)) = (intrinsic_width, intrinsic_height) {
                write!(output, " width=\"{}\" height=\"{}\"", w, h).unwrap();
            }
        }

        if !style.is_empty() {
            write!(output, " style=\"{}\"", escape_html(&style)).unwrap();
        }

        output.push_str(" />");

        // Wrap in link to full image if showing thumbnail
        if !opts.full && info.has_thumbnail() {
            let full_url = info.direct_url();
            let _wrapped = format!(
                "<a href=\"{}\" class=\"{}-attach-link\">{}</a>",
                escape_html(&full_url),
                ctx.class_prefix,
                output.split_at(output.rfind("<img").unwrap_or(0)).1
            );
            // This is a simplification - in practice you'd build this differently
        }
    }

    /// Renders a video attachment.
    fn render_video(
        &self,
        info: &AttachmentInfo,
        opts: &AttachDisplayOptions,
        ctx: &RenderContext,
        output: &mut String,
    ) {
        let src = info.direct_url();
        let style = opts.style_string();

        write!(
            output,
            "<video class=\"{}-attach-video\" controls",
            ctx.class_prefix
        )
        .unwrap();

        if !style.is_empty() {
            write!(output, " style=\"{}\"", escape_html(&style)).unwrap();
        }

        output.push_str("><source src=\"");
        output.push_str(&escape_html(&src));
        output.push_str("\" />Your browser does not support video.</video>");
    }

    /// Renders an audio attachment.
    fn render_audio(
        &self,
        info: &AttachmentInfo,
        _opts: &AttachDisplayOptions,
        ctx: &RenderContext,
        output: &mut String,
    ) {
        let src = info.direct_url();

        write!(
            output,
            "<audio class=\"{}-attach-audio\" controls><source src=\"{}\" />Your browser does not support audio.</audio>",
            ctx.class_prefix,
            escape_html(&src)
        )
        .unwrap();
    }

    /// Renders a generic file attachment (download link).
    fn render_file(&self, info: &AttachmentInfo, ctx: &RenderContext, output: &mut String) {
        let url = info.view_url();
        let filename = &info.data.filename;

        write!(
            output,
            "<a href=\"{}\" class=\"{}-attach-file\">📎 {}</a>",
            escape_html(&url),
            ctx.class_prefix,
            escape_html(filename)
        )
        .unwrap();
    }

    /// Renders a missing attachment placeholder.
    fn render_missing(&self, id: u64, ctx: &RenderContext, output: &mut String) {
        write!(
            output,
            "<a href=\"/attachments/{}\" class=\"{}-attach-missing\">View attachment {}</a>",
            id, ctx.class_prefix, id
        )
        .unwrap();
    }
}

impl CustomTagHandler for AttachHandler {
    fn tag_name(&self) -> &str {
        "attach"
    }

    fn collect(&self, tag: &TagNode) {
        if let Some(id) = Self::parse_id(tag) {
            let mut ids = self.collected_ids.lock().unwrap();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }

    fn prepare(&self) {
        let ids = self.collected_ids.lock().unwrap();
        if ids.is_empty() {
            return;
        }

        // Batch fetch from "database"
        let fetched_data = self.db.get_many(&ids);

        let mut fetched = self.fetched.lock().unwrap();
        *fetched = fetched_data;
    }

    fn render(&self, tag: &TagNode, ctx: &RenderContext, output: &mut String) -> bool {
        let id = match Self::parse_id(tag) {
            Some(id) => id,
            None => {
                // Invalid ID - render nothing
                return true;
            }
        };

        let opts = AttachDisplayOptions::from_tag(tag);
        let fetched = self.fetched.lock().unwrap();

        match fetched.get(&id) {
            Some(info) => {
                if !self.can_view {
                    // User can't view - show placeholder
                    self.render_missing(id, ctx, output);
                } else if info.is_video() {
                    self.render_video(info, &opts, ctx, output);
                } else if info.is_audio() {
                    self.render_audio(info, &opts, ctx, output);
                } else if info.is_image() {
                    self.render_image(info, &opts, ctx, output);
                } else {
                    self.render_file(info, ctx, output);
                }
            }
            None => {
                self.render_missing(id, ctx, output);
            }
        }

        true
    }
}

// ============================================================================
// Tag Definition
// ============================================================================

/// Creates the custom tag definition for [attach].
/// This defines the parsing rules for the tag - it must be registered with the parser.
fn attach_tag_def() -> CustomTagDef {
    CustomTagDef {
        name: "attach".into(),
        aliases: vec!["attachment".into()],
        tag_type: TagType::Inline,
        option_allowed: true,
        has_content: true,
        trim_content: true,
        stop_auto_link: true,
        stop_smilies: true,
        ..Default::default()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Arc<AttachmentDatabase>, Parser, Renderer) {
        let db = Arc::new(AttachmentDatabase::new());

        // Create parser and register the attach tag
        let mut parser = Parser::new();
        parser.register_custom_tag(attach_tag_def());

        // Create renderer and register the handler
        let mut renderer = Renderer::new();
        renderer.register_handler(Arc::new(AttachHandler::new(Arc::clone(&db))));

        (db, parser, renderer)
    }

    fn render_with_collect(parser: &Parser, renderer: &Renderer, input: &str) -> String {
        let doc = parser.parse(input);
        renderer.collect_from_document(&doc);
        renderer.render(&doc)
    }

    #[test]
    fn debug_option_parsing() {
        let (_, parser, _) = setup();

        let test_cases = [
            "[attach]1[/attach]",
            "[attach=full]1[/attach]",
            "[attach width=100px]1[/attach]",
            "[attach width=100px height=100px]1[/attach]",
            "[attach type=full width=300px]2[/attach]",
        ];

        for input in test_cases {
            let doc = parser.parse(input);
            println!("\nInput: {}", input);
            for node in &doc.nodes {
                if let Some(tag) = node.as_tag() {
                    println!("  Tag: {:?}, Option: {:?}", tag.raw_name, tag.option);
                } else if let Some(text) = node.as_text() {
                    println!("  Text: {:?}", text);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Basic functionality tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_url_onclick_still_blocked() {
        // This verifies that URL tags with malicious onclick are still blocked
        let parser = Parser::new();
        let doc = parser.parse("[url=http://google.com onclick=alert(1)]Click[/url]");

        let renderer = Renderer::new();
        let html = renderer.render(&doc);

        // The onclick should either:
        // 1. Be in raw BBCode text (safe - not interpreted as HTML)
        // 2. Not appear at all in HTML attribute context
        let is_raw_bbcode = html.contains("[url=");
        let has_onclick_in_anchor = html.contains("<a ") && html.contains(" onclick=");

        assert!(
            is_raw_bbcode || !has_onclick_in_anchor,
            "onclick should not appear in anchor tag: {}",
            html
        );
    }

    #[test]
    fn test_basic_attach_thumbnail() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach]1[/attach]");

        assert!(html.contains("bbcode-attach"));
        assert!(html.contains("src=\"/attachments/1-thumb.jpg\""));
        assert!(html.contains("Erykah Badu Album Cover.jpg"));
    }

    #[test]
    fn test_attach_full() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach=full]1[/attach]");

        assert!(html.contains("src=\"/attachments/1.jpg\""));
        assert!(!html.contains("thumb"));
    }

    #[test]
    fn test_attach_type_full() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach type=full]1[/attach]");

        assert!(html.contains("src=\"/attachments/1.jpg\""));
    }

    #[test]
    fn test_attach_with_dimensions() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(
            &parser,
            &renderer,
            "[attach width=100px height=200px]1[/attach]",
        );

        assert!(html.contains("width: 100px"));
        assert!(html.contains("height: 200px"));
    }

    #[test]
    fn test_attach_with_percentage() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach width=50%]1[/attach]");

        assert!(html.contains("width: 50%"));
    }

    #[test]
    fn test_attach_with_alt() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(
            &parser,
            &renderer,
            "[attach alt=\"Custom alt text\"]1[/attach]",
        );

        assert!(html.contains("alt=\"Custom alt text\""));
    }

    #[test]
    fn test_attach_with_alignment() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach align=left]1[/attach]");

        assert!(html.contains("bbImage--left"));
    }

    // -------------------------------------------------------------------------
    // Missing/invalid attachment tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_attach_missing() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach]99999[/attach]");

        assert!(html.contains("bbcode-attach-missing"));
        assert!(html.contains("View attachment 99999"));
    }

    #[test]
    fn test_attach_invalid_id() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach]not-a-number[/attach]");

        // Invalid ID produces no output
        assert!(html.is_empty() || !html.contains("attach"));
    }

    #[test]
    fn test_attach_empty() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach][/attach]");

        assert!(html.is_empty() || !html.contains("attach"));
    }

    // -------------------------------------------------------------------------
    // Media type tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_attach_video() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach]100[/attach]");

        assert!(html.contains("<video"));
        assert!(html.contains("controls"));
        assert!(html.contains("/attachments/100.mp4"));
    }

    #[test]
    fn test_attach_audio() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach]101[/attach]");

        assert!(html.contains("<audio"));
        assert!(html.contains("controls"));
        assert!(html.contains("/attachments/101.mp3"));
    }

    #[test]
    fn test_attach_generic_file() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach]102[/attach]");

        assert!(html.contains("bbcode-attach-file"));
        assert!(html.contains("document.pdf"));
        assert!(html.contains("📎"));
    }

    // -------------------------------------------------------------------------
    // Batch fetching tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_multiple_attachments_batched() {
        let (_, parser, renderer) = setup();
        let input = "[attach]1[/attach] [attach]2[/attach] [attach]3[/attach]";
        let html = render_with_collect(&parser, &renderer, input);

        // All three attachments should be rendered
        assert!(html.contains("/attachments/1"));
        assert!(html.contains("/attachments/2"));
        assert!(html.contains("/attachments/3"));
    }

    #[test]
    fn test_duplicate_attachments() {
        let (_, parser, renderer) = setup();
        let input = "[attach]1[/attach] [attach]1[/attach]";
        let html = render_with_collect(&parser, &renderer, input);

        // Should have two images
        assert_eq!(html.matches("bbcode-attach").count(), 2);
    }

    // -------------------------------------------------------------------------
    // Permission tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_attach_no_permission() {
        let db = Arc::new(AttachmentDatabase::new());
        let mut parser = Parser::new();
        parser.register_custom_tag(attach_tag_def());
        let mut renderer = Renderer::new();
        renderer.register_handler(Arc::new(AttachHandler::new_restricted(Arc::clone(&db))));

        let doc = parser.parse("[attach]1[/attach]");
        renderer.collect_from_document(&doc);
        let html = renderer.render(&doc);

        assert!(html.contains("bbcode-attach-missing"));
        assert!(html.contains("View attachment 1"));
    }

    // -------------------------------------------------------------------------
    // Case insensitivity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_attach_case_insensitive() {
        let (_, parser, renderer) = setup();

        let html1 = render_with_collect(&parser, &renderer, "[ATTACH]1[/ATTACH]");
        let html2 = render_with_collect(&parser, &renderer, "[Attach]1[/Attach]");
        let html3 = render_with_collect(&parser, &renderer, "[attach]1[/attach]");

        assert!(html1.contains("bbcode-attach"));
        assert!(html2.contains("bbcode-attach"));
        assert!(html3.contains("bbcode-attach"));
    }

    #[test]
    fn test_attach_full_case_insensitive() {
        let (_, parser, renderer) = setup();

        let html1 = render_with_collect(&parser, &renderer, "[attach=FULL]1[/attach]");
        let html2 = render_with_collect(&parser, &renderer, "[attach=Full]1[/attach]");

        assert!(html1.contains("src=\"/attachments/1.jpg\""));
        assert!(html2.contains("src=\"/attachments/1.jpg\""));
    }

    // -------------------------------------------------------------------------
    // Integration tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_attach_in_quote() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(
            &parser,
            &renderer,
            "[quote]Check this out: [attach]1[/attach][/quote]",
        );

        assert!(html.contains("<blockquote"));
        assert!(html.contains("bbcode-attach"));
    }

    #[test]
    fn test_attach_with_text() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "Before [attach]1[/attach] After");

        assert!(html.starts_with("Before "));
        assert!(html.contains("bbcode-attach"));
        assert!(html.ends_with(" After"));
    }

    // -------------------------------------------------------------------------
    // Intrinsic dimension tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_attach_intrinsic_dimensions() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach=full]1[/attach]");

        // Full size should use original dimensions
        assert!(html.contains("width=\"600\""));
        assert!(html.contains("height=\"600\""));
    }

    #[test]
    fn test_attach_thumbnail_dimensions() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(&parser, &renderer, "[attach]2[/attach]");

        // Thumbnail should use thumbnail dimensions
        assert!(html.contains("width=\"250\""));
        assert!(html.contains("height=\"188\""));
    }

    #[test]
    fn test_explicit_dimensions_override_intrinsic() {
        let (_, parser, renderer) = setup();
        let html = render_with_collect(
            &parser,
            &renderer,
            "[attach width=100px height=100px]1[/attach]",
        );

        // Explicit dimensions should be used, not intrinsic
        assert!(html.contains("width: 100px"));
        assert!(html.contains("height: 100px"));
        assert!(!html.contains("width=\"600\""));
    }
}

// ============================================================================
// Main example
// ============================================================================

fn main() {
    println!("BBCode Attachment Handler Example\n");
    println!("==================================\n");

    // Create the database
    let db = Arc::new(AttachmentDatabase::new());

    // Create parser and renderer
    let mut parser = Parser::new();

    // Register the custom [attach] tag with the parser
    // This is required so the parser recognizes [attach] as a valid tag
    parser.register_custom_tag(attach_tag_def());

    let mut renderer = Renderer::with_config(RenderConfig {
        class_prefix: Cow::Borrowed("xf"),
        ..Default::default()
    });

    // Register the custom attach handler
    renderer.register_handler(Arc::new(AttachHandler::new(Arc::clone(&db))));

    // Example inputs
    let examples = vec![
        ("[attach]1[/attach]", "Basic thumbnail"),
        ("[attach=full]1[/attach]", "Full size image"),
        (
            "[attach type=full width=300px]2[/attach]",
            "Full with explicit width",
        ),
        (
            "[attach width=50% align=left]3[/attach]",
            "Percentage width, left aligned",
        ),
        ("[attach]100[/attach]", "Video attachment"),
        ("[attach]101[/attach]", "Audio attachment"),
        ("[attach]102[/attach]", "Generic file attachment"),
        ("[attach]99999[/attach]", "Missing attachment"),
        (
            "[quote=\"User\"]Check this image: [attach]4[/attach][/quote]",
            "Attachment in quote",
        ),
    ];

    for (input, description) in examples {
        println!("{}:", description);
        println!("  Input:  {}", input);

        let doc = parser.parse(input);
        renderer.collect_from_document(&doc);
        let html = renderer.render(&doc);

        println!("  Output: {}", html);
        println!();
    }

    println!("==================================");
    println!("Example complete!");
}
