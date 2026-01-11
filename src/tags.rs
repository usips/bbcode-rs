//! BBCode tag definitions and registry.
//!
//! This module defines all supported BBCode tags, their properties,
//! and how they should be parsed and rendered.
//!
//! ## Custom Tags
//!
//! To add custom tags to your application, use [`CustomTagDef`] which supports
//! runtime-defined tags with owned strings:
//!
//! ```rust
//! use bbcode::{TagRegistry, CustomTagDef, TagType};
//!
//! let mut registry = TagRegistry::new();
//! registry.register_custom(CustomTagDef {
//!     name: "attach".into(),
//!     aliases: vec!["attachment".into()],
//!     tag_type: TagType::Inline,
//!     ..Default::default()
//! });
//! ```

use crate::ast::TagType;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

/// Definition of a BBCode tag (static, compile-time).
///
/// This is used for the built-in tags. For custom tags defined at runtime,
/// use [`CustomTagDef`] instead.
#[derive(Debug, Clone)]
pub struct TagDef {
    /// The canonical tag name (lowercase).
    pub name: &'static str,

    /// Alternative names for this tag (aliases).
    pub aliases: &'static [&'static str],

    /// The type of tag (inline, block, verbatim, etc.).
    pub tag_type: TagType,

    /// Whether this tag requires an option/argument.
    pub option_required: bool,

    /// Whether this tag allows an option/argument.
    pub option_allowed: bool,

    /// Whether this tag can have children.
    pub has_content: bool,

    /// Tags that this tag cannot be nested inside.
    pub forbidden_ancestors: &'static [&'static str],

    /// Tags that must be direct parents of this tag.
    pub required_parents: &'static [&'static str],

    /// The HTML tag to render as (if simple).
    pub html_tag: Option<&'static str>,

    /// Whether to stop smilie/emoji conversion inside this tag.
    pub stop_smilies: bool,

    /// Whether to stop auto-linking URLs inside this tag.
    pub stop_auto_link: bool,

    /// Whether newlines should be converted to <br>.
    pub convert_newlines: bool,

    /// Whether content should be trimmed.
    pub trim_content: bool,
}

impl Default for TagDef {
    fn default() -> Self {
        Self {
            name: "",
            aliases: &[],
            tag_type: TagType::Inline,
            option_required: false,
            option_allowed: true,
            has_content: true,
            forbidden_ancestors: &[],
            required_parents: &[],
            html_tag: None,
            stop_smilies: false,
            stop_auto_link: false,
            convert_newlines: true,
            trim_content: false,
        }
    }
}

impl TagDef {
    /// Returns true if this tag is a verbatim tag (content not parsed).
    #[inline]
    pub fn is_verbatim(&self) -> bool {
        self.tag_type == TagType::Verbatim
    }

    /// Returns true if this tag is self-closing.
    #[inline]
    pub fn is_self_closing(&self) -> bool {
        self.tag_type == TagType::SelfClosing
    }

    /// Returns true if this tag is a block element.
    #[inline]
    pub fn is_block(&self) -> bool {
        self.tag_type == TagType::Block
    }

    /// Returns true if this tag is an inline element.
    #[inline]
    pub fn is_inline(&self) -> bool {
        self.tag_type == TagType::Inline
    }
}

/// Definition of a custom BBCode tag (runtime, owned strings).
///
/// Use this to define tags at runtime in your application without
/// modifying the bbcode library.
///
/// # Example
///
/// ```rust
/// use bbcode::{CustomTagDef, TagType};
///
/// let attach_tag = CustomTagDef {
///     name: "attach".into(),
///     aliases: vec!["attachment".into()],
///     tag_type: TagType::Inline,
///     option_allowed: true,
///     has_content: true,
///     trim_content: true,
///     stop_auto_link: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CustomTagDef {
    /// The canonical tag name (lowercase).
    pub name: Cow<'static, str>,

    /// Alternative names for this tag (aliases).
    pub aliases: Vec<Cow<'static, str>>,

    /// The type of tag (inline, block, verbatim, etc.).
    pub tag_type: TagType,

    /// Whether this tag requires an option/argument.
    pub option_required: bool,

    /// Whether this tag allows an option/argument.
    pub option_allowed: bool,

    /// Whether this tag can have children.
    pub has_content: bool,

    /// Tags that this tag cannot be nested inside.
    pub forbidden_ancestors: Vec<Cow<'static, str>>,

    /// Tags that must be direct parents of this tag.
    pub required_parents: Vec<Cow<'static, str>>,

    /// The HTML tag to render as (if simple).
    pub html_tag: Option<Cow<'static, str>>,

    /// Whether to stop smilie/emoji conversion inside this tag.
    pub stop_smilies: bool,

    /// Whether to stop auto-linking URLs inside this tag.
    pub stop_auto_link: bool,

    /// Whether newlines should be converted to <br>.
    pub convert_newlines: bool,

    /// Whether content should be trimmed.
    pub trim_content: bool,
}

