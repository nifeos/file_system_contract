//! Apply one `store::set` or `store::remove` to an arbitrary blob, and check the blob it produces.
//!
//! The input is an operation byte (even sets, odd removes), a name length byte, a kind (four bytes,
//! little-endian), a value length (two bytes, little-endian), the name, the value, and then the
//! blob. A success must leave every other record as the walk read it, in order; a refusal must be
//! the one error the inputs call for.

#![no_main]

use file_system_contract::directory::EINVAL;
use file_system_contract::extended_attribute::{
    E2BIG, ENODATA, ENOSPC, ERANGE, MAXIMUM_ATTRIBUTES, MAXIMUM_KIND, MAXIMUM_VALUE_BYTES,
    is_valid_name, store,
};
use libfuzzer_sys::fuzz_target;

type Record<'a> = (&'a [u8], u32, &'a [u8]);

fuzz_target!(|data: &[u8]| {
    let Some((head, rest)) = data.split_first_chunk::<8>() else {
        return;
    };
    let is_set = head[0] % 2 == 0;
    let name_len = usize::from(head[1]);
    let kind = u32::from_le_bytes([head[2], head[3], head[4], head[5]]);
    let value_len = usize::from(u16::from_le_bytes([head[6], head[7]]));
    let Some((name, rest)) = rest.split_at_checked(name_len) else {
        return;
    };
    let (value, blob) = if is_set {
        let Some(split) = rest.split_at_checked(value_len) else {
            return;
        };
        split
    } else {
        (&rest[..0], rest)
    };

    let before: Vec<Record> = store::iter(blob).collect();
    let present = before.iter().any(|(other, ..)| *other == name);
    // A record the walk read but the writer refuses: a value longer than any the store writes.
    let unwritable_other = before
        .iter()
        .any(|(other, _, other_value)| *other != name && other_value.len() > MAXIMUM_VALUE_BYTES);

    if is_set {
        let mut out = vec![0u8; blob.len() + store::record_len(name.len(), value.len())];
        let expected: Vec<Record> = if present {
            before
                .iter()
                .map(|&record| {
                    if record.0 == name {
                        (name, kind, value)
                    } else {
                        record
                    }
                })
                .collect()
        } else {
            let mut grown = before.clone();
            grown.push((name, kind, value));
            grown
        };
        // A damaged blob that holds the name twice has each copy replaced, so a longer value can
        // need more room than `out`, which is sized for a blob `set` wrote.
        let needed: usize = expected
            .iter()
            .map(|(n, _, v)| store::record_len(n.len(), v.len()))
            .sum();
        match store::set(blob, name, kind, value, &mut out) {
            Ok(n) => {
                let after: Vec<Record> = store::iter(&out[..n]).collect();
                assert_eq!(after, expected);
                assert_eq!(store::get(&out[..n], name), Some((kind, value)));
            }
            Err(ERANGE) => assert!(!is_valid_name(name)),
            Err(E2BIG) => assert!(is_valid_name(name) && value.len() > MAXIMUM_VALUE_BYTES),
            Err(EINVAL) => assert!(
                is_valid_name(name) && value.len() <= MAXIMUM_VALUE_BYTES && kind > MAXIMUM_KIND
            ),
            Err(ENOSPC) => assert!(
                is_valid_name(name)
                    && value.len() <= MAXIMUM_VALUE_BYTES
                    && kind <= MAXIMUM_KIND
                    && (unwritable_other
                        || needed > out.len()
                        || (!present && before.len() >= MAXIMUM_ATTRIBUTES))
            ),
            Err(other) => panic!("set answered {other}, which it never returns"),
        }
    } else {
        let mut out = vec![0u8; blob.len()];
        match store::remove(blob, name, &mut out) {
            Ok(n) => {
                assert!(present);
                let expected: Vec<Record> = before
                    .iter()
                    .copied()
                    .filter(|(other, ..)| *other != name)
                    .collect();
                let after: Vec<Record> = store::iter(&out[..n]).collect();
                assert_eq!(after, expected);
                assert_eq!(store::get(&out[..n], name), None);
            }
            Err(ERANGE) => assert!(!is_valid_name(name)),
            Err(ENODATA) => assert!(is_valid_name(name) && !present && !unwritable_other),
            Err(ENOSPC) => assert!(is_valid_name(name) && unwritable_other),
            Err(other) => panic!("remove answered {other}, which it never returns"),
        }
    }
});
