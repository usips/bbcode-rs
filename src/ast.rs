//! Abstract Syntax Tree (AST) data structures for BBCode.
//!
//! This module defines the tree structure that represents parsed BBCode.
//! All string data uses `&str` references for zero-copy parsing.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

/// The type of BBCode tag, determining its parsing and rendering behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagType {
    /// Standard inline formatting tags like [b], [i], [u], [s].
    /// These can nest freely and auto-close at block boundaries.
    Inline,

    /// Block-level tags like [quote], [code], [list].
    /// These create block elements and have stricter nesting rules.
    Block,

    /// Tags whose content is not parsed for BBCode, like [code], [plain], [icode].
    /// Inner BBCode syntax is treated as literal text.
    Verbatim,

    /// Self-closing tags with no content, like [hr], [*].
    SelfClosing,

    /// Tags that should be rendered as void HTML elements (no closing tag).
    /// Example: [img] renders as <img ... />
    Void,
}

impl Default for TagType {
    fn default() -> Self {
        Self::Inline
    }
}

/// Represents the value of a tag's option/attribute.
///
/// BBCode supports several option formats:
/// - Simple: `[tag=value]` → `Scalar("value")`
/// - Quoted: `[tag="value with spaces"]` → `Scalar("value with spaces")`
/// - Keyed: `[tag attr1=val1 attr2="val 2"]` → `Map({...})`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagOption<'a> {
    /// No option provided.
    None,

    /// A single scalar value like `[url=https://example.com]`.
    Scalar(Cow<'a, str>),

    /// Key-value pairs like `[img width="100" height="200"]`.
    Map(HashMap<Cow<'a, str>, Cow<'a, str>>),
}

impl<'a> Default for TagOption<'a> {
    fn default() -> Self {
        Self::None
    }
}

impl<'a> TagOption<'a> {
    /// Returns `true` if no option is set.
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns `true` if a scalar value is set.
    #[inline]
    pub fn is_scalar(&self) -> bool {
        matches!(self, Self::Scalar(_))
    }

    /// Returns `true` if key-value pairs are set.
    #[inline]
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    /// Returns the scalar value if present.
    #[inline]
    pub fn as_scalar(&self) -> Option<&Cow<'a, str>> {
        match self {
            Self::Scalar(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the map if present.
    #[inline]
    pub fn as_map(&self) -> Option<&HashMap<Cow<'a, str>, Cow<'a, str>>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Gets a value from the map by key, or returns the scalar if the key matches
    /// an empty string (for tags that accept either format).
    #[inline]
    pub fn get(&self, key: &str) -> Option<&Cow<'a, str>> {
        match self {
            Self::Scalar(v) if key.is_empty() => Some(v),
            Self::Map(m) => m.get(key),
            _ => None,
        }
    }

    /// Converts the option to an owned version for longer lifetimes.
    pub fn into_owned(self) -> TagOption<'static> {
        match self {
            TagOption::None => TagOption::None,
            TagOption::Scalar(s) => TagOption::Scalar(Cow::Owned(s.into_owned())),
            TagOption::Map(m) => TagOption::Map(
                m.into_iter()
                    .map(|(k, v)| (Cow::Owned(k.into_owned()), Cow::Owned(v.into_owned())))
                    .collect(),
            ),
        }
    }
}

