//! Benchmarks for the BBCode parser using Criterion.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use bbcode::{parse, Parser, Renderer};

// ============================================================================
// Sample BBCode Content
// ============================================================================

const SIMPLE_TEXT: &str = "Hello, World!";

const BASIC_FORMATTING: &str = "[b]Bold[/b] and [i]italic[/i] and [u]underline[/u]";

const NESTED_FORMATTING: &str = "[b][i][u]Triple nested formatting[/u][/i][/b]";

const URL_TAG: &str = "[url=https://example.com]Example Website[/url]";

const QUOTE_BLOCK: &str = r#"[quote="PreviousUser"]This is a quoted message from someone else[/quote]"#;

const CODE_BLOCK: &str = r#"[code=rust]
fn main() {
    println!("Hello, world!");
    let x = 42;
    for i in 0..x {
        println!("{}", i);
    }
}
[/code]"#;

const LIST_BLOCK: &str = r#"[list=1]
[*]First item
[*]Second item
[*]Third item
[*]Fourth item
[*]Fifth item
[/list]"#;

const COMPLEX_POST: &str = r#"[quote="Admin"]Please follow the rules[/quote]

I have some thoughts on this:

[b]Main Points:[/b]
[list]
[*][i]First point[/i] - This is important
[*][i]Second point[/i] - Also important
[*][i]Third point[/i] - Very important
[/list]

Here's some code:
[code=python]
def hello():
    print("Hello, World!")
[/code]

Check out [url=https://example.com]this link[/url] for more info.

[center][color=gray][size=2]
Thanks for reading!
[/size][/color][/center]"#;

const TABLE_BLOCK: &str = r#"[table]
[tr][th]Name[/th][th]Value[/th][th]Description[/th][/tr]
[tr][td]Item 1[/td][td]100[/td][td]First item[/td][/tr]
[tr][td]Item 2[/td][td]200[/td][td]Second item[/td][/tr]
[tr][td]Item 3[/td][td]300[/td][td]Third item[/td][/tr]
[tr][td]Item 4[/td][td]400[/td][td]Fourth item[/td][/tr]
[/table]"#;

// ============================================================================
// Parsing Benchmarks
// ============================================================================

fn bench_parse_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_simple");
    
    group.throughput(Throughput::Bytes(SIMPLE_TEXT.len() as u64));
    group.bench_function("plain_text", |b| {
        b.iter(|| parse(black_box(SIMPLE_TEXT)))
    });

    group.throughput(Throughput::Bytes(BASIC_FORMATTING.len() as u64));
    group.bench_function("basic_formatting", |b| {
        b.iter(|| parse(black_box(BASIC_FORMATTING)))
    });

    group.throughput(Throughput::Bytes(NESTED_FORMATTING.len() as u64));
    group.bench_function("nested_formatting", |b| {
        b.iter(|| parse(black_box(NESTED_FORMATTING)))
    });

    group.finish();
}

fn bench_parse_blocks(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_blocks");

    group.throughput(Throughput::Bytes(URL_TAG.len() as u64));
    group.bench_function("url", |b| {
        b.iter(|| parse(black_box(URL_TAG)))
    });

    group.throughput(Throughput::Bytes(QUOTE_BLOCK.len() as u64));
    group.bench_function("quote", |b| {
        b.iter(|| parse(black_box(QUOTE_BLOCK)))
    });

    group.throughput(Throughput::Bytes(CODE_BLOCK.len() as u64));
    group.bench_function("code", |b| {
        b.iter(|| parse(black_box(CODE_BLOCK)))
    });

    group.throughput(Throughput::Bytes(LIST_BLOCK.len() as u64));
    group.bench_function("list", |b| {
        b.iter(|| parse(black_box(LIST_BLOCK)))
    });

    group.throughput(Throughput::Bytes(TABLE_BLOCK.len() as u64));
    group.bench_function("table", |b| {
        b.iter(|| parse(black_box(TABLE_BLOCK)))
    });

    group.finish();
}

fn bench_parse_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_complex");

    group.throughput(Throughput::Bytes(COMPLEX_POST.len() as u64));
    group.bench_function("forum_post", |b| {
        b.iter(|| parse(black_box(COMPLEX_POST)))
    });

    group.finish();
}

// ============================================================================
// Scaling Benchmarks
// ============================================================================