impl Default for CustomTagDef {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(""),
            aliases: Vec::new(),
            tag_type: TagType::Inline,
            option_required: false,
            option_allowed: true,
            has_content: true,
            forbidden_ancestors: Vec::new(),
            required_parents: Vec::new(),
            html_tag: None,
            stop_smilies: false,
            stop_auto_link: false,
            convert_newlines: true,
            trim_content: false,
        }
    }
}

impl CustomTagDef {
    /// Creates a new custom tag with the given name.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Returns true if this tag is a verbatim tag (content not parsed).
    #[inline]
    pub fn is_verbatim(&self) -> bool {
        self.tag_type == TagType::Verbatim
    }

    /// Returns true if this tag is self-closing.
    #[inline]
    pub fn is_self_closing(&self) -> bool {
        self.tag_type == TagType::SelfClosing
    }

    /// Returns true if this tag is a block element.
    #[inline]
    pub fn is_block(&self) -> bool {
        self.tag_type == TagType::Block
    }

    /// Returns true if this tag is an inline element.
    #[inline]
    pub fn is_inline(&self) -> bool {
        self.tag_type == TagType::Inline
    }
}

/// A resolved tag definition that can be either static or custom.
#[derive(Debug, Clone)]
pub enum ResolvedTag {
    /// A built-in static tag definition.
    Static(&'static TagDef),
    /// A custom runtime tag definition.
    Custom(Arc<CustomTagDef>),
}

impl ResolvedTag {
    /// Returns the tag name.
    pub fn name(&self) -> &str {
        match self {
            ResolvedTag::Static(t) => t.name,
            ResolvedTag::Custom(t) => &t.name,
        }
    }

    /// Returns the tag type.
    pub fn tag_type(&self) -> TagType {
        match self {
            ResolvedTag::Static(t) => t.tag_type,
            ResolvedTag::Custom(t) => t.tag_type,
        }
    }

    /// Returns true if this tag is verbatim.
    pub fn is_verbatim(&self) -> bool {
        match self {
            ResolvedTag::Static(t) => t.is_verbatim(),
            ResolvedTag::Custom(t) => t.is_verbatim(),
        }
    }

    /// Returns true if this tag is self-closing.
    pub fn is_self_closing(&self) -> bool {
        match self {
            ResolvedTag::Static(t) => t.is_self_closing(),
            ResolvedTag::Custom(t) => t.is_self_closing(),
        }
    }

    /// Returns true if this tag is a block element.
    pub fn is_block(&self) -> bool {
        match self {
            ResolvedTag::Static(t) => t.is_block(),
            ResolvedTag::Custom(t) => t.is_block(),
        }
    }

    /// Returns true if content should be trimmed.
    pub fn trim_content(&self) -> bool {
        match self {
            ResolvedTag::Static(t) => t.trim_content,
            ResolvedTag::Custom(t) => t.trim_content,
        }
    }

    /// Returns true if this tag has content.
    pub fn has_content(&self) -> bool {
        match self {
            ResolvedTag::Static(t) => t.has_content,
            ResolvedTag::Custom(t) => t.has_content,
        }
    }

    /// Returns true if auto-linking should be stopped.
    pub fn stop_auto_link(&self) -> bool {
        match self {
            ResolvedTag::Static(t) => t.stop_auto_link,
            ResolvedTag::Custom(t) => t.stop_auto_link,
        }
    }

    /// Returns true if an option is required.
    pub fn option_required(&self) -> bool {
        match self {
            ResolvedTag::Static(t) => t.option_required,
            ResolvedTag::Custom(t) => t.option_required,
        }
    }

