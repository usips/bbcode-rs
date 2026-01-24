#![no_main]

use libfuzzer_sys::fuzz_target;

/// Fuzz style-related tags - looking for CSS injection
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test color tag
        let color_input = format!("[color={}]text[/color]", s);
        let color_result = bbcode::parse(&color_input);
        check_no_css_injection(&color_result);

        // Test size tag
        let size_input = format!("[size={}]text[/size]", s);
        let size_result = bbcode::parse(&size_input);
        check_no_css_injection(&size_result);

        // Test font tag
        let font_input = format!("[font={}]text[/font]", s);
        let font_result = bbcode::parse(&font_input);
        check_no_css_injection(&font_result);
    }
});

fn check_no_css_injection(result: &str) {
    let lower = result.to_lowercase();

    // No CSS expression (IE)
    assert!(!lower.contains("expression("), "CSS expression leaked!");

    // No JavaScript in URLs
    assert!(!lower.contains("url(javascript:"), "JavaScript in CSS url()!");

    // No behavior (IE)
    assert!(!lower.contains("behavior:"), "CSS behavior leaked!");

    // No event handlers
    assert!(!lower.contains(" onclick="), "onclick leaked!");
    assert!(!lower.contains(" onerror="), "onerror leaked!");
    assert!(!lower.contains(" onmouseover="), "onmouseover leaked!");
}
