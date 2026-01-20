//! BBCode parser that converts tokens into an AST.
//!
//! This module contains the parser that takes tokenized BBCode and
//! builds a tree structure representing the document.

use std::borrow::Cow;
use std::collections::HashMap;

use crate::ast::{Document, Node, TagNode, TagOption};
use crate::tags::{CustomTagDef, ResolvedTag, TagRegistry};
use crate::tokenizer::{tokenize, tokenize_until_close, Token};

/// Maximum nesting depth to prevent stack overflow.
const MAX_NESTING_DEPTH: usize = 50;

/// Configuration for the BBCode parser.
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum allowed nesting depth.
    pub max_depth: usize,

    /// Whether to auto-detect URLs in text.
    pub auto_link: bool,

    /// Whether to convert line breaks to <br>.
    pub convert_linebreaks: bool,

    /// Whether unknown tags should be treated as text.
    pub allow_unknown_tags: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_depth: MAX_NESTING_DEPTH,
            auto_link: true,
            convert_linebreaks: true,
            allow_unknown_tags: true,
        }
    }
}

/// The BBCode parser.
pub struct Parser {
    /// Tag registry for looking up tag definitions.
    registry: TagRegistry,

    /// Parser configuration.
    config: ParserConfig,
}

impl Parser {
    /// Creates a new parser with default settings.
    pub fn new() -> Self {
        Self {
            registry: TagRegistry::new(),
            config: ParserConfig::default(),
        }
    }

    /// Creates a new parser with custom configuration.
    pub fn with_config(config: ParserConfig) -> Self {
        Self {
            registry: TagRegistry::new(),
            config,
        }
    }

    /// Creates a new parser with a custom tag registry.
    pub fn with_registry(registry: TagRegistry) -> Self {
        Self {
            registry,
            config: ParserConfig::default(),
        }
    }

    /// Creates a new parser with custom configuration and registry.
    pub fn with_config_and_registry(config: ParserConfig, registry: TagRegistry) -> Self {
        Self { registry, config }
    }

    /// Registers a custom tag definition.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bbcode::{Parser, CustomTagDef, TagType};
    ///
    /// let mut parser = Parser::new();
    /// parser.register_custom_tag(CustomTagDef {
    ///     name: "attach".into(),
    ///     aliases: vec!["attachment".into()],
    ///     tag_type: TagType::Inline,
    ///     option_allowed: true,
    ///     trim_content: true,
    ///     ..Default::default()
    /// });
    /// ```
    pub fn register_custom_tag(&mut self, tag: CustomTagDef) {
        self.registry.register_custom(tag);
    }

    /// Returns a reference to the tag registry.
    pub fn registry(&self) -> &TagRegistry {
        &self.registry
    }

