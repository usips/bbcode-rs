#![no_main]

use libfuzzer_sys::fuzz_target;

/// Fuzz the main parse function with arbitrary input
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = bbcode::parse(s);
    }
});
