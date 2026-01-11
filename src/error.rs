//! Error types for the BBCode parser.
//!
//! This module defines all error types using `thiserror` for zero-overhead,
//! typed errors.

use std::borrow::Cow;
use thiserror::Error;

/// Errors that can occur during BBCode parsing.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ParseError {
    /// The input ended unexpectedly while parsing.
    #[error("unexpected end of input at position {position}")]
    UnexpectedEof { position: usize },

    /// An invalid tag name was encountered.
    #[error("invalid tag name: {name}")]
    InvalidTagName { name: String },

    /// An unclosed tag was found.
    #[error("unclosed tag: [{tag}]")]
    UnclosedTag { tag: String },

    /// A closing tag was found without a matching opening tag.
    #[error("unmatched closing tag: [/{tag}]")]
    UnmatchedClosingTag { tag: String },

    /// An invalid attribute value was provided.
    #[error("invalid attribute value for [{tag}]: {message}")]
    InvalidAttribute { tag: String, message: String },

    /// An invalid URL was provided.
    #[error("invalid URL: {url}")]
    InvalidUrl { url: String },

    /// An invalid color value was provided.
    #[error("invalid color: {color}")]
    InvalidColor { color: String },

    /// An invalid size value was provided.
    #[error("invalid size: {size}")]
    InvalidSize { size: String },

    /// Nesting depth exceeded.
    #[error("maximum nesting depth ({max_depth}) exceeded")]
    NestingTooDeep { max_depth: usize },

    /// A tag is not allowed in this context.
    #[error("tag [{child}] is not allowed inside [{parent}]")]
    InvalidNesting { parent: String, child: String },

    /// Generic parse error.
    #[error("parse error: {message}")]
    Generic { message: Cow<'static, str> },
}

/// Errors that can occur during HTML rendering.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum RenderError {
    /// An IO error occurred during rendering.
    #[error("render error: {message}")]
    Generic { message: Cow<'static, str> },
}

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// Result type for rendering operations.
pub type RenderResult<T> = Result<T, RenderError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = ParseError::UnexpectedEof { position: 42 };
        assert_eq!(err.to_string(), "unexpected end of input at position 42");

        let err = ParseError::InvalidTagName {
            name: "foo".to_string(),
        };
        assert_eq!(err.to_string(), "invalid tag name: foo");

        let err = ParseError::UnclosedTag {
            tag: "b".to_string(),
        };
        assert_eq!(err.to_string(), "unclosed tag: [b]");

        let err = ParseError::UnmatchedClosingTag {
            tag: "i".to_string(),
        };
        assert_eq!(err.to_string(), "unmatched closing tag: [/i]");

        let err = ParseError::InvalidAttribute {
            tag: "color".to_string(),
            message: "must be a valid CSS color".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid attribute value for [color]: must be a valid CSS color"
        );

        let err = ParseError::InvalidUrl {
            url: "not-a-url".to_string(),
        };
        assert_eq!(err.to_string(), "invalid URL: not-a-url");

        let err = ParseError::InvalidColor {
            color: "notacolor".to_string(),
        };
        assert_eq!(err.to_string(), "invalid color: notacolor");

        let err = ParseError::InvalidSize {
            size: "giant".to_string(),
        };
        assert_eq!(err.to_string(), "invalid size: giant");

        let err = ParseError::NestingTooDeep { max_depth: 20 };
        assert_eq!(err.to_string(), "maximum nesting depth (20) exceeded");

        let err = ParseError::InvalidNesting {
            parent: "url".to_string(),
            child: "url".to_string(),
        };
        assert_eq!(err.to_string(), "tag [url] is not allowed inside [url]");

        let err = ParseError::Generic {
            message: Cow::Borrowed("something went wrong"),
        };
        assert_eq!(err.to_string(), "parse error: something went wrong");
    }

    #[test]
    fn render_error_display() {
        let err = RenderError::Generic {
            message: Cow::Borrowed("failed to render"),
        };
        assert_eq!(err.to_string(), "render error: failed to render");
    }

    #[test]
    fn error_equality() {
        let err1 = ParseError::InvalidTagName {
            name: "foo".to_string(),
        };
        let err2 = ParseError::InvalidTagName {
            name: "foo".to_string(),
        };
        let err3 = ParseError::InvalidTagName {
            name: "bar".to_string(),
        };

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn error_clone() {
        let err = ParseError::UnclosedTag {
            tag: "quote".to_string(),
        };
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }
}