    /// Returns true if an option is allowed.
    pub fn option_allowed(&self) -> bool {
        match self {
            ResolvedTag::Static(t) => t.option_allowed,
            ResolvedTag::Custom(t) => t.option_allowed,
        }
    }

    /// Checks if the given ancestor name is forbidden.
    pub fn is_ancestor_forbidden(&self, ancestor: &str) -> bool {
        let ancestor_lower = ancestor.to_ascii_lowercase();
        match self {
            ResolvedTag::Static(t) => t
                .forbidden_ancestors
                .iter()
                .any(|a| a.eq_ignore_ascii_case(&ancestor_lower)),
            ResolvedTag::Custom(t) => t
                .forbidden_ancestors
                .iter()
                .any(|a| a.eq_ignore_ascii_case(&ancestor_lower)),
        }
    }

    /// Checks if a required parent is satisfied by the stack.
    pub fn has_required_parent(&self, stack: &[impl AsRef<str>]) -> bool {
        let required = match self {
            ResolvedTag::Static(t) => t.required_parents,
            ResolvedTag::Custom(t) => {
                return t.required_parents.is_empty()
                    || stack.iter().any(|s| {
                        t.required_parents
                            .iter()
                            .any(|r| r.eq_ignore_ascii_case(s.as_ref()))
                    })
            }
        };

        if required.is_empty() {
            return true;
        }

        stack
            .iter()
            .any(|s| required.iter().any(|r| r.eq_ignore_ascii_case(s.as_ref())))
    }
}

/// Registry of all supported BBCode tags.
pub struct TagRegistry {
    static_tags: HashMap<&'static str, &'static TagDef>,
    custom_tags: HashMap<String, Arc<CustomTagDef>>,
}

impl TagRegistry {
    /// Creates a new registry with all standard tags registered.
    pub fn new() -> Self {
        let mut registry = Self {
            static_tags: HashMap::new(),
            custom_tags: HashMap::new(),
        };

        // Register all standard tags
        for tag in STANDARD_TAGS.iter() {
            registry.register(tag);
        }

        registry
    }

    /// Creates an empty registry with no tags.
    pub fn empty() -> Self {
        Self {
            static_tags: HashMap::new(),
            custom_tags: HashMap::new(),
        }
    }

    /// Registers a static tag definition.
    pub fn register(&mut self, tag: &'static TagDef) {
        self.static_tags.insert(tag.name, tag);
        for alias in tag.aliases {
            self.static_tags.insert(alias, tag);
        }
    }

    /// Registers a custom tag definition.
    ///
    /// Custom tags take precedence over static tags with the same name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bbcode::{TagRegistry, CustomTagDef, TagType};
    ///
    /// let mut registry = TagRegistry::new();
    /// registry.register_custom(CustomTagDef {
    ///     name: "attach".into(),
    ///     aliases: vec!["attachment".into()],
    ///     tag_type: TagType::Inline,
    ///     option_allowed: true,
    ///     trim_content: true,
    ///     ..Default::default()
    /// });
    /// ```
    pub fn register_custom(&mut self, tag: CustomTagDef) {
        let tag = Arc::new(tag);
        let name = tag.name.to_ascii_lowercase();
        self.custom_tags.insert(name, Arc::clone(&tag));
        for alias in &tag.aliases {
            let alias_lower = alias.to_ascii_lowercase();
            self.custom_tags.insert(alias_lower, Arc::clone(&tag));
        }
    }

    /// Looks up a tag by name (case-insensitive).
    ///
    /// Custom tags take precedence over static tags.
    pub fn resolve(&self, name: &str) -> Option<ResolvedTag> {
        let lower = name.to_ascii_lowercase();

        // Check custom tags first
        if let Some(tag) = self.custom_tags.get(&lower) {
            return Some(ResolvedTag::Custom(Arc::clone(tag)));
        }

        // Fall back to static tags
        self.static_tags
            .get(lower.as_str())
            .map(|t| ResolvedTag::Static(t))
    }

    /// Looks up a static tag by name (case-insensitive).
    ///
    /// This only returns built-in static tags, not custom tags.
    pub fn get(&self, name: &str) -> Option<&'static TagDef> {
        let lower = name.to_ascii_lowercase();
        self.static_tags.get(lower.as_str()).copied()
    }