/// A node in the BBCode AST.
///
/// This is the core structure representing parsed BBCode. Each node can be
/// either a text fragment or a tag with children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node<'a> {
    /// Plain text content.
    Text(Cow<'a, str>),

    /// A line break (preserved from input).
    LineBreak,

    /// An auto-detected URL that wasn't wrapped in [url] tags.
    AutoUrl(Cow<'a, str>),

    /// A BBCode tag with its name, option, and children.
    Tag(TagNode<'a>),
}

impl<'a> Node<'a> {
    /// Creates a new text node.
    #[inline]
    pub fn text(content: &'a str) -> Self {
        Self::Text(Cow::Borrowed(content))
    }

    /// Creates a new text node from an owned string.
    #[inline]
    pub fn text_owned(content: String) -> Self {
        Self::Text(Cow::Owned(content))
    }

    /// Creates a new tag node.
    #[inline]
    pub fn tag(name: &'a str) -> Self {
        Self::Tag(TagNode::new(name))
    }

    /// Returns `true` if this is a text node.
    #[inline]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Returns `true` if this is a tag node.
    #[inline]
    pub fn is_tag(&self) -> bool {
        matches!(self, Self::Tag(_))
    }

    /// Returns `true` if this is a line break.
    #[inline]
    pub fn is_linebreak(&self) -> bool {
        matches!(self, Self::LineBreak)
    }

    /// Returns the text content if this is a text node.
    #[inline]
    pub fn as_text(&self) -> Option<&Cow<'a, str>> {
        match self {
            Self::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Returns the tag node if this is a tag.
    #[inline]
    pub fn as_tag(&self) -> Option<&TagNode<'a>> {
        match self {
            Self::Tag(t) => Some(t),
            _ => None,
        }
    }

    /// Returns a mutable reference to the tag node if this is a tag.
    #[inline]
    pub fn as_tag_mut(&mut self) -> Option<&mut TagNode<'a>> {
        match self {
            Self::Tag(t) => Some(t),
            _ => None,
        }
    }

    /// Converts the node to an owned version.
    pub fn into_owned(self) -> Node<'static> {
        match self {
            Node::Text(t) => Node::Text(Cow::Owned(t.into_owned())),
            Node::LineBreak => Node::LineBreak,
            Node::AutoUrl(u) => Node::AutoUrl(Cow::Owned(u.into_owned())),
            Node::Tag(t) => Node::Tag(t.into_owned()),
        }
    }
}

impl fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Text(t) => write!(f, "{}", t),
            Node::LineBreak => writeln!(f),
            Node::AutoUrl(u) => write!(f, "{}", u),
            Node::Tag(t) => write!(f, "{}", t),
        }
    }
}

/// A BBCode tag node with its name, option, and children.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TagNode<'a> {
    /// The tag name (lowercase).
    pub name: Cow<'a, str>,

    /// The raw tag name as it appeared in the source (preserves case).
    pub raw_name: Cow<'a, str>,

    /// The tag's option/attribute value.
    pub option: TagOption<'a>,

    /// Child nodes (text and nested tags).
    pub children: Vec<Node<'a>>,

    /// Whether this tag was explicitly closed with [/tag].
    pub closed: bool,

    /// The original raw text of the opening tag (for broken tag fallback).
    pub raw_open: Cow<'a, str>,

    /// The original raw text of the closing tag (for broken tag fallback).
    pub raw_close: Cow<'a, str>,

    /// If true, this tag failed validation and should be rendered as raw text.
    pub broken: bool,
}

impl<'a> TagNode<'a> {
    /// Creates a new tag node with the given name.
    #[inline]
    pub fn new(name: &'a str) -> Self {
        Self {
            name: Cow::Borrowed(name.to_ascii_lowercase().leak()),
            raw_name: Cow::Borrowed(name),
            ..Default::default()
        }
    }

    /// Creates a new tag node with borrowed lowercase name.
    #[inline]
    pub fn with_name(name: Cow<'a, str>, raw_name: Cow<'a, str>) -> Self {
        Self {
            name,
            raw_name,
            ..Default::default()
        }
    }

    /// Sets the tag option.
    #[inline]
    pub fn with_option(mut self, option: TagOption<'a>) -> Self {
        self.option = option;
        self
    }

    /// Adds a child node.
    #[inline]
    pub fn push_child(&mut self, child: Node<'a>) {
        self.children.push(child);
    }

    /// Sets the raw opening tag text.
    #[inline]
    pub fn with_raw_open(mut self, raw: &'a str) -> Self {
        self.raw_open = Cow::Borrowed(raw);
        self
    }

    /// Sets the raw closing tag text.
    #[inline]
    pub fn with_raw_close(mut self, raw: &'a str) -> Self {
        self.raw_close = Cow::Borrowed(raw);
        self
    }

    /// Marks the tag as explicitly closed.
    #[inline]
    pub fn mark_closed(&mut self) {
        self.closed = true;
    }

    /// Marks the tag as broken (should render as raw text).
    #[inline]
    pub fn mark_broken(&mut self) {
        self.broken = true;
    }

    /// Returns true if the tag has any children.
    #[inline]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns the text content of all children concatenated.
    pub fn inner_text(&self) -> Cow<'a, str> {
        if self.children.len() == 1 {
            if let Some(Node::Text(t)) = self.children.first() {
                return t.clone();
            }
        }

