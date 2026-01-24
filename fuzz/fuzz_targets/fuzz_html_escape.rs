#![no_main]

use libfuzzer_sys::fuzz_target;

/// Fuzz to ensure HTML is always properly escaped
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let result = bbcode::parse(s);

        // No raw script tags
        assert!(!result.contains("<script"), "Script tag leaked!");
        assert!(!result.contains("<SCRIPT"), "Script tag leaked (uppercase)!");

        // No raw SVG (which can contain scripts)
        assert!(!result.contains("<svg"), "SVG tag leaked!");
        assert!(!result.contains("<SVG"), "SVG tag leaked (uppercase)!");

        // No raw iframe
        assert!(!result.contains("<iframe"), "Iframe tag leaked!");
        assert!(!result.contains("<IFRAME"), "Iframe tag leaked (uppercase)!");

        // No raw object/embed
        assert!(!result.contains("<object"), "Object tag leaked!");
        assert!(!result.contains("<embed"), "Embed tag leaked!");

        // No raw form elements with dangerous attributes
        if result.contains("<form") || result.contains("<input") || result.contains("<button") {
            panic!("Form element leaked: {}", result);
        }

        // Ensure < in input is escaped in output (unless part of our generated HTML)
        if s.contains('<') && !s.contains('[') {
            // If input has < but no BBCode, output should have &lt;
            // This is a heuristic - raw HTML should be escaped
            let input_lt_count = s.matches('<').count();
            let output_lt_count = result.matches('<').count();
            let output_escaped_count = result.matches("&lt;").count();

            // There should be some escaping if input had <
            // (This is a fuzzy check - we allow some < from our generated tags)
            if input_lt_count > 0 && output_lt_count > input_lt_count * 2 && output_escaped_count == 0 {
                panic!(
                    "Possible unescaped HTML: input had {} '<', output has {} '<' and {} '&lt;'",
                    input_lt_count, output_lt_count, output_escaped_count
                );
            }
        }
    }
});