    /// Returns true if the tag is known (either static or custom).
    pub fn is_known(&self, name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        self.custom_tags.contains_key(&lower) || self.static_tags.contains_key(lower.as_str())
    }

    /// Returns an iterator over all registered static tags.
    pub fn iter(&self) -> impl Iterator<Item = &'static TagDef> + '_ {
        // Deduplicate by name
        let mut seen = std::collections::HashSet::new();
        self.static_tags
            .values()
            .filter(move |tag| seen.insert(tag.name))
            .copied()
    }

    /// Returns an iterator over all registered custom tags.
    pub fn iter_custom(&self) -> impl Iterator<Item = &Arc<CustomTagDef>> + '_ {
        let mut seen = std::collections::HashSet::new();
        self.custom_tags
            .values()
            .filter(move |tag| seen.insert(tag.name.as_ref()))
    }
}

impl Default for TagRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Standard Tag Definitions
// ============================================================================

/// Bold text: [b]...[/b]
pub static TAG_BOLD: TagDef = TagDef {
    name: "b",
    aliases: &["bold"],
    tag_type: TagType::Inline,
    html_tag: Some("strong"),
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Italic text: [i]...[/i]
pub static TAG_ITALIC: TagDef = TagDef {
    name: "i",
    aliases: &["italic"],
    tag_type: TagType::Inline,
    html_tag: Some("em"),
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Underline text: [u]...[/u]
pub static TAG_UNDERLINE: TagDef = TagDef {
    name: "u",
    aliases: &["underline"],
    tag_type: TagType::Inline,
    html_tag: Some("u"),
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Strikethrough text: [s]...[/s]
pub static TAG_STRIKETHROUGH: TagDef = TagDef {
    name: "s",
    aliases: &["strike", "strikethrough"],
    tag_type: TagType::Inline,
    html_tag: Some("s"),
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Color text: [color=red]...[/color]
pub static TAG_COLOR: TagDef = TagDef {
    name: "color",
    aliases: &["colour"],
    tag_type: TagType::Inline,
    html_tag: None, // Custom rendering
    option_required: true,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Font: [font=Arial]...[/font]
pub static TAG_FONT: TagDef = TagDef {
    name: "font",
    aliases: &[],
    tag_type: TagType::Inline,
    html_tag: None, // Custom rendering
    option_required: true,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Size: [size=150]...[/size] or [size=12px]...[/size]
pub static TAG_SIZE: TagDef = TagDef {
    name: "size",
    aliases: &[],
    tag_type: TagType::Inline,
    html_tag: None, // Custom rendering
    option_required: true,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// URL/Link: [url]...[/url] or [url=http://...]...[/url]
pub static TAG_URL: TagDef = TagDef {
    name: "url",
    aliases: &["link"],
    tag_type: TagType::Inline,
    html_tag: None, // Custom rendering
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &["url", "email"],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: true,
    convert_newlines: true,
    trim_content: false,
};

/// Email: [email]...[/email] or [email=addr]...[/email]
pub static TAG_EMAIL: TagDef = TagDef {
    name: "email",
    aliases: &["mail"],
    tag_type: TagType::Inline,
    html_tag: None, // Custom rendering
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &["url", "email"],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: true,
    convert_newlines: true,
    trim_content: false,
};

/// Image: [img]url[/img] or [img=widthxheight]url[/img]
pub static TAG_IMG: TagDef = TagDef {
    name: "img",
    aliases: &["image"],
    tag_type: TagType::Void,
    html_tag: None, // Custom rendering
    option_required: false,
    option_allowed: true,
    has_content: true, // URL is the content
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: true,
    stop_auto_link: true,
    convert_newlines: false,
    trim_content: true,
};

/// Quote: [quote]...[/quote] or [quote="author"]...[/quote]
pub static TAG_QUOTE: TagDef = TagDef {
    name: "quote",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: None, // Custom rendering with blockquote
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: true,
};

/// Code block: [code]...[/code] or [code=lang]...[/code]
pub static TAG_CODE: TagDef = TagDef {
    name: "code",
    aliases: &[],
    tag_type: TagType::Verbatim,
    html_tag: None, // Custom rendering with pre/code
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: true,
    stop_auto_link: true,
    convert_newlines: false,
    trim_content: false,
};

/// Inline code: [icode]...[/icode]
pub static TAG_ICODE: TagDef = TagDef {
    name: "icode",
    aliases: &["c", "inline"],
    tag_type: TagType::Verbatim,
    html_tag: Some("code"),
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: true,
    stop_auto_link: true,
    convert_newlines: false,
    trim_content: false,
};

/// PHP code: [php]...[/php]
pub static TAG_PHP: TagDef = TagDef {
    name: "php",
    aliases: &[],
    tag_type: TagType::Verbatim,
    html_tag: None,
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: true,
    stop_auto_link: true,
    convert_newlines: false,
    trim_content: false,
};

/// HTML code: [html]...[/html]
pub static TAG_HTML: TagDef = TagDef {
    name: "html",
    aliases: &[],
    tag_type: TagType::Verbatim,
    html_tag: None,
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: true,
    stop_auto_link: true,
    convert_newlines: false,
    trim_content: false,
};

/// Plain text (no parsing): [plain]...[/plain]
pub static TAG_PLAIN: TagDef = TagDef {
    name: "plain",
    aliases: &["noparse", "nobbc"],
    tag_type: TagType::Verbatim,
    html_tag: None, // Renders as-is
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: true,
    stop_auto_link: true,
    convert_newlines: true,
    trim_content: false,
};

/// List: [list]...[/list] or [list=1]...[/list]
pub static TAG_LIST: TagDef = TagDef {
    name: "list",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: None, // Can be ul or ol
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: false,
    trim_content: true,
};

/// List item: [*]
pub static TAG_LIST_ITEM: TagDef = TagDef {
    name: "*",
    aliases: &["li"],
    tag_type: TagType::SelfClosing,
    html_tag: Some("li"),
    option_required: false,
    option_allowed: false,
    has_content: true, // Content until next [*] or [/list]
    forbidden_ancestors: &[],
    required_parents: &["list"],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Horizontal rule: [hr]
pub static TAG_HR: TagDef = TagDef {
    name: "hr",
    aliases: &[],
    tag_type: TagType::SelfClosing,
    html_tag: Some("hr"),
    option_required: false,
    option_allowed: false,
    has_content: false,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Line break: [br]
pub static TAG_BR: TagDef = TagDef {
    name: "br",
    aliases: &[],
    tag_type: TagType::SelfClosing,
    html_tag: Some("br"),
    option_required: false,
    option_allowed: false,
    has_content: false,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Left align: [left]...[/left]
pub static TAG_LEFT: TagDef = TagDef {
    name: "left",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: None, // Custom rendering with div
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Center align: [center]...[/center]
pub static TAG_CENTER: TagDef = TagDef {
    name: "center",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: None, // Custom rendering with div
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Right align: [right]...[/right]
pub static TAG_RIGHT: TagDef = TagDef {
    name: "right",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: None, // Custom rendering with div
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Justify align: [justify]...[/justify]
pub static TAG_JUSTIFY: TagDef = TagDef {
    name: "justify",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: None, // Custom rendering with div
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Indent: [indent]...[/indent] or [indent=2]...[/indent]
pub static TAG_INDENT: TagDef = TagDef {
    name: "indent",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: None, // Custom rendering with margin
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Heading: [heading=1]...[/heading]
pub static TAG_HEADING: TagDef = TagDef {
    name: "heading",
    aliases: &["h"],
    tag_type: TagType::Block,
    html_tag: None, // h1, h2, h3, etc.
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: false,
    trim_content: true,
};

/// Spoiler (block): [spoiler]...[/spoiler] or [spoiler=title]...[/spoiler]
pub static TAG_SPOILER: TagDef = TagDef {
    name: "spoiler",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: None, // Custom rendering with details/summary
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Inline spoiler: [ispoiler]...[/ispoiler]
pub static TAG_ISPOILER: TagDef = TagDef {
    name: "ispoiler",
    aliases: &[],
    tag_type: TagType::Inline,
    html_tag: None, // Custom rendering
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// User mention: [user=123]username[/user]
pub static TAG_USER: TagDef = TagDef {
    name: "user",
    aliases: &["member"],
    tag_type: TagType::Inline,
    html_tag: None, // Custom rendering with link
    option_required: true,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: true,
    stop_auto_link: true,
    convert_newlines: false,
    trim_content: false,
};

/// Subscript: [sub]...[/sub]
pub static TAG_SUB: TagDef = TagDef {
    name: "sub",
    aliases: &[],
    tag_type: TagType::Inline,
    html_tag: Some("sub"),
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Superscript: [sup]...[/sup]
pub static TAG_SUP: TagDef = TagDef {
    name: "sup",
    aliases: &[],
    tag_type: TagType::Inline,
    html_tag: Some("sup"),
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

// ============================================================================
// Table Tags
// ============================================================================

/// Table: [table]...[/table]
pub static TAG_TABLE: TagDef = TagDef {
    name: "table",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: Some("table"),
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &[],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: false,
    trim_content: true,
};

/// Table row: [tr]...[/tr]
pub static TAG_TR: TagDef = TagDef {
    name: "tr",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: Some("tr"),
    option_required: false,
    option_allowed: false,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &["table"],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: false,
    trim_content: true,
};

/// Table header cell: [th]...[/th]
pub static TAG_TH: TagDef = TagDef {
    name: "th",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: Some("th"),
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &["tr"],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

/// Table data cell: [td]...[/td]
pub static TAG_TD: TagDef = TagDef {
    name: "td",
    aliases: &[],
    tag_type: TagType::Block,
    html_tag: Some("td"),
    option_required: false,
    option_allowed: true,
    has_content: true,
    forbidden_ancestors: &[],
    required_parents: &["tr"],
    stop_smilies: false,
    stop_auto_link: false,
    convert_newlines: true,
    trim_content: false,
};

// ============================================================================
// Collection of all standard tags
// ============================================================================

/// All standard BBCode tags.
///
/// To add custom tags (like `[attach]` or `[media]`), use
/// [`TagRegistry::register_custom`] with a [`CustomTagDef`].
pub static STANDARD_TAGS: &[&TagDef] = &[
    // Basic formatting
    &TAG_BOLD,
    &TAG_ITALIC,
    &TAG_UNDERLINE,
    &TAG_STRIKETHROUGH,
    &TAG_COLOR,
    &TAG_FONT,
    &TAG_SIZE,
    &TAG_SUB,
    &TAG_SUP,
    // Links and images
    &TAG_URL,
    &TAG_EMAIL,
    &TAG_IMG,
    // Block elements
    &TAG_QUOTE,
    &TAG_CODE,
    &TAG_ICODE,
    &TAG_PHP,
    &TAG_HTML,
    &TAG_PLAIN,
    &TAG_LIST,
    &TAG_LIST_ITEM,
    // Alignment
    &TAG_LEFT,
    &TAG_CENTER,
    &TAG_RIGHT,
    &TAG_JUSTIFY,
    &TAG_INDENT,
    &TAG_HEADING,
    // Special
    &TAG_HR,
    &TAG_BR,
    &TAG_SPOILER,
    &TAG_ISPOILER,
    &TAG_USER,
    // Tables
    &TAG_TABLE,
    &TAG_TR,
    &TAG_TH,
    &TAG_TD,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_creation() {
        let registry = TagRegistry::new();
        assert!(registry.is_known("b"));
        assert!(registry.is_known("bold"));
        assert!(registry.is_known("B"));
        assert!(registry.is_known("BOLD"));
    }

    #[test]
    fn registry_lookup() {
        let registry = TagRegistry::new();

        let tag = registry.get("b").unwrap();
        assert_eq!(tag.name, "b");
        assert_eq!(tag.html_tag, Some("strong"));

        let tag = registry.get("BOLD").unwrap();
        assert_eq!(tag.name, "b"); // Alias resolves to canonical
    }

    #[test]
    fn registry_unknown_tag() {
        let registry = TagRegistry::new();
        assert!(registry.get("unknowntag").is_none());
        assert!(!registry.is_known("unknowntag"));
    }

    #[test]
    fn tag_type_checks() {
        assert!(TAG_BOLD.is_inline());
        assert!(!TAG_BOLD.is_block());
        assert!(!TAG_BOLD.is_verbatim());
        assert!(!TAG_BOLD.is_self_closing());

        assert!(TAG_QUOTE.is_block());
        assert!(!TAG_QUOTE.is_inline());

        assert!(TAG_CODE.is_verbatim());
        assert!(TAG_PLAIN.is_verbatim());

        assert!(TAG_HR.is_self_closing());
        assert!(TAG_BR.is_self_closing());
    }

    #[test]
    fn tag_option_requirements() {
        assert!(!TAG_BOLD.option_required);
        assert!(!TAG_BOLD.option_allowed);

        assert!(TAG_COLOR.option_required);
        assert!(TAG_COLOR.option_allowed);

        assert!(!TAG_URL.option_required);
        assert!(TAG_URL.option_allowed);
    }

    #[test]
    fn tag_forbidden_ancestors() {
        assert!(TAG_URL.forbidden_ancestors.contains(&"url"));
        assert!(TAG_URL.forbidden_ancestors.contains(&"email"));
        assert!(TAG_EMAIL.forbidden_ancestors.contains(&"url"));
    }

    #[test]
    fn tag_required_parents() {
        assert!(TAG_LIST_ITEM.required_parents.contains(&"list"));
        assert!(TAG_TR.required_parents.contains(&"table"));
        assert!(TAG_TD.required_parents.contains(&"tr"));
        assert!(TAG_TH.required_parents.contains(&"tr"));
    }

    #[test]
    fn verbatim_tags_stop_processing() {
        assert!(TAG_CODE.stop_smilies);
        assert!(TAG_CODE.stop_auto_link);

        assert!(TAG_PLAIN.stop_smilies);
        assert!(TAG_PLAIN.stop_auto_link);

        assert!(TAG_IMG.stop_smilies);
        assert!(TAG_IMG.stop_auto_link);
    }

    #[test]
    fn all_standard_tags_valid() {
        for tag in STANDARD_TAGS {
            assert!(!tag.name.is_empty(), "Tag name should not be empty");

            // Self-closing tags shouldn't have required content (except list item)
            if tag.tag_type == TagType::SelfClosing && tag.name != "*" {
                assert!(
                    !tag.has_content || tag.name == "*",
                    "Self-closing tag {} shouldn't have content",
                    tag.name
                );
            }
        }
    }

    #[test]
    fn registry_iter() {
        let registry = TagRegistry::new();
        let tags: Vec<_> = registry.iter().collect();

        // Should have all unique tags (not counting aliases)
        assert!(tags.len() >= 30);

        // Should include common tags
        let names: Vec<_> = tags.iter().map(|t| t.name).collect();
        assert!(names.contains(&"b"));
        assert!(names.contains(&"i"));
        assert!(names.contains(&"url"));
        assert!(names.contains(&"quote"));
        assert!(names.contains(&"code"));
    }

    #[test]
    fn tag_aliases() {
        let registry = TagRegistry::new();

        // Bold aliases
        assert_eq!(registry.get("b").unwrap().name, "b");
        assert_eq!(registry.get("bold").unwrap().name, "b");

        // Strikethrough aliases
        assert_eq!(registry.get("s").unwrap().name, "s");
        assert_eq!(registry.get("strike").unwrap().name, "s");
        assert_eq!(registry.get("strikethrough").unwrap().name, "s");

        // URL aliases
        assert_eq!(registry.get("url").unwrap().name, "url");
        assert_eq!(registry.get("link").unwrap().name, "url");

        // Plain aliases
        assert_eq!(registry.get("plain").unwrap().name, "plain");
        assert_eq!(registry.get("noparse").unwrap().name, "plain");
        assert_eq!(registry.get("nobbc").unwrap().name, "plain");
    }

    #[test]
    fn newline_conversion_settings() {
        // Block code shouldn't convert newlines
        assert!(!TAG_CODE.convert_newlines);

        // Lists shouldn't convert newlines
        assert!(!TAG_LIST.convert_newlines);

        // Inline tags should convert newlines
        assert!(TAG_BOLD.convert_newlines);
        assert!(TAG_ITALIC.convert_newlines);
    }

    #[test]
    fn trim_content_settings() {
        assert!(TAG_QUOTE.trim_content);
        assert!(TAG_LIST.trim_content);
        assert!(TAG_IMG.trim_content);
        assert!(!TAG_BOLD.trim_content);
        assert!(!TAG_CODE.trim_content);
    }
}
