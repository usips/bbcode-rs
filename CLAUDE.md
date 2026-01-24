# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A zero-copy BBCode parser for Rust supporting phpBB and XenForo syntax. Converts BBCode to safe, XSS-protected HTML.

## Commands

```bash
# Build
cargo build
cargo build --release

# Run tests
cargo test
cargo test --release

# Run a single test
cargo test test_name

# Run benchmarks
cargo bench

# Run examples
cargo run --example attach
cargo run --example debug
cargo run --example xss_check

# Enable plugins feature (linkme-based distributed registration)
cargo build --features plugins
```

## Architecture

The library follows a three-stage pipeline: **Tokenize → Parse → Render**

### Pipeline Flow

```
Input BBCode → Tokenizer → Tokens → Parser → AST (Document) → Renderer → HTML
```

### Key Modules

- **`tokenizer.rs`**: Zero-copy tokenizer using `winnow` parser combinators. Produces `Token` enum (Text, LineBreak, Url, OpenTag, CloseTag). All string data references the original input.

- **`parser.rs`**: Converts tokens into AST (`Document` containing `Node`s). Handles tag nesting validation, forbidden ancestors, required parents, verbatim content, and max depth limits. Uses `TagRegistry` to resolve tag definitions.

- **`ast.rs`**: Core data structures - `Document`, `Node` (Text/LineBreak/AutoUrl/Tag), `TagNode`, `TagOption` (None/Scalar/Map), `TagType` (Inline/Block/Verbatim/SelfClosing/Void).

- **`renderer.rs`**: Converts AST to HTML with XSS protection. Supports custom tag handlers via `CustomTagHandler` trait. Validates colors, fonts, sizes, and URLs.

- **`tags.rs`**: Tag definitions - `TagDef` (static, compile-time), `CustomTagDef` (runtime, owned strings), `TagRegistry` for lookup. Contains `STANDARD_TAGS` array with all built-in tags.

### Extensibility

Custom tags are added in two places:
1. **Parser**: Register `CustomTagDef` with `parser.register_custom_tag()`
2. **Renderer**: Register `CustomTagHandler` with `renderer.register_handler()`

See `examples/attach.rs` for a complete custom tag implementation with batch data fetching.

### Key Design Decisions

- **Zero-copy parsing**: Tokens and AST nodes use `&str` or `Cow<str>` referencing original input
- **Tag case insensitivity**: Tag names normalized to lowercase internally, raw case preserved
- **Graceful degradation**: Unknown/broken tags render as escaped text
- **XSS protection**: URL scheme validation, HTML escaping, event handler blocking
- **Verbatim tags**: `[code]`, `[plain]` content not parsed for BBCode

### Tag Types

- `Inline`: `[b]`, `[i]`, `[color]` - nest freely
- `Block`: `[quote]`, `[list]`, `[table]` - structural elements
- `Verbatim`: `[code]`, `[plain]`, `[icode]` - content not parsed
- `SelfClosing`: `[hr]`, `[br]`, `[*]` - no closing tag needed
- `Void`: `[img]` - renders as void HTML element

## Feature Parity Goals

Primary goal: XenForo compatibility. Secondary: phpBB compatibility.

### Current Tag Support

**Implemented (simple rendering):**
`[b]`, `[i]`, `[u]`, `[s]`, `[color]`, `[font]`, `[size]`, `[sub]`, `[sup]`, `[url]`, `[email]`, `[img]`, `[quote]`, `[code]`, `[icode]`, `[php]`, `[html]`, `[plain]`, `[list]`, `[*]`, `[left]`, `[center]`, `[right]`, `[justify]`, `[indent]`, `[heading]`, `[hr]`, `[br]`, `[spoiler]`, `[ispoiler]`, `[user]`, `[table]`, `[tr]`, `[th]`, `[td]`

### Missing XenForo Tags (Priority)

| Tag | Type | Requires Prefetch | Notes |
|-----|------|-------------------|-------|
| `[attach]` | Complex | Yes - attachment entities | See `examples/attach.rs` for pattern |
| `[media]` | Complex | Yes - media site configs | YouTube, Vimeo, etc. via oEmbed or callbacks |
| `[url unfurl="true"]` | Complex | Yes - UnfurlResult data | Rich URL previews with title/description/image |
| `[embed]` | Complex | Yes - entity loading | Embeds other posts/content with permission checks |

### Missing phpBB Tags

| Tag | Type | Notes |
|-----|------|-------|
| `[flash]` | Simple | Deprecated, low priority |
| `[attachment]` | Complex | Same as XenForo `[attach]` |

### Prefetch Pattern for Complex Tags

XenForo uses `getBbCodeRenderOptions()` on entities to provide prefetched data. Our equivalent:

```rust
// 1. First pass: collect IDs from AST
let attachment_ids = collect_attachment_ids(&document);

// 2. Batch fetch from database
let attachments = fetch_attachments(attachment_ids).await;

// 3. Render with context
let mut renderer = Renderer::new();
renderer.set_context("attachments", attachments);
let html = renderer.render(&document);
```

### XenForo-Specific Behaviors to Implement

1. **Quote metadata injection**: XenForo's `[quote]` supports `post_id`, `user_id`, `time` attributes that generate URLs at render time
2. **Image proxy**: `[img]` can route through image proxy service for dimension fetching
3. **Media site callbacks**: `[media]` has per-site helper classes (YouTube.php, Vimeo.php)
4. **Cookie consent**: Third-party embeds can require consent before loading

### phpBB-Specific Behaviors (Lower Priority)

1. **UID suffix system**: phpBB stores `[b:uid]text[/b:uid]` - not needed for our use case
2. **Bitfield optimization**: phpBB tracks which BBCodes are used per post - optimization only
3. **PHP syntax highlighting**: `[code=php]` uses PHP's `highlight_string()` - use syntect/tree-sitter instead
