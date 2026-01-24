#![no_main]

use libfuzzer_sys::fuzz_target;

/// Fuzz image tag handling - looking for XSS via src/onerror
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let input = format!("[img]{}[/img]", s);
        let result = bbcode::parse(&input);

        let lower = result.to_lowercase();

        // No dangerous protocols in src
        assert!(!lower.contains("src=\"javascript:"), "JavaScript in img src!");
        assert!(!lower.contains("src=\"data:"), "Data URL in img src!");
        assert!(!lower.contains("src=\"vbscript:"), "VBScript in img src!");

        // No event handlers as attributes
        if result.contains("<img") {
            assert!(!lower.contains(" onerror="), "onerror handler leaked!");
            assert!(!lower.contains(" onload="), "onload handler leaked!");
            assert!(!lower.contains(" onmouseover="), "onmouseover handler leaked!");
            assert!(!lower.contains(" onclick="), "onclick handler leaked!");
        }
    }
});
