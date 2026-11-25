//! Walk an arbitrary `READDIR` reply, then encode every entry the walk read and walk the result.
//!
//! The walk must stay inside the reply, and what it read must come back whole from the encoder.
//! The one entry the encoder refuses is an empty name, which the reader still accepts, so the
//! round trip is checked over the entries with a name.

#![no_main]

use file_system_contract::directory_entry::{self, record_len};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|reply: &[u8]| {
    let entries: Vec<(&[u8], bool)> = directory_entry::iter(reply).collect();
    let read: usize = entries.iter().map(|(name, _)| record_len(name.len())).sum();
    assert!(read <= reply.len());

    let mut out = vec![0u8; read];
    let mut used = 0;
    for &(name, is_directory) in &entries {
        match directory_entry::encode(&mut out[used..], name, is_directory) {
            Some(n) => {
                assert_eq!(n, record_len(name.len()));
                used += n;
            }
            None => assert!(name.is_empty()),
        }
    }

    let named: Vec<(&[u8], bool)> = entries
        .iter()
        .copied()
        .filter(|(name, _)| !name.is_empty())
        .collect();
    let again: Vec<(&[u8], bool)> = directory_entry::iter(&out[..used]).collect();
    assert_eq!(again, named);
});
