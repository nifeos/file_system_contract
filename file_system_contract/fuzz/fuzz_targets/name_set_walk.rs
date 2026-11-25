//! Walk an arbitrary name set, then encode the names the walk read and walk the result.
//!
//! The walk reads at most `MAXIMUM_NAMES` records. The encoder takes them back exactly when every
//! name is one a grant can carry and the names together fit `MAXIMUM_NAME_SET_BYTES`; otherwise it
//! refuses the whole set, never a prefix of it.

#![no_main]

use file_system_contract::{grant, nameset};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|set: &[u8]| {
    let names: Vec<(&[u8], bool)> = nameset::iter(set).collect();
    assert!(names.len() <= nameset::MAXIMUM_NAMES);
    assert_eq!(nameset::count(set), names.len());
    for (name, _) in &names {
        assert!(nameset::contains(set, name));
    }

    let mut out = [0u8; nameset::MAXIMUM_BYTES];
    let encoded = nameset::encode(&names, &mut out);
    let name_bytes: usize = names.iter().map(|(name, _)| name.len()).sum();
    let acceptable = names.iter().all(|(name, _)| grant::fits(name))
        && name_bytes <= nameset::MAXIMUM_NAME_SET_BYTES;
    match encoded {
        Some(used) => {
            assert!(acceptable);
            let again: Vec<(&[u8], bool)> = nameset::iter(&out[..used]).collect();
            assert_eq!(again, names);
        }
        None => assert!(!acceptable),
    }
});
