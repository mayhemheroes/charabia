#![no_main]
use libfuzzer_sys::fuzz_target;
use charabia::Tokenize;

fuzz_target!(|input: String| {
    let orig: &str = &input;
    let _ = orig.tokenize().collect::<Vec<_>>();
});