#![no_main]

use hexane_fuzz::run_public_api;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &[u8]| {
    run_public_api(input);
});