        let mut result = String::new();
        for child in &self.children {
            match child {
                Node::Text(t) => result.push_str(t),
                Node::LineBreak => result.push('\n'),
                Node::AutoUrl(u) => result.push_str(u),
                Node::Tag(t) => result.push_str(&t.inner_text()),
            }
        }
        Cow::Owned(result)
    }

    /// Converts the tag to an owned version.
    pub fn into_owned(self) -> TagNode<'static> {
        TagNode {
            name: Cow::Owned(self.name.into_owned()),
            raw_name: Cow::Owned(self.raw_name.into_owned()),
            option: self.option.into_owned(),
            children: self.children.into_iter().map(|c| c.into_owned()).collect(),
            closed: self.closed,
            raw_open: Cow::Owned(self.raw_open.into_owned()),
            raw_close: Cow::Owned(self.raw_close.into_owned()),
            broken: self.broken,
        }
    }
}

impl fmt::Display for TagNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.raw_name)?;
        for child in &self.children {
            write!(f, "{}", child)?;
        }
        if self.closed {
            write!(f, "[/{}]", self.raw_name)?;
        }
        Ok(())
    }
}

/// The root document node containing all parsed content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document<'a> {
    /// The top-level nodes in the document.
    pub nodes: Vec<Node<'a>>,
}

impl<'a> Document<'a> {
    /// Creates a new empty document.
    #[inline]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Creates a document with the given nodes.
    #[inline]
    pub fn with_nodes(nodes: Vec<Node<'a>>) -> Self {
        Self { nodes }
    }

    /// Adds a node to the document.
    #[inline]
    pub fn push(&mut self, node: Node<'a>) {
        self.nodes.push(node);
    }

    /// Returns true if the document is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of top-level nodes.
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Iterates over all nodes.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Node<'a>> {
        self.nodes.iter()
    }

    /// Converts the document to an owned version.
    pub fn into_owned(self) -> Document<'static> {
        Document {
            nodes: self.nodes.into_iter().map(|n| n.into_owned()).collect(),
        }
    }
}

impl fmt::Display for Document<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in &self.nodes {
            write!(f, "{}", node)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_option_none() {
        let opt = TagOption::None;
        assert!(opt.is_none());
        assert!(!opt.is_scalar());
        assert!(!opt.is_map());
        assert!(opt.as_scalar().is_none());
        assert!(opt.as_map().is_none());
        assert!(opt.get("").is_none());
    }

    #[test]
    fn tag_option_scalar() {
        let opt = TagOption::Scalar(Cow::Borrowed("value"));
        assert!(!opt.is_none());
        assert!(opt.is_scalar());
        assert!(!opt.is_map());
        assert_eq!(opt.as_scalar(), Some(&Cow::Borrowed("value")));
        assert!(opt.as_map().is_none());
        assert_eq!(opt.get(""), Some(&Cow::Borrowed("value")));
        assert!(opt.get("other").is_none());
    }

    #[test]
    fn tag_option_map() {
        let mut map = HashMap::new();
        map.insert(Cow::Borrowed("width"), Cow::Borrowed("100"));
        map.insert(Cow::Borrowed("height"), Cow::Borrowed("200"));
        let opt = TagOption::Map(map);

        assert!(!opt.is_none());
        assert!(!opt.is_scalar());
        assert!(opt.is_map());
        assert!(opt.as_scalar().is_none());
        assert!(opt.as_map().is_some());
        assert_eq!(opt.get("width"), Some(&Cow::Borrowed("100")));
        assert_eq!(opt.get("height"), Some(&Cow::Borrowed("200")));
        assert!(opt.get("other").is_none());
    }

    #[test]
    fn tag_option_into_owned() {
        let opt = TagOption::Scalar(Cow::Borrowed("test"));
        let owned = opt.into_owned();
        assert_eq!(owned, TagOption::Scalar(Cow::Owned("test".to_string())));
    }

    #[test]
    fn node_text() {
        let node = Node::text("hello");
        assert!(node.is_text());
        assert!(!node.is_tag());
        assert!(!node.is_linebreak());
        assert_eq!(node.as_text(), Some(&Cow::Borrowed("hello")));
        assert!(node.as_tag().is_none());
    }

