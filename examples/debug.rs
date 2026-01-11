use bbcode::parse;

fn main() {
    let input = r#"[code=rust]
fn main() {
    println!("Hello, world!");
}
[/code]

You can also use [icode]println![/icode] macro.

[b]Note:[/b] Dont forget!"#;

    let result = parse(input);
    println!("Result:
{}", result);
}
