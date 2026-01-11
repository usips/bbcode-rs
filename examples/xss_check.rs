use bbcode::parse;

fn main() {
    let tests = vec![
        (r#"[url=" onclick="alert(1)"]Click Me[/url]"#, "Double quote breakout"),
        (r#"[img]http://site.com/x.jpg" onerror="alert(1)[/img]"#, "Img onerror"),
        (r#"[color=red" onmouseover="alert(1)]Text[/color]"#, "Color onmouseover"),
        (r#"[email]test@test.com" onclick="alert(1)[/email]"#, "Email onclick"),
        (r#"[size=10;width:expression(alert(1))]Text[/size]"#, "Size expression"),
    ];
    
    for (input, desc) in tests {
        let result = parse(input);
        println!("=== {} ===", desc);
        println!("Input: {}", input);
        println!("Output: {}", result);
        
        // More accurate checks - look for DANGEROUS attributes not just the text
        // Pattern: space followed by event handler with unescaped =
        // Safe: &quot;onclick=&quot; or [something onclick=something]
        // Dangerous: <a onclick="..." or src="..." onclick="...
        
        let is_dangerous_onclick = result.contains(" onclick=") 
            && !result.contains("&quot;onclick=") 
            && !result.contains("onclick=&quot;");
        let is_dangerous_onerror = result.contains(" onerror=")
            && !result.contains("&quot;onerror=")
            && !result.contains("onerror=&quot;");
        let is_dangerous_onmouseover = result.contains(" onmouseover=")
            && !result.contains("&quot;onmouseover=")
            && !result.contains("onmouseover=&quot;");
        let has_dangerous_expression = result.contains("style=") && result.contains("expression(");
        
        // Check if rendered as BBCode text (safe)
        let rendered_as_text = result.starts_with('[') && !result.starts_with("<");
        
        if rendered_as_text {
            println!("✅ SAFE: Rendered as BBCode text (tag rejected)");
        } else {
            if is_dangerous_onclick {
                println!("❌ VULNERABILITY: Dangerous onclick in HTML!");
            }
            if is_dangerous_onerror {
                println!("❌ VULNERABILITY: Dangerous onerror in HTML!");
            }
            if is_dangerous_onmouseover {
                println!("❌ VULNERABILITY: Dangerous onmouseover in HTML!");
            }
            if has_dangerous_expression {
                println!("❌ VULNERABILITY: CSS expression in style!");
            }
            if !is_dangerous_onclick && !is_dangerous_onerror && !is_dangerous_onmouseover && !has_dangerous_expression {
                println!("✅ SAFE: No dangerous patterns detected");
            }
        }
        
        println!();
    }
}