    /// Parses BBCode input into a document AST.
    pub fn parse<'a>(&self, input: &'a str) -> Document<'a> {
        let tokens = tokenize(input);
        self.parse_tokens(&tokens, input, 0)
    }

    /// Parses tokens into a document, tracking depth.
    fn parse_tokens<'a>(
        &self,
        tokens: &[Token<'a>],
        original_input: &'a str,
        depth: usize,
    ) -> Document<'a> {
        let mut doc = Document::new();
        let mut stack: Vec<TagNode<'a>> = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let token = &tokens[i];

            match token {
                Token::Text(text) => {
                    let node = Node::Text(Cow::Borrowed(*text));
                    self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                }

                Token::LineBreak(_raw) => {
                    let node = Node::LineBreak;
                    self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                }

                Token::Url(url) => {
                    let node = Node::AutoUrl(Cow::Borrowed(*url));
                    self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                }

                Token::OpenTag { raw, name, arg } => {
                    let lower_name = name.to_ascii_lowercase();

                    // Look up the tag definition (static or custom)
                    if let Some(resolved) = self.registry.resolve(&lower_name) {
                        // Check nesting depth
                        if depth + stack.len() >= self.config.max_depth {
                            // Too deep, treat as text
                            let node = Node::Text(Cow::Borrowed(*raw));
                            self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                            i += 1;
                            continue;
                        }

                        // Check forbidden ancestors
                        if !self.check_ancestors_resolved(&stack, &resolved) {
                            // Invalid nesting, treat as text
                            let node = Node::Text(Cow::Borrowed(*raw));
                            self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                            i += 1;
                            continue;
                        }

                        // Check required parents
                        if !self.check_required_parents_resolved(&stack, &resolved) {
                            // Missing required parent, treat as text
                            let node = Node::Text(Cow::Borrowed(*raw));
                            self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                            i += 1;
                            continue;
                        }

                        // Parse the option
                        let option = self.parse_option_resolved(*arg, &resolved);

                        // Check if option is required but missing
                        if resolved.option_required() && option.is_none() {
                            // Missing required option, treat as text
                            let node = Node::Text(Cow::Borrowed(*raw));
                            self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                            i += 1;
                            continue;
                        }

                        // Create the tag node
                        let tag_name_for_close = lower_name.clone();
                        let mut tag_node = TagNode {
                            name: Cow::Owned(lower_name),
                            raw_name: Cow::Borrowed(*name),
                            option,
                            children: Vec::new(),
                            closed: false,
                            raw_open: Cow::Borrowed(*raw),
                            raw_close: Cow::Borrowed(""),
                            broken: false,
                        };

                        // Handle self-closing tags
                        if resolved.is_self_closing() {
                            // List items are special - they need content until next [*] or [/list]
                            if resolved.name() == "*" {
                                // Collect content until next [*] or [/list]
                                i += 1;
                                while i < tokens.len() {
                                    match &tokens[i] {
                                        Token::OpenTag { name, .. }
                                            if name.eq_ignore_ascii_case("*") =>
                                        {
                                            break;
                                        }
                                        Token::CloseTag { name, .. }
                                            if name.eq_ignore_ascii_case("list") =>
                                        {
                                            break;
                                        }
                                        Token::Text(t) => {
                                            tag_node.children.push(Node::Text(Cow::Borrowed(*t)));
                                        }
                                        Token::LineBreak(_) => {
                                            tag_node.children.push(Node::LineBreak);
                                        }
                                        Token::Url(u) => {
                                            tag_node
                                                .children
                                                .push(Node::AutoUrl(Cow::Borrowed(*u)));
                                        }
                                        _ => {
                                            // Recursively parse nested content
                                            let remaining = &tokens[i..];
                                            if let Some((node, consumed)) = self.parse_single_tag(
                                                remaining,
                                                original_input,
                                                depth + stack.len(),
                                            ) {
                                                tag_node.children.push(node);
                                                i += consumed;
                                                continue;
                                            }
                                        }
                                    }
                                    i += 1;
                                }
                                tag_node.mark_closed();
                                let node = Node::Tag(tag_node);
                                self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                                continue;
                            } else {
                                // Regular self-closing tag
                                tag_node.mark_closed();
                                let node = Node::Tag(tag_node);
                                self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                            }
                        }
                        // Handle verbatim tags (content not parsed)
                        else if resolved.is_verbatim() {
                            // Find the closing tag in the remaining input
                            let remaining_start = self.find_token_end(original_input, *raw);
                            if let Some(start_pos) = remaining_start {
                                let remaining = &original_input[start_pos..];
                                let (content, close_tag, _rest) =
                                    tokenize_until_close(remaining, &tag_name_for_close);

                                if !close_tag.is_empty() {
                                    tag_node.children.push(Node::Text(Cow::Borrowed(content)));
                                    tag_node.raw_close = Cow::Borrowed(close_tag);
                                    tag_node.mark_closed();

                                    // Skip tokens until after the close tag
                                    let close_end = start_pos + content.len() + close_tag.len();
                                    i = self.skip_tokens_until_pos(
                                        &tokens,
                                        i + 1,
                                        original_input,
                                        close_end,
                                    );

                                    let node = Node::Tag(tag_node);
                                    self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                                    continue;
                                }
                            }

                            // No close tag found, push to stack like normal
                            stack.push(tag_node);
                        }
                        // Regular tag with content
                        else {
                            stack.push(tag_node);
                        }
                    } else if self.config.allow_unknown_tags {
                        // Unknown tag, treat as text
                        let node = Node::Text(Cow::Borrowed(*raw));
                        self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                    } else {
                        // Unknown tag, treat as text
                        let node = Node::Text(Cow::Borrowed(*raw));
                        self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                    }
                }

                Token::CloseTag { raw, name } => {
                    let lower_name = name.to_ascii_lowercase();

                    // Find matching open tag in stack
                    if let Some(pos) = self.find_matching_open_tag(&stack, &lower_name) {
                        // Close all tags from pos to end
                        let mut closed = stack.split_off(pos);

                        if let Some(mut tag_node) = closed.first_mut() {
                            // The matching tag is at the front of closed list
                            tag_node.raw_close = Cow::Borrowed(*raw);
                            tag_node.mark_closed();

                            // Auto-close any intervening tags (XenForo behavior)
                            // Intervening tags are everything after the first element
                            for unclosed in closed.iter_mut().skip(1).rev() {
                                unclosed.mark_closed();
                            }

                            // Build the nested structure from inside out
                            let mut result = closed.pop().unwrap(); // Start with innermost
                            while let Some(mut parent) = closed.pop() {
                                parent.children.push(Node::Tag(result));
                                result = parent;
                            }

                            let node = Node::Tag(result);
                            self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                        }
                    } else {
                        // No matching open tag, treat close tag as text
                        let node = Node::Text(Cow::Borrowed(*raw));
                        self.push_to_stack_or_doc(&mut stack, &mut doc, node);
                    }
                }
            }

            i += 1;
        }

        // Close any remaining open tags
        while let Some(tag_node) = stack.pop() {
            // Unclosed tag - mark as closed but missing close tag
            let node = Node::Tag(tag_node);
            if let Some(parent) = stack.last_mut() {
                parent.children.push(node);
            } else {
                doc.push(node);
            }
        }

        doc
    }

    /// Parses a single tag and returns the node and number of tokens consumed.
    fn parse_single_tag<'a>(
        &self,
        tokens: &[Token<'a>],
        _original_input: &'a str,
        _depth: usize,
    ) -> Option<(Node<'a>, usize)> {
        if tokens.is_empty() {
            return None;
        }

        match &tokens[0] {
            Token::OpenTag { raw, name, arg } => {
                let lower_name = name.to_ascii_lowercase();
                let resolved = self.registry.resolve(&lower_name)?;

                let option = self.parse_option_resolved(*arg, &resolved);

                let mut tag_node = TagNode {
                    name: Cow::Owned(lower_name.clone()),
                    raw_name: Cow::Borrowed(*name),
                    option,
                    children: Vec::new(),
                    closed: false,
                    raw_open: Cow::Borrowed(*raw),
                    raw_close: Cow::Borrowed(""),
                    broken: false,
                };

                // Find matching close tag
                let mut consumed = 1;
                let mut nesting = 1;

                while consumed < tokens.len() && nesting > 0 {
                    match &tokens[consumed] {
                        Token::OpenTag { name: n, .. } if n.eq_ignore_ascii_case(&lower_name) => {
                            nesting += 1;
                        }
                        Token::CloseTag {
                            name: n,
                            raw: close_raw,
                        } if n.eq_ignore_ascii_case(&lower_name) => {
                            nesting -= 1;
                            if nesting == 0 {
                                tag_node.raw_close = Cow::Borrowed(*close_raw);
                                tag_node.mark_closed();
                                consumed += 1;
                                break;
                            }
                        }
                        Token::Text(t) => {
                            tag_node.children.push(Node::Text(Cow::Borrowed(*t)));
                        }
                        Token::LineBreak(_) => {
                            tag_node.children.push(Node::LineBreak);
                        }
                        Token::Url(u) => {
                            tag_node.children.push(Node::AutoUrl(Cow::Borrowed(*u)));
                        }
                        _ => {}
                    }
                    consumed += 1;
                }

                Some((Node::Tag(tag_node), consumed))
            }
            _ => None,
        }
    }

    /// Pushes a node to the current context (stack top or document root).
    fn push_to_stack_or_doc<'a>(
        &self,
        stack: &mut Vec<TagNode<'a>>,
        doc: &mut Document<'a>,
        node: Node<'a>,
    ) {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        } else {
            doc.push(node);
        }
    }

    /// Checks if the tag is allowed based on forbidden ancestors (for resolved tags).
    fn check_ancestors_resolved(&self, stack: &[TagNode], resolved: &ResolvedTag) -> bool {
        for ancestor in stack {
            if resolved.is_ancestor_forbidden(&ancestor.name) {
                return false;
            }
        }
        true
    }

    /// Checks if required parent tags are present (for resolved tags).
    fn check_required_parents_resolved(&self, stack: &[TagNode], resolved: &ResolvedTag) -> bool {
        let names: Vec<String> = stack.iter().map(|t| t.name.to_string()).collect();
        resolved.has_required_parent(&names)
    }

    /// Finds the position of a matching open tag in the stack.
    fn find_matching_open_tag(&self, stack: &[TagNode], name: &str) -> Option<usize> {
        stack
            .iter()
            .rposition(|t| t.name.eq_ignore_ascii_case(name))
    }

    /// Parses a tag option string into a TagOption (for resolved tags).
    fn parse_option_resolved<'a>(
        &self,
        arg: Option<&'a str>,
        _resolved: &ResolvedTag,
    ) -> TagOption<'a> {
        match arg {
            None => TagOption::None,
            Some(s) if s.is_empty() => TagOption::None,
            Some(s) => {
                // Try to parse as key-value pairs if it looks like key=value format.
                // Key-value format starts with an identifier (alphabetic) followed by =
                // This distinguishes [attach width=100] from [url=http://example.com?foo=bar]
                if self.looks_like_keyed_options(s) {
                    if let Some(map) = self.parse_keyed_options(s) {
                        return TagOption::Map(map);
                    }
                }
                // Simple scalar value
                TagOption::Scalar(Cow::Borrowed(s))
            }
        }
    }

    /// Checks if a string looks like key=value format (vs a scalar value).
    /// Key-value format: starts with alphabetic chars, then =
    /// Scalar format: starts with value directly (URL, number, quoted string, etc.)
    fn looks_like_keyed_options(&self, s: &str) -> bool {
        // Find the first non-alphabetic character
        let first_non_alpha = s.find(|c: char| !c.is_ascii_alphabetic());

        match first_non_alpha {
            // If the first non-alpha is '=', and there's at least one alpha char before it,
            // this looks like key=value
            Some(pos) if pos > 0 && s.as_bytes().get(pos) == Some(&b'=') => true,
            _ => false,
        }
    }

    /// Parses keyed options like `width=100 height="200"`.
    fn parse_keyed_options<'a>(
        &self,
        input: &'a str,
    ) -> Option<HashMap<Cow<'a, str>, Cow<'a, str>>> {
        let mut map = HashMap::new();
        let mut remaining = input.trim();

        while !remaining.is_empty() {
            // Find key
            let eq_pos = remaining.find('=')?;
            let key = remaining[..eq_pos].trim();
            remaining = remaining[eq_pos + 1..].trim_start();

            // Find value
            let (value, rest) = if remaining.starts_with('"') {
                // Quoted value
                let end = remaining[1..].find('"')?;
                let val = &remaining[1..=end];
                (val, remaining[end + 2..].trim_start())
            } else if remaining.starts_with('\'') {
                let end = remaining[1..].find('\'')?;
                let val = &remaining[1..=end];
                (val, remaining[end + 2..].trim_start())
            } else {
                // Unquoted value - until space
                let end = remaining.find(' ').unwrap_or(remaining.len());
                (&remaining[..end], remaining[end..].trim_start())
            };

            map.insert(Cow::Borrowed(key), Cow::Borrowed(value));
            remaining = rest;
        }

        if map.is_empty() {
            None
        } else {
            Some(map)
        }
    }

    /// Finds the end position of a token's raw text in the original input.
    fn find_token_end(&self, input: &str, raw: &str) -> Option<usize> {
        let raw_start = raw.as_ptr() as usize;
        let input_start = input.as_ptr() as usize;
        if raw_start >= input_start {
            let offset = raw_start - input_start;
            Some(offset + raw.len())
        } else {
            None
        }
    }

    /// Skips tokens until we reach a position past the given offset.
    fn skip_tokens_until_pos(
        &self,
        tokens: &[Token],
        start: usize,
        input: &str,
        target_pos: usize,
    ) -> usize {
        let input_start = input.as_ptr() as usize;
        let mut i = start;

        while i < tokens.len() {
            let raw = tokens[i].as_raw();
            let raw_start = raw.as_ptr() as usize - input_start;
            if raw_start >= target_pos {
                return i;
            }
            i += 1;
        }

        tokens.len()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Basic Parsing Tests ====================

    #[test]
    fn parse_empty() {
        let parser = Parser::new();
        let doc = parser.parse("");
        assert!(doc.is_empty());
    }

    #[test]
    fn parse_plain_text() {
        let parser = Parser::new();
        let doc = parser.parse("Hello, world!");
        assert_eq!(doc.len(), 1);
        assert!(doc.nodes[0].is_text());
    }

    #[test]
    fn parse_simple_tag() {
        let parser = Parser::new();
        let doc = parser.parse("[b]Bold[/b]");

        assert_eq!(doc.len(), 1);
        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.name, "b");
        assert!(tag.closed);
        assert_eq!(tag.children.len(), 1);
    }

    #[test]
    fn parse_nested_tags() {
        let parser = Parser::new();
        let doc = parser.parse("[b][i]Bold Italic[/i][/b]");

        assert_eq!(doc.len(), 1);
        let outer = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*outer.name, "b");

        let inner = outer.children[0].as_tag().unwrap();
        assert_eq!(&*inner.name, "i");
        assert_eq!(&*inner.inner_text(), "Bold Italic");
    }

    #[test]
    fn parse_multiple_tags() {
        let parser = Parser::new();
        let doc = parser.parse("[b]one[/b] [i]two[/i]");

        assert_eq!(doc.len(), 3); // tag, space text, tag
    }

    // ==================== Tag Option Tests ====================

    #[test]
    fn parse_tag_with_scalar_option() {
        let parser = Parser::new();
        let doc = parser.parse("[color=red]Red[/color]");

        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.name, "color");
        assert!(tag.option.is_scalar());
        assert_eq!(tag.option.as_scalar().unwrap().as_ref(), "red");
    }

    #[test]
    fn parse_tag_with_url_option() {
        let parser = Parser::new();
        let doc = parser.parse("[url=https://example.com]Link[/url]");

        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.name, "url");
        assert_eq!(
            tag.option.as_scalar().unwrap().as_ref(),
            "https://example.com"
        );
    }

    #[test]
    fn parse_tag_with_quoted_option() {
        let parser = Parser::new();
        let doc = parser.parse(r#"[quote="John Doe"]Quote[/quote]"#);

        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.name, "quote");
        assert_eq!(tag.option.as_scalar().unwrap().as_ref(), "John Doe");
    }

    #[test]
    fn parse_tag_with_keyed_options() {
        let parser = Parser::new();
        // Note: Standard BBCode uses [img=100x200] format for dimensions
        // The parser treats space-separated content as text within the tag
        let doc = parser.parse("[img=100x200]url[/img]");

        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.name, "img");
        assert_eq!(tag.option.as_scalar().unwrap().as_ref(), "100x200");
    }

    // ==================== Self-Closing Tag Tests ====================

    #[test]
    fn parse_hr() {
        let parser = Parser::new();
        let doc = parser.parse("Before[hr]After");

        assert_eq!(doc.len(), 3);
        assert!(doc.nodes[0].is_text());
        assert!(doc.nodes[1].is_tag());
        assert!(doc.nodes[2].is_text());

        let hr = doc.nodes[1].as_tag().unwrap();
        assert_eq!(&*hr.name, "hr");
        assert!(hr.closed);
    }

    #[test]
    fn parse_list_items() {
        let parser = Parser::new();
        let doc = parser.parse("[list][*]One[*]Two[/list]");

        let list = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*list.name, "list");

        // Should have two list items
        let items: Vec<_> = list
            .children
            .iter()
            .filter(|n| n.is_tag())
            .filter_map(|n| n.as_tag())
            .filter(|t| t.name == "*")
            .collect();

        assert_eq!(items.len(), 2);
    }

    // ==================== Verbatim Tag Tests ====================

    #[test]
    fn parse_code_verbatim() {
        let parser = Parser::new();
        let doc = parser.parse("[code][b]Not bold[/b][/code]");

        let code = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*code.name, "code");
        assert!(code.closed);

        // Content should be literal text, not parsed
        let content = code.inner_text();
        assert_eq!(&*content, "[b]Not bold[/b]");
    }

    #[test]
    fn parse_plain_verbatim() {
        let parser = Parser::new();
        let doc = parser.parse("[plain][i]Not italic[/i][/plain]");

        let plain = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*plain.name, "plain");
        assert!(plain.closed);

        let content = plain.inner_text();
        assert_eq!(&*content, "[i]Not italic[/i]");
    }

    // ==================== Case Insensitivity Tests ====================

    #[test]
    fn parse_case_insensitive_tags() {
        let parser = Parser::new();

        let doc = parser.parse("[B]Bold[/B]");
        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.name, "b");
        assert!(tag.closed);

        let doc = parser.parse("[B]Bold[/b]");
        let tag = doc.nodes[0].as_tag().unwrap();
        assert!(tag.closed);

        let doc = parser.parse("[b]Bold[/B]");
        let tag = doc.nodes[0].as_tag().unwrap();
        assert!(tag.closed);
    }

    // ==================== Unclosed Tag Tests ====================

    #[test]
    fn parse_unclosed_tag() {
        let parser = Parser::new();
        let doc = parser.parse("[b]Bold without close");

        assert_eq!(doc.len(), 1);
        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.name, "b");
        // Tag should still be captured, just not explicitly closed
    }

    #[test]
    fn parse_unopened_close_tag() {
        let parser = Parser::new();
        let doc = parser.parse("text[/b]more");

        // Close tag without open should be text
        assert!(doc.len() >= 2);
    }

    #[test]
    fn parse_mismatched_tags() {
        let parser = Parser::new();
        let doc = parser.parse("[b][i]text[/b][/i]");

        // Parser should handle this gracefully
        assert!(!doc.is_empty());
    }

    // ==================== Invalid Tag Tests ====================

    #[test]
    fn parse_unknown_tag() {
        let parser = Parser::new();
        let doc = parser.parse("[unknown]text[/unknown]");

        // Unknown tags should be treated as text
        assert!(!doc.is_empty());
    }

    #[test]
    fn parse_empty_brackets() {
        let parser = Parser::new();
        let doc = parser.parse("[]text[]");

        // Empty brackets should be text
        assert!(!doc.is_empty());
    }

    // ==================== Forbidden Ancestor Tests ====================

    #[test]
    fn parse_url_in_url_forbidden() {
        let parser = Parser::new();
        let doc = parser.parse("[url=http://a.com][url=http://b.com]inner[/url][/url]");

        // Nested URL should be treated as text due to forbidden ancestor
        let outer = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*outer.name, "url");
    }

    // ==================== Required Parent Tests ====================

    #[test]
    fn parse_list_item_outside_list() {
        let parser = Parser::new();
        let doc = parser.parse("[*]Item without list");

        // [*] outside [list] should be treated as text
        assert!(!doc.is_empty());
    }

    #[test]
    fn parse_tr_outside_table() {
        let parser = Parser::new();
        let doc = parser.parse("[tr][td]Cell[/td][/tr]");

        // [tr] outside [table] should be treated as text
        assert!(!doc.is_empty());
    }

    // ==================== Line Break Tests ====================

    #[test]
    fn parse_linebreaks() {
        let parser = Parser::new();
        let doc = parser.parse("Line 1\nLine 2\r\nLine 3");

        // Should have text, linebreak, text, linebreak, text
        assert_eq!(doc.len(), 5);
        assert!(doc.nodes[1].is_linebreak());
        assert!(doc.nodes[3].is_linebreak());
    }

    #[test]
    fn parse_linebreaks_in_tag() {
        let parser = Parser::new();
        let doc = parser.parse("[b]Line 1\nLine 2[/b]");

        let tag = doc.nodes[0].as_tag().unwrap();
        // Should contain linebreak
        assert!(tag.children.iter().any(|n| n.is_linebreak()));
    }

    // ==================== URL Auto-Detection Tests ====================

    #[test]
    fn parse_auto_url() {
        let parser = Parser::new();
        let doc = parser.parse("Visit https://example.com today!");

        // Should have auto-detected URL
        let has_url = doc.nodes.iter().any(|n| matches!(n, Node::AutoUrl(_)));
        assert!(has_url);
    }

    // ==================== Complex Document Tests ====================

    #[test]
    fn parse_complex_document() {
        let parser = Parser::new();
        let input = r#"[quote="Admin"]
Hello [b]everyone[/b]!

Check out https://example.com for more info.

[list]
[*]First item
[*]Second item
[/list]
[/quote]"#;

        let doc = parser.parse(input);

        assert_eq!(doc.len(), 1);
        let quote = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*quote.name, "quote");
        assert!(quote.closed);
    }

    #[test]
    fn parse_table() {
        let parser = Parser::new();
        let doc = parser.parse("[table][tr][td]Cell 1[/td][td]Cell 2[/td][/tr][/table]");

        let table = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*table.name, "table");
    }

    // ==================== Edge Cases ====================

    #[test]
    fn parse_deep_nesting() {
        let parser = Parser::new();
        let mut input = String::new();
        for _ in 0..30 {
            input.push_str("[b]");
        }
        input.push_str("deep");
        for _ in 0..30 {
            input.push_str("[/b]");
        }

        let doc = parser.parse(&input);
        assert!(!doc.is_empty());
    }

    #[test]
    fn parse_max_depth_exceeded() {
        let config = ParserConfig {
            max_depth: 5,
            ..Default::default()
        };
        let parser = Parser::with_config(config);

        let mut input = String::new();
        for _ in 0..10 {
            input.push_str("[b]");
        }
        input.push_str("text");
        for _ in 0..10 {
            input.push_str("[/b]");
        }

        let doc = parser.parse(&input);
        // Should still parse, but some tags will be treated as text
        assert!(!doc.is_empty());
    }

    #[test]
    fn parse_unicode() {
        let parser = Parser::new();
        let doc = parser.parse("[b]私は猫です[/b]");

        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.inner_text(), "私は猫です");
    }

    #[test]
    fn parse_emoji() {
        let parser = Parser::new();
        let doc = parser.parse("[b]🔥🎉[/b]");

        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.inner_text(), "🔥🎉");
    }

    // ==================== Parser Reuse Tests ====================

    #[test]
    fn parser_reuse() {
        let parser = Parser::new();

        let doc1 = parser.parse("[b]First[/b]");
        let doc2 = parser.parse("[i]Second[/i]");

        assert_eq!(doc1.len(), 1);
        assert_eq!(doc2.len(), 1);

        assert_eq!(&*doc1.nodes[0].as_tag().unwrap().name, "b");
        assert_eq!(&*doc2.nodes[0].as_tag().unwrap().name, "i");
    }

    // ==================== Raw Preservation Tests ====================

    #[test]
    fn parse_preserves_raw() {
        let parser = Parser::new();
        let doc = parser.parse("[B]Bold[/B]");

        let tag = doc.nodes[0].as_tag().unwrap();
        assert_eq!(&*tag.raw_open, "[B]");
        assert_eq!(&*tag.raw_close, "[/B]");
        assert_eq!(&*tag.raw_name, "B"); // Original case
        assert_eq!(&*tag.name, "b"); // Normalized
    }
}
