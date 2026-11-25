# Why the benchmarks have the shapes they have

Two benchmarks read this crate: the throughput benchmark (`throughput`) and the walk pricing
(`walk`). This page holds the reasons behind their numbers. The doc comments in `src/lib.rs` say
what each constant is and what reads it.

## Throughput

### The transfer unit is the whole file channel

`throughput::UNIT_BYTES` is the most one request
can carry, so a phase measures the protocol's own ceiling and the tuning lives at that one constant.
A figure is how fast the stack moves one request's worth. A comparison against a system whose
client passes a buffer of another size compares two different things.

### The bytes per phase stay fixed across transfer sizes

`throughput::TOTAL_BYTES` is 1 MiB, set
by the 16 MiB test image rather than chosen for statistics. The write phases fit beside everything
else the image carries, and RedoxFS is copy-on-write, so a rewritten block takes fresh space until
the old one frees. Fixing the bytes rather than the transfer count means two runs move the same file
however the client shapes the requests, so a before and an after compare. A build-time assertion holds
the phase to exactly `TOTAL_BYTES` at every unit, because the phase runs in user mode where no test
runs.

### The write phase creates the file

`THROUGHPUT_NAME` is not on the image. A file the image
already carried would have its layout from the host tool rather than from the server under test,
and the sequential write phase is that creation. The phases run in the order of their tags for the
same reason.

### The record read refuted a model

`throughput::RECORD_READ` reads the first page of each record.
The model predicted a read at a record's start would cost less than one in its middle. It measures
the same as the other read phases, within 3%, because RedoxFS fetches the block the pointer stores:
a read of an ordinary file fetches the whole record whatever offset it asked for. The phase stays so
that the measurement stays repeatable. `throughput::RECORD_BYTES` is 128 KiB, a whole number of
records at every RedoxFS record level from 0 to 5, so a change of level leaves every figure
comparable. It is the store's number, not the protocol's, and the filesystem server asserts at
compile time that it is a whole number of records.

### The payload fill is the price of honest writes

RedoxFS compresses a record with lz4 and skips
a write whose record matches what the disk already holds. A benchmark that sends the same page
twice measures the short circuit, and one that sends 32 identical pages into one record measures
lz4. Each write therefore needs fresh, incompressible bytes, which costs 512 stores a page.
`throughput::PAYLOAD_FILL` reports that cost with no filesystem in it, so a reader can subtract it
from the write phases.

### Two binaries share one role

`throughput::ROLE` is the one value two programs must
agree on rather than two modules: the boot that spawns the client and the client that matches on
its role. The other roles are literals with comments on both sides.

## The walk

`walk::ROOT` is a sibling of `tree::SUB`, granted alone and written by nothing, so every figure the
pricing prints depends only on this shape. Each region prices one cost as a slope over one variable.

- `chain` prices a path component per level. Every directory below it has the name `c`, so a deeper
  path costs components rather than name bytes. Eight levels give a slope nine points, and the
  deepest open is ten components from the grant.
- `wide` prices an entry per name. 128 names of four bytes come to 768 bytes of `READDIR` records,
  one page, so the slope is per entry and not per page. Pricing a page boundary would want more
  than 680 names.
- `narrow` is the control: the shape of `wide` with one file, so the difference is entries alone.
- `sizes` prices a byte per byte. Its smallest file is one page, and its largest is sixteen of the
  file channel's largest requests.

Every small file holds `walk::SMALL_BODY`, 16 bytes, so a small read is one request and the byte
region is the only place bytes vary. The sized files hold printable bytes with a newline every 64,
so a search tool has lines to match and the host and the guest agree on the bytes without a file.

The totals (`WALK_ENTRIES`, `WALK_FILES`, `WALK_BYTES`, `WALK_COMPONENTS`) come from the shape, not
from walking it, so a walker that skipped a directory cannot agree with them by construction.
`WALK_COMPONENTS` sums each entry's depth below the root: the hops a resolver that takes one
component at a time pays, and what a resolver of many components at once would save.

## Searching the walk with ripgrep

`walk::RG_SEARCH` is the line typed at the prompt with the tree as the working directory, and a host
run types the same words. `--threads 1` and `--no-mmap` hold the host to what the guest offered: one
thread and no `mmap`. `--stats` makes the search measurable from inside
both, since ripgrep times its own walk and search.

`walk::RG_NAMED` is the control for a refused walk. It names one file, so it needs `READ` and
`DESCEND` and no `ENUMERATE`, and it still finds its line under a grant that lacks `ENUMERATE`. A
refused walk is then about enumeration alone. It carries neither pin, because a single named file
is where ripgrep tries a memory map.

`walk::rg_match_path` gives the expected output as one ordered list: the chain's files from the top
down, then the wide files in name order, then the narrow one. The guest and the host both compare
against it, so a parallel walk that prints in another order, or prints a line twice, fails on either
side.