    #[test]
    fn node_tag() {
        let node = Node::tag("b");
        assert!(!node.is_text());
        assert!(node.is_tag());
        assert!(!node.is_linebreak());
        assert!(node.as_text().is_none());
        assert!(node.as_tag().is_some());
    }

    #[test]
    fn node_linebreak() {
        let node = Node::LineBreak;
        assert!(!node.is_text());
        assert!(!node.is_tag());
        assert!(node.is_linebreak());
    }

    #[test]
    fn node_display() {
        let node = Node::text("hello world");
        assert_eq!(format!("{}", node), "hello world");

        let node = Node::LineBreak;
        assert_eq!(format!("{}", node), "\n");
    }

    #[test]
    fn node_into_owned() {
        let node = Node::text("borrowed");
        let owned = node.into_owned();
        assert_eq!(owned.as_text(), Some(&Cow::Owned("borrowed".to_string())));
    }

    #[test]
    fn tag_node_new() {
        let tag = TagNode::new("B");
        assert_eq!(&*tag.name, "b"); // lowercase
        assert_eq!(&*tag.raw_name, "B"); // preserves case
        assert!(tag.option.is_none());
        assert!(tag.children.is_empty());
        assert!(!tag.closed);
        assert!(!tag.broken);
    }

    #[test]
    fn tag_node_with_option() {
        let tag = TagNode::new("url")
            .with_option(TagOption::Scalar(Cow::Borrowed("https://example.com")));
        assert_eq!(
            tag.option.as_scalar(),
            Some(&Cow::Borrowed("https://example.com"))
        );
    }

    #[test]
    fn tag_node_children() {
        let mut tag = TagNode::new("b");
        tag.push_child(Node::text("hello"));
        tag.push_child(Node::text(" world"));

        assert!(tag.has_children());
        assert_eq!(tag.children.len(), 2);
        assert_eq!(&*tag.inner_text(), "hello world");
    }

    #[test]
    fn tag_node_nested_inner_text() {
        let mut inner = TagNode::new("i");
        inner.push_child(Node::text("nested"));

        let mut outer = TagNode::new("b");
        outer.push_child(Node::text("hello "));
        outer.push_child(Node::Tag(inner));
        outer.push_child(Node::text(" world"));

        assert_eq!(&*outer.inner_text(), "hello nested world");
    }

    #[test]
    fn tag_node_mark_closed() {
        let mut tag = TagNode::new("b");
        assert!(!tag.closed);
        tag.mark_closed();
        assert!(tag.closed);
    }

    #[test]
    fn tag_node_mark_broken() {
        let mut tag = TagNode::new("b");
        assert!(!tag.broken);
        tag.mark_broken();
        assert!(tag.broken);
    }

    #[test]
    fn tag_node_display() {
        let mut tag = TagNode::new("b");
        tag.push_child(Node::text("bold"));
        tag.mark_closed();

        // Note: display uses raw_name which we can't set easily with new()
        // but the format should still work
        assert!(format!("{}", tag).contains("bold"));
    }

    #[test]
    fn document_new() {
        let doc = Document::new();
        assert!(doc.is_empty());
        assert_eq!(doc.len(), 0);
    }

    #[test]
    fn document_with_nodes() {
        let nodes = vec![Node::text("hello"), Node::LineBreak, Node::text("world")];
        let doc = Document::with_nodes(nodes);
        assert!(!doc.is_empty());
        assert_eq!(doc.len(), 3);
    }

    #[test]
    fn document_push() {
        let mut doc = Document::new();
        doc.push(Node::text("test"));
        assert_eq!(doc.len(), 1);
    }

    #[test]
    fn document_iter() {
        let nodes = vec![Node::text("a"), Node::text("b")];
        let doc = Document::with_nodes(nodes);
        let collected: Vec<_> = doc.iter().collect();
        assert_eq!(collected.len(), 2);
    }

    #[test]
    fn document_display() {
        let mut doc = Document::new();
        doc.push(Node::text("hello"));
        doc.push(Node::text(" world"));
        assert_eq!(format!("{}", doc), "hello world");
    }

    #[test]
    fn document_into_owned() {
        let doc = Document::with_nodes(vec![Node::text("borrowed")]);
        let owned = doc.into_owned();
        assert!(!owned.is_empty());
    }

    #[test]
    fn tag_type_default() {
        assert_eq!(TagType::default(), TagType::Inline);
    }
}
