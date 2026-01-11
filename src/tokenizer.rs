//! Tokenizer for BBCode using winnow parser combinators.
//!
//! This module converts raw BBCode input into a stream of tokens using
//! zero-copy parsing. All string data references the original input.

use winnow::combinator::{alt, delimited};
use winnow::error::{ContextError, ErrMode};
use winnow::token::take_till;
use winnow::Parser;

/// A token produced by the tokenizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'a> {
    /// Plain text content.
    Text(&'a str),

    /// A line break (\n or \r\n).
    LineBreak(&'a str),

    /// An opening tag with name and optional argument.
    /// `[tag]` → `OpenTag { raw: "[tag]", name: "tag", arg: None }`
    /// `[tag=value]` → `OpenTag { raw: "[tag=value]", name: "tag", arg: Some("value") }`
    OpenTag {
        /// The complete raw text of the tag including brackets.
        raw: &'a str,
        /// The tag name.
        name: &'a str,
        /// The argument/option after the `=`, if present.
        arg: Option<&'a str>,
    },

    /// A closing tag.
    /// `[/tag]` → `CloseTag { raw: "[/tag]", name: "tag" }`
    CloseTag {
        /// The complete raw text of the tag.
        raw: &'a str,
        /// The tag name.
        name: &'a str,
    },

    /// An auto-detected URL.
    Url(&'a str),
}

impl<'a> Token<'a> {
    /// Returns the raw text of the token.
    #[inline]
    pub fn as_raw(&self) -> &'a str {
        match self {
            Self::Text(s) => s,
            Self::LineBreak(s) => s,
            Self::OpenTag { raw, .. } => raw,
            Self::CloseTag { raw, .. } => raw,
            Self::Url(s) => s,
        }
    }

    /// Returns `true` if this is a text token.
    #[inline]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Returns `true` if this is an opening tag.
    #[inline]
    pub fn is_open_tag(&self) -> bool {
        matches!(self, Self::OpenTag { .. })
    }

    /// Returns `true` if this is a closing tag.
    #[inline]
    pub fn is_close_tag(&self) -> bool {
        matches!(self, Self::CloseTag { .. })
    }

    /// Returns `true` if this is a line break.
    #[inline]
    pub fn is_linebreak(&self) -> bool {
        matches!(self, Self::LineBreak(_))
    }

    /// Returns `true` if this is a URL.
    #[inline]
    pub fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }
}

type PResult<O> = Result<O, ErrMode<ContextError>>;

/// Tokenizes BBCode input into a vector of tokens.
///
/// This is a zero-copy operation - all string data in tokens reference
/// the original input string.
///
/// # Example
/// ```
/// use bbcode::tokenizer::tokenize;
///
/// let tokens = tokenize("[b]Hello[/b]");
/// assert_eq!(tokens.len(), 3);
/// ```
pub fn tokenize(input: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut remaining = input;
    let original_input = input;

    while !remaining.is_empty() {
        let start_offset = original_input.len() - remaining.len();
        
        match parse_token(&mut remaining, original_input, start_offset) {
            Ok(token) => {
                // Skip null/empty tokens
                if !matches!(&token, Token::Text(s) if s.is_empty()) {
                    tokens.push(token);
                }
            }
            Err(_) => {
                // On error, consume one character as text and continue
                if let Some(c) = remaining.chars().next() {
                    let char_len = c.len_utf8();
                    let char_str = &original_input[start_offset..start_offset + char_len];
                    remaining = &remaining[char_len..];
                    
                    // Merge with previous text token if possible
                    if let Some(Token::Text(prev)) = tokens.last_mut() {
                        let prev_start = prev.as_ptr() as usize - original_input.as_ptr() as usize;
                        let prev_end = prev_start + prev.len();
                        if prev_end == start_offset {
                            // Adjacent, so extend
                            *prev = &original_input[prev_start..start_offset + char_len];
                            continue;
                        }
                    }
                    tokens.push(Token::Text(char_str));
                }
            }
        }
    }

    // Merge adjacent text tokens
    merge_text_tokens(tokens, original_input)
}

/// Merges adjacent text tokens into single tokens.
fn merge_text_tokens<'a>(tokens: Vec<Token<'a>>, input: &'a str) -> Vec<Token<'a>> {
    if tokens.is_empty() {
        return tokens;
    }

    let mut result = Vec::with_capacity(tokens.len());
    let input_start = input.as_ptr() as usize;

    for token in tokens {
        match (&token, result.last_mut()) {
            (Token::Text(new_text), Some(Token::Text(ref mut existing))) => {
                // Check if adjacent
                let existing_start = existing.as_ptr() as usize - input_start;
                let existing_end = existing_start + existing.len();
                let new_start = new_text.as_ptr() as usize - input_start;

                if existing_end == new_start {
                    // Merge them
                    *existing = &input[existing_start..new_start + new_text.len()];
                } else {
                    result.push(token);
                }
            }
            _ => result.push(token),
        }
    }

    result
}

