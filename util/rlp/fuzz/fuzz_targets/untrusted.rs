#[cfg(feature = "nightly")
#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate rlp;

use rlp::UntrustedRlp;

fuzz_target!(|data: &[u8]| {
    let _ = UntrustedRlp::new(data);
});
]
