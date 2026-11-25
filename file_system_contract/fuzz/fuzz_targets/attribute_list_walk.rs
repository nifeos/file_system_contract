//! Walk an arbitrary `LISTXATTR` reply, then encode every name the walk read and walk the result.
//!
//! The walk must stay inside the reply. The reader accepts a name holding a zero byte, which the
//! encoder refuses because the store never holds one, so the round trip is checked over the names
//! the store could hold.

#![no_main]

use file_system_contract::extended_attribute::{is_valid_name, list};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|reply: &[u8]| {
    let names: Vec<&[u8]> = list::iter(reply).collect();
    let read: usize = names.iter().map(|name| list::record_len(name.len())).sum();
    assert!(read <= reply.len());

    let mut out = vec![0u8; read];
    let mut used = 0;
    for name in &names {
        match list::encode(&mut out[used..], name) {
            Some(n) => {
                assert!(is_valid_name(name));
                used += n;
            }
            None => assert!(!is_valid_name(name)),
        }
    }

    let valid: Vec<&[u8]> = names
        .iter()
        .copied()
        .filter(|name| is_valid_name(name))
        .collect();
    let again: Vec<&[u8]> = list::iter(&out[..used]).collect();
    assert_eq!(again, valid);
});