/// Parses a single token from the input.
fn parse_token<'a>(
    input: &mut &'a str,
    original: &'a str,
    offset: usize,
) -> PResult<Token<'a>> {
    let start = *input;
    
    alt((
        parse_close_tag,
        parse_open_tag,
        parse_url,
        parse_linebreak,
        parse_text,
    ))
    .parse_next(input)
    .map(|mut token| {
        // Update raw references to point to original input
        let consumed = start.len() - input.len();
        let raw_slice = &original[offset..offset + consumed];
        
        match &mut token {
            Token::OpenTag { ref mut raw, .. } => *raw = raw_slice,
            Token::CloseTag { ref mut raw, .. } => *raw = raw_slice,
            Token::Text(ref mut s) => *s = raw_slice,
            Token::LineBreak(ref mut s) => *s = raw_slice,
            Token::Url(ref mut s) => *s = raw_slice,
        }
        
        token
    })
}

/// Parses an opening tag like `[tag]` or `[tag=value]`.
fn parse_open_tag<'a>(input: &mut &'a str) -> PResult<Token<'a>> {
    // Match opening bracket
    if !input.starts_with('[') {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }
    
    // Check this is not a closing tag
    if input.get(1..2) == Some("/") {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }

    *input = &input[1..]; // consume '['

    // Parse tag name (alphanumeric, *, -)
    let name_end = input.find(|c: char| !c.is_alphanumeric() && c != '*' && c != '-').unwrap_or(input.len());
    if name_end == 0 {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }
    let name = &input[..name_end];
    *input = &input[name_end..];

    // Parse optional argument
    let arg = if input.starts_with('=') {
        *input = &input[1..]; // consume '='
        
        // Check for quoted value
        if input.starts_with('"') {
            let quoted: &str = delimited('"', take_till(0.., |c: char| c == '"'), '"')
                .parse_next(input)
                .map_err(|_: ErrMode<ContextError>| ErrMode::Backtrack(ContextError::new()))?;
            Some(quoted)
        } else if input.starts_with('\'') {
            let quoted: &str = delimited('\'', take_till(0.., |c: char| c == '\''), '\'')
                .parse_next(input)
                .map_err(|_: ErrMode<ContextError>| ErrMode::Backtrack(ContextError::new()))?;
            Some(quoted)
        } else {
            // Unquoted value - take until ]
            let value_end = input.find(']').unwrap_or(input.len());
            if value_end == 0 {
                return Err(ErrMode::Backtrack(ContextError::new()));
            }
            let value = &input[..value_end];
            *input = &input[value_end..];
            Some(value)
        }
    } else {
        None
    };

    // Match closing bracket
    if !input.starts_with(']') {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }
    *input = &input[1..]; // consume ']'

    Ok(Token::OpenTag {
        raw: "", // Will be updated by parse_token
        name,
        arg,
    })
}

/// Parses a closing tag like `[/tag]`.
fn parse_close_tag<'a>(input: &mut &'a str) -> PResult<Token<'a>> {
    // Match [/
    if !input.starts_with("[/") {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }
    
    *input = &input[2..]; // consume "[/"

    // Parse tag name
    let name_end = input.find(|c: char| !c.is_alphanumeric() && c != '*' && c != '-').unwrap_or(input.len());
    if name_end == 0 {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }
    let name = &input[..name_end];
    *input = &input[name_end..];

    // Match closing bracket
    if !input.starts_with(']') {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }
    *input = &input[1..]; // consume ']'

    Ok(Token::CloseTag {
        raw: "", // Will be updated by parse_token
        name,
    })
}

/// Parses a URL (http:// or https://).
fn parse_url<'a>(input: &mut &'a str) -> PResult<Token<'a>> {
    // Check for http:// or https://
    if !input.starts_with("http://") && !input.starts_with("https://") {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }

    // Take the protocol
    let protocol_len = if input.starts_with("https://") { 8 } else { 7 };
    
    // Take characters that are valid in URLs
    let rest = &input[protocol_len..];
    let url_end = rest.find(|c: char| {
        c.is_whitespace()
            || matches!(c, '[' | ']' | '<' | '>')
    }).unwrap_or(rest.len());

    let total_len = protocol_len + url_end;
    let mut url = &input[..total_len];
    *input = &input[total_len..];

    // Trim trailing punctuation that's likely not part of the URL
    url = url.trim_end_matches(|c: char| matches!(c, '.' | ',' | ')' | '!' | '?' | ':' | ';'));
    
    Ok(Token::Url(url))
}

