//! Walk an arbitrary attribute blob, list it, and rebuild it with `store::set`.
//!
//! The walk never yields a kind past `MAXIMUM_KIND`, `get` finds the first record of a name, and
//! the listing holds the walk's names in order up to the first one it cannot carry. When the blob
//! is one the store could have written (distinct valid names, values within bounds, at most
//! `MAXIMUM_ATTRIBUTES` records), setting each record in turn on an empty blob gives back the
//! same walk.

#![no_main]

use file_system_contract::extended_attribute::{
    MAXIMUM_ATTRIBUTES, MAXIMUM_KIND, MAXIMUM_VALUE_BYTES, is_valid_name, list, store,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|blob: &[u8]| {
    let records: Vec<(&[u8], u32, &[u8])> = store::iter(blob).collect();
    assert_eq!(store::count(blob), records.len());
    for (index, &(name, kind, _)) in records.iter().enumerate() {
        assert!(kind <= MAXIMUM_KIND);
        let first = records
            .iter()
            .position(|(other, ..)| *other == name)
            .unwrap();
        assert!(first <= index);
        let (_, first_kind, first_value) = records[first];
        assert_eq!(store::get(blob, name), Some((first_kind, first_value)));
    }

    let mut listing = [0u8; list::MAXIMUM_BYTES];
    let used = store::list(blob, &mut listing);
    let listed: Vec<&[u8]> = list::iter(&listing[..used]).collect();
    assert!(listed.len() <= records.len());
    for (listed_name, (name, ..)) in listed.iter().zip(&records) {
        assert_eq!(listed_name, name);
    }
    if let Some(&(next, ..)) = records.get(listed.len()) {
        assert!(!is_valid_name(next) || used + list::record_len(next.len()) > listing.len());
    }

    let distinct = records
        .iter()
        .enumerate()
        .all(|(index, (name, ..))| records[..index].iter().all(|(other, ..)| other != name));
    let writable = records.len() <= MAXIMUM_ATTRIBUTES
        && distinct
        && records
            .iter()
            .all(|(name, _, value)| is_valid_name(name) && value.len() <= MAXIMUM_VALUE_BYTES);
    if !writable {
        return;
    }
    let mut rebuilt = Vec::new();
    for &(name, kind, value) in &records {
        let mut out = vec![0u8; rebuilt.len() + store::record_len(name.len(), value.len())];
        let n = store::set(&rebuilt, name, kind, value, &mut out)
            .expect("a writable blob must rebuild");
        out.truncate(n);
        rebuilt = out;
    }
    let again: Vec<(&[u8], u32, &[u8])> = store::iter(&rebuilt).collect();
    assert_eq!(again, records);
});
