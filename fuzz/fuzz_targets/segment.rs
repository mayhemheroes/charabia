#![no_main]
use libfuzzer_sys::fuzz_target;
use charabia::Segment;

fuzz_target!(|input: String| {
    let orig: &str = &input;
    let _ = orig.segment().collect::<Vec<_>>();
});