/// Parses a line break.
fn parse_linebreak<'a>(input: &mut &'a str) -> PResult<Token<'a>> {
    if input.starts_with("\r\n") {
        *input = &input[2..];
        Ok(Token::LineBreak("\r\n"))
    } else if input.starts_with('\n') {
        *input = &input[1..];
        Ok(Token::LineBreak("\n"))
    } else if input.starts_with('\r') {
        *input = &input[1..];
        Ok(Token::LineBreak("\r"))
    } else {
        Err(ErrMode::Backtrack(ContextError::new()))
    }
}

/// Parses plain text until a special character.
fn parse_text<'a>(input: &mut &'a str) -> PResult<Token<'a>> {
    let end = input.find(|c: char| c == '[' || c == '\n' || c == '\r' || c == 'h').unwrap_or(input.len());
    
    if end == 0 {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }
    
    let text = &input[..end];
    *input = &input[end..];

    Ok(Token::Text(text))
}

/// Tokenizes verbatim content until we find a close tag.
/// Used for [code], [plain], etc. where BBCode inside should not be parsed.
/// Returns (content before close tag, close tag itself, remaining after close tag)
pub fn tokenize_until_close<'a>(input: &'a str, tag_name: &str) -> (&'a str, &'a str, &'a str) {
    let close_pattern_lower = format!("[/{}]", tag_name.to_lowercase());
    
    // Search for the close tag (case-insensitive)
    let input_lower = input.to_lowercase();
    
    if let Some(pos) = input_lower.find(&close_pattern_lower) {
        let content = &input[..pos];
        let close_tag_len = close_pattern_lower.len();
        let close_tag = &input[pos..pos + close_tag_len];
        let remaining = &input[pos + close_tag_len..];
        (content, close_tag, remaining)
    } else {
        // No close tag found, return everything as content
        (input, "", "")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Basic tokenization tests
    #[test]
    fn tokenize_plain_text() {
        let tokens = tokenize("Hello World");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Text("Hello World")));
    }

    #[test]
    fn tokenize_bold_tag() {
        let tokens = tokenize("[b]Bold[/b]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::OpenTag { name: "b", .. }));
        assert!(matches!(tokens[1], Token::Text("Bold")));
        assert!(matches!(tokens[2], Token::CloseTag { name: "b", .. }));
    }

    #[test]
    fn tokenize_tag_with_arg() {
        let tokens = tokenize("[color=red]Text[/color]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(
            tokens[0],
            Token::OpenTag {
                name: "color",
                arg: Some("red"),
                ..
            }
        ));
    }

    #[test]
    fn tokenize_tag_with_quoted_arg() {
        let tokens = tokenize("[url=\"https://example.com\"]Link[/url]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(
            tokens[0],
            Token::OpenTag {
                name: "url",
                arg: Some("https://example.com"),
                ..
            }
        ));
    }

    #[test]
    fn tokenize_tag_with_single_quoted_arg() {
        let tokens = tokenize("[font='Arial']Text[/font]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(
            tokens[0],
            Token::OpenTag {
                name: "font",
                arg: Some("Arial"),
                ..
            }
        ));
    }

    #[test]
    fn tokenize_self_closing_style() {
        let tokens = tokenize("[*]Item");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::OpenTag { name: "*", .. }));
        assert!(matches!(tokens[1], Token::Text("Item")));
    }

    #[test]
    fn tokenize_linebreaks() {
        let tokens = tokenize("Line1\nLine2\r\nLine3");
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::Text("Line1")));
        assert!(matches!(tokens[1], Token::LineBreak("\n")));
        assert!(matches!(tokens[2], Token::Text("Line2")));
        assert!(matches!(tokens[3], Token::LineBreak("\r\n")));
        assert!(matches!(tokens[4], Token::Text("Line3")));
    }

    #[test]
    fn tokenize_url() {
        let tokens = tokenize("Visit https://example.com today!");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Text("Visit ")));
        assert!(matches!(tokens[1], Token::Url("https://example.com")));
        assert!(matches!(tokens[2], Token::Text(" today!")));
    }

    #[test]
    fn tokenize_http_url() {
        let tokens = tokenize("Go to http://example.com");
        assert!(tokens.iter().any(|t| matches!(t, Token::Url(u) if u.starts_with("http://"))));
    }

    #[test]
    fn tokenize_complex() {
        let input = "[quote=\"User\"]Hello [b]World[/b]![/quote]";
        let tokens = tokenize(input);
        
        assert!(tokens.len() >= 5);
        assert!(matches!(
            tokens[0],
            Token::OpenTag {
                name: "quote",
                arg: Some("User"),
                ..
            }
        ));
    }

    #[test]
    fn tokenize_nested() {
        let tokens = tokenize("[b][i]Nested[/i][/b]");
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::OpenTag { name: "b", .. }));
        assert!(matches!(tokens[1], Token::OpenTag { name: "i", .. }));
        assert!(matches!(tokens[2], Token::Text("Nested")));
        assert!(matches!(tokens[3], Token::CloseTag { name: "i", .. }));
        assert!(matches!(tokens[4], Token::CloseTag { name: "b", .. }));
    }

    #[test]
    fn tokenize_invalid_tag() {
        let tokens = tokenize("[invalid");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Text("[invalid")));
    }

    #[test]
    fn tokenize_empty_brackets() {
        let tokens = tokenize("[]");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Text("[]")));
    }

    #[test]
    fn tokenize_bracket_only() {
        let tokens = tokenize("[");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Text("[")));
    }

    #[test]
    fn tokenize_unbalanced_brackets() {
        let tokens = tokenize("[b");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Text("[b")));
    }

    #[test]
    fn tokenize_empty_input() {
        let tokens = tokenize("");
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize_preserves_whitespace() {
        let tokens = tokenize("  spaces  ");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Text("  spaces  ")));
    }

    #[test]
    fn tokenize_case_sensitivity() {
        let tokens = tokenize("[B]Text[/B]");
        assert!(matches!(tokens[0], Token::OpenTag { name: "B", .. }));
        assert!(matches!(tokens[2], Token::CloseTag { name: "B", .. }));
    }

    #[test]
    fn tokenize_url_in_text() {
        let tokens = tokenize("Check out https://rust-lang.org for more info");
        assert!(tokens.iter().any(|t| matches!(t, Token::Url(_))));
    }

    #[test]
    fn tokenize_url_with_path() {
        let tokens = tokenize("See https://example.com/path/to/page?q=test#anchor end");
        let url_token = tokens.iter().find(|t| matches!(t, Token::Url(_))).unwrap();
        if let Token::Url(url) = url_token {
            assert!(url.contains("/path/to/page"));
            assert!(url.contains("?q=test"));
        }
    }

    #[test]
    fn tokenize_until_close_basic() {
        let (content, close_tag, remaining) = tokenize_until_close("some [b]code[/b] here[/code]rest", "code");
        assert_eq!(content, "some [b]code[/b] here");
        assert_eq!(close_tag, "[/code]");
        assert_eq!(remaining, "rest");
    }

    #[test]
    fn tokenize_until_close_case_insensitive() {
        let (content, close_tag, remaining) = tokenize_until_close("content[/CODE]rest", "code");
        assert_eq!(content, "content");
        assert_eq!(close_tag, "[/CODE]");
        assert_eq!(remaining, "rest");
    }

    #[test]
    fn tokenize_until_close_not_found() {
        let (content, close_tag, remaining) = tokenize_until_close("no close tag here", "code");
        assert_eq!(content, "no close tag here");
        assert_eq!(close_tag, "");
        assert_eq!(remaining, "");
    }

    #[test]
    fn tokenize_multiple_same_tags() {
        let tokens = tokenize("[b]one[/b] [b]two[/b]");
        let open_count = tokens.iter().filter(|t| matches!(t, Token::OpenTag { name: "b", .. })).count();
        let close_count = tokens.iter().filter(|t| matches!(t, Token::CloseTag { name: "b", .. })).count();
        assert_eq!(open_count, 2);
        assert_eq!(close_count, 2);
    }

    #[test]
    fn tokenize_special_tag_names() {
        let tokens = tokenize("[list-item]Test[/list-item]");
        assert!(matches!(tokens[0], Token::OpenTag { name: "list-item", .. }));
    }

    #[test]
    fn raw_text_preservation() {
        let input = "[b]Bold[/b]";
        let tokens = tokenize(input);
        
        if let Token::OpenTag { raw, .. } = &tokens[0] {
            assert_eq!(*raw, "[b]");
        }
        if let Token::CloseTag { raw, .. } = &tokens[2] {
            assert_eq!(*raw, "[/b]");
        }
    }

    #[test]
    fn tokenize_unicode() {
        let tokens = tokenize("[b]日本語[/b]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[1], Token::Text("日本語")));
    }

    #[test]
    fn tokenize_emoji() {
        let tokens = tokenize("[b]🔥🎉[/b]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[1], Token::Text("🔥🎉")));
    }

    #[test]
    fn tokenize_mixed_content() {
        let tokens = tokenize("Text [b]bold[/b] more https://example.com end");
        assert!(tokens.len() >= 5);
        assert!(tokens.iter().any(|t| matches!(t, Token::Text(_))));
        assert!(tokens.iter().any(|t| matches!(t, Token::OpenTag { .. })));
        assert!(tokens.iter().any(|t| matches!(t, Token::Url(_))));
    }
}