fn bench_scaling_repetitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_repetitions");

    for count in [1, 10, 100, 1000].iter() {
        let input = BASIC_FORMATTING.repeat(*count);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("basic_formatting", count),
            &input,
            |b, input| b.iter(|| parse(black_box(input))),
        );
    }

    group.finish();
}

fn bench_scaling_nesting_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_nesting");

    for depth in [1, 5, 10, 20, 50].iter() {
        let mut input = String::new();
        for _ in 0..*depth {
            input.push_str("[b]");
        }
        input.push_str("deep");
        for _ in 0..*depth {
            input.push_str("[/b]");
        }

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("depth", depth),
            &input,
            |b, input| b.iter(|| parse(black_box(input))),
        );
    }

    group.finish();
}

fn bench_scaling_list_items(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_list");

    for item_count in [1, 10, 50, 100].iter() {
        let mut input = String::from("[list]");
        for i in 0..*item_count {
            input.push_str(&format!("[*]Item number {}", i));
        }
        input.push_str("[/list]");

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("items", item_count),
            &input,
            |b, input| b.iter(|| parse(black_box(input))),
        );
    }

    group.finish();
}

// ============================================================================
// Component Benchmarks
// ============================================================================

fn bench_tokenizer_only(c: &mut Criterion) {
    use bbcode::tokenizer::tokenize;

    let mut group = c.benchmark_group("tokenizer");

    group.throughput(Throughput::Bytes(COMPLEX_POST.len() as u64));
    group.bench_function("complex_post", |b| {
        b.iter(|| tokenize(black_box(COMPLEX_POST)))
    });

    group.finish();
}

fn bench_parser_only(c: &mut Criterion) {
    let parser = Parser::new();

    let mut group = c.benchmark_group("parser");

    group.throughput(Throughput::Bytes(COMPLEX_POST.len() as u64));
    group.bench_function("complex_post", |b| {
        b.iter(|| parser.parse(black_box(COMPLEX_POST)))
    });

    group.finish();
}

fn bench_renderer_only(c: &mut Criterion) {
    let parser = Parser::new();
    let renderer = Renderer::new();
    let doc = parser.parse(COMPLEX_POST);

    let mut group = c.benchmark_group("renderer");

    group.bench_function("complex_post", |b| {
        b.iter(|| renderer.render(black_box(&doc)))
    });

    group.finish();
}

// ============================================================================
// Real-world Simulation
// ============================================================================

fn bench_realistic_workload(c: &mut Criterion) {
    // Simulate parsing multiple forum posts
    let posts: Vec<&str> = vec![
        SIMPLE_TEXT,
        BASIC_FORMATTING,
        QUOTE_BLOCK,
        CODE_BLOCK,
        LIST_BLOCK,
        COMPLEX_POST,
        TABLE_BLOCK,
    ];

    let total_bytes: usize = posts.iter().map(|p| p.len()).sum();

    let mut group = c.benchmark_group("realistic");
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("parse_page", |b| {
        b.iter(|| {
            for post in &posts {
                black_box(parse(post));
            }
        })
    });

    // Simulate a thread with many posts of the same type
    let thread: String = COMPLEX_POST.repeat(20);
    group.throughput(Throughput::Bytes(thread.len() as u64));

    group.bench_function("parse_thread", |b| {
        b.iter(|| parse(black_box(&thread)))
    });

    group.finish();
}

// ============================================================================
// Memory Efficiency
// ============================================================================

fn bench_zero_copy_benefit(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_copy");

    // Text that doesn't need escaping - should be zero-copy
    let clean_text = "This is plain text without any special characters that need escaping.";
    
    // Text that needs escaping
    let dirty_text = "This has <html> & \"quotes\" that need escaping.";

    group.bench_function("clean_text", |b| {
        b.iter(|| parse(black_box(clean_text)))
    });

    group.bench_function("dirty_text", |b| {
        b.iter(|| parse(black_box(dirty_text)))
    });

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    simple,
    bench_parse_simple,
);

criterion_group!(
    blocks,
    bench_parse_blocks,
);

criterion_group!(
    complex,
    bench_parse_complex,
);

criterion_group!(
    scaling,
    bench_scaling_repetitions,
    bench_scaling_nesting_depth,
    bench_scaling_list_items,
);

criterion_group!(
    components,
    bench_tokenizer_only,
    bench_parser_only,
    bench_renderer_only,
);

criterion_group!(
    realistic,
    bench_realistic_workload,
    bench_zero_copy_benefit,
);

criterion_main!(simple, blocks, complex, scaling, components, realistic);
