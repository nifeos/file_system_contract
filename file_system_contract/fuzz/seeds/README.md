# Fuzz seeds

Inputs the five fuzz targets start from, committed on purpose. Pass a directory here as a read-only
corpus after the target's own scratch corpus:

```sh
cargo +nightly fuzz run directory_entry_walk fuzz/corpus/directory_entry_walk fuzz/seeds/directory_entry_walk
```

| Target | Seeds | What they are |
|---|---|---|
| `directory_entry_walk` | `directory_entry_walk/` | a reply with one file and one directory, and a reply whose first record has an empty name |
| `name_set_walk` | `name_set_walk/` | a set of two file names, and a set of one directory name |
| `attribute_list_walk` | `attribute_list_walk/` | a listing of two attribute names |
| `attribute_store_walk` | `attribute_store_walk/` | a blob of two records, and the same blob cut three bytes short |
| `attribute_store_edit` | `attribute_store_edit/` | a set of a new name, a set of a name already there, and a remove, each over the two-record blob |

The `attribute_store_edit` input is an operation byte (even sets, odd removes), a name length byte,
a kind (four bytes, little-endian), a value length (two bytes, little-endian), the name, the value
and then the blob.
