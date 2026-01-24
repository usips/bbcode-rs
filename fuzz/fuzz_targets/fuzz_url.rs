#![no_main]

use libfuzzer_sys::fuzz_target;

/// Fuzz URL handling specifically - looking for XSS bypasses
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Wrap in URL tag to specifically fuzz URL handling
        let input = format!("[url={}]Click[/url]", s);
        let result = bbcode::parse(&input);

        // Check for dangerous output
        let lower = result.to_lowercase();
        assert!(!lower.contains("href=\"javascript:"), "JavaScript URL leaked!");
        assert!(!lower.contains("href=\"vbscript:"), "VBScript URL leaked!");
        assert!(!lower.contains("href=\"data:"), "Data URL leaked!");

        // Also test as URL content
        let input2 = format!("[url]{}[/url]", s);
        let result2 = bbcode::parse(&input2);
        let lower2 = result2.to_lowercase();
        assert!(!lower2.contains("href=\"javascript:"), "JavaScript URL leaked in content!");
        assert!(!lower2.contains("href=\"data:"), "Data URL leaked in content!");
    }
});
