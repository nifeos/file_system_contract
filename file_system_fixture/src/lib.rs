//! Test data for a filesystem stack: the names and contents of the files a test image ships, the
//! witnesses' verdict words, and the shapes of the `walk` and `throughput` benchmarks. The host
//! build that writes the image, the client that reads it through the filesystem server, and the
//! test that asserts the client's report all read these constants, so the three agree.
//!
//! The reasons behind the values are in the repository's `docs/`: `witnesses.md`, `benchmarks.md`
//! and `image.md`.
//!
//! # BUGS
//!
//! - The dependency on `file_system_contract` is a path inside this repository's workspace, so this
//!   crate builds only beside it.
//! - Much of the crate describes the QEMU test runs of the stack it was written for (the witnesses, the benchmark phases), not
//!   a filesystem in general. A third party gets the file names and the shapes as examples.
//! - It ships no harness. The programs that drive the witnesses and the benchmarks live with that
//!   stack, so a third party gets the vocabulary and nothing to run against it.

#![no_std]

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeExample;

/// The version of the test disk layouts, 1. A change an older reader would misread bumps it.
pub const VERSION: u32 = 1;

/// This interface's number in a program's `INTERFACES` note, 16, declared beside [`VERSION`]. It
/// never changes; a format change is a new version.
pub const INTERFACE_ID: u64 = 16;

/// A file at the image root that the client reads back through a directory capability.
#[doc(alias = "motd")]
pub const MOTD_NAME: &str = "motd";
/// What [`MOTD_NAME`] holds; the client reports its first eight bytes as the report's head word.
#[doc(alias = "motd")]
pub const MOTD: &[u8] =
    b"redoxfs served this file to an unprivileged client through a capability handle\n";

/// A file at the image root that the client opens and overwrites.
pub const SCRATCH_NAME: &str = "scratch";
/// What the host build writes into [`SCRATCH_NAME`] before the run.
pub const SCRATCH_PLACEHOLDER: &[u8] = b"(placeholder overwritten by the client's write test)";
/// What the client writes into [`SCRATCH_NAME`] and reads back. A host check reads it again after
/// the run.
pub const WRITE_PATTERN: &[u8] = b"CRKFS_WRITE_OK: this round-tripped through the RedoxFS image\n";

/// The client's success sentinel, the second word of its report; the test refuses any other value.
pub const SUCCESS: u64 = 0xF11E_600D;

/// The file the shared-frame witness's victim names, at the image root. The witness rests on its
/// name being as long as [`SHARED_USURPER_NAME`].
pub const SHARED_VICTIM_NAME: &str = "fs-witness-victim";
/// The file an attacker substitutes for [`SHARED_VICTIM_NAME`] in the shared page, at the image
/// root.
pub const SHARED_USURPER_NAME: &str = "fs-witness-usurpr";
/// What [`SHARED_VICTIM_NAME`] holds, which the victim expects to read back.
pub const SHARED_VICTIM_BODY: &[u8] = b"CRK599-VICTIM: the file this client actually named\n";
/// What [`SHARED_USURPER_NAME`] holds, which the victim reads if the server resolved the attacker's
/// name. Its first eight bytes differ from [`SHARED_VICTIM_BODY`]'s.
pub const SHARED_USURPER_BODY: &[u8] =
    b"CRK599-USURPER: a file named by a different client entirely\n";

/// The two shared-frame names have equal length, checked at build time.
const _: () = assert!(SHARED_VICTIM_NAME.len() == SHARED_USURPER_NAME.len());

/// The shared-frame victim's verdict when the server resolved the attacker's name: the expected
/// verdict of the negative control, which shares one window.
pub const SHARED_SUBSTITUTED: u64 = 0x5995_500B;
/// The shared-frame victim's verdict when its own name resolved: the expected verdict on separate
/// windows.
pub const SHARED_ISOLATED: u64 = 0x5991_150A;
/// The shared-frame victim's verdict when the open returned neither body, which means the witness
/// is wired wrong.
pub const SHARED_UNEXPECTED: u64 = 0x599B_ADDD;

/// The file the throughput benchmark creates at the image root. It is not on the image.
pub const THROUGHPUT_NAME: &str = "throughput";

/// The throughput benchmark's shape, read by the client that runs it and the boot that names its
/// results.
pub mod throughput {
    /// Bytes per transfer: the most one `file_system_contract` request carries,
    /// [`file_system_contract::file_system::MAXIMUM_TRANSFER_BYTES`].
    pub const UNIT_BYTES: u64 = file_system_contract::file_system::MAXIMUM_TRANSFER_BYTES as u64;

    /// Bytes each phase moves, 1 MiB, the same at every transfer size.
    pub const TOTAL_BYTES: u64 = 1024 * 1024;

    /// Transfers per phase: [`TOTAL_BYTES`] divided by [`UNIT_BYTES`].
    pub const BLOCKS: u64 = TOTAL_BYTES / UNIT_BYTES;

    /// Each phase moves exactly [`TOTAL_BYTES`] at every transfer size, checked at build time.
    const _: () = {
        assert!(BLOCKS >= 1);
        assert!(BLOCKS * UNIT_BYTES == TOTAL_BYTES);
        assert!(TOTAL_BYTES.is_multiple_of(RECORD_BYTES));
    };

    /// Untimed transfers before each phase.
    pub const WARMUP_TRANSFERS: u64 = 8;

    /// The sequential write phase's tag, the third word of its report. The first phase: it
    /// creates the file the others read.
    pub const SEQUENTIAL_WRITE: u64 = 1;
    /// The sequential read phase's tag.
    pub const SEQUENTIAL_READ: u64 = 2;
    /// The tag of the phase that reads at page-aligned offsets from a pseudo-random number
    /// generator with a fixed seed.
    pub const RANDOM_READ: u64 = 3;
    /// The tag of the phase that writes in place at those same offsets.
    pub const RANDOM_WRITE: u64 = 4;
    /// The tag of the phase that reads the first page of each record.
    pub const RECORD_READ: u64 = 6;

    /// The record boundary [`RECORD_READ`] reads at, 128 KiB. The filesystem server asserts that
    /// it is a whole number of the store's records.
    pub const RECORD_BYTES: u64 = 128 * 1024;

    /// The tag of the row that reports the cost of filling one page of fresh payload, with no
    /// filesystem in it.
    pub const PAYLOAD_FILL: u64 = 5;
    /// How many phase reports the bench boot waits for.
    pub const PHASES: usize = 6;

    /// The client role the bench boot spawns and the client matches on.
    pub const ROLE: u64 = 9;

    /// The name the bench boot prints for a phase tag, or `None` for a tag this table does not
    /// know.
    pub const fn name(tag: u64) -> Option<&'static str> {
        match tag {
            SEQUENTIAL_WRITE => Some("fs_seq_write"),
            SEQUENTIAL_READ => Some("fs_seq_read"),
            RANDOM_READ => Some("fs_rand_read"),
            RANDOM_WRITE => Some("fs_rand_write"),
            RECORD_READ => Some("fs_record_read"),
            PAYLOAD_FILL => Some("fs_payload_fill"),
            _ => None,
        }
    }
}

/// The per-file attacker's verdict bits, which the test compares with an exact expected set.
pub mod escape {
    /// It opened a file the grant does not designate. Never allowed.
    pub const SECOND_FILE: u64 = 1;
    /// Its write to the granted file was accepted. Expected only with `grant::rights::WRITE`.
    pub const WROTE: u64 = 1 << 1;
    /// Its truncate of the granted file was accepted. Expected only with `grant::rights::WRITE`.
    pub const TRUNCATED: u64 = 1 << 2;
    /// It created a file through a file capability. Never allowed.
    pub const CREATED: u64 = 1 << 3;
    /// It reached a file with a handle it was never given. Never allowed.
    pub const FORGED_HANDLE: u64 = 1 << 4;
    /// Reading the granted file failed, so the refusals prove nothing. The control bit.
    pub const GRANTED_READ_FAILED: u64 = 1 << 5;
    /// Its extended-attribute write was accepted. Expected only with `grant::rights::WRITE`.
    pub const WROTE_ATTRIBUTE: u64 = 1 << 6;
    /// Reading the granted file's attributes failed. A control bit.
    pub const GRANTED_ATTRIBUTES_FAILED: u64 = 1 << 7;

    /// Every bit of this witness, in declaration order, for a consumer that checks a word holds
    /// no others.
    pub const ALL: [u64; 8] = [
        SECOND_FILE,
        WROTE,
        TRUNCATED,
        CREATED,
        FORGED_HANDLE,
        GRANTED_READ_FAILED,
        WROTE_ATTRIBUTE,
        GRANTED_ATTRIBUTES_FAILED,
    ];
}

/// The first word of the attacker's report, so a silent client never reads as a verdict of zero.
pub const VERDICT: u64 = 0xE5_CA9E00;

/// The names and contents of the image's directories and files, read by the host build that
/// stages them, the programs that open them and the host check after a run.
///
/// ```text
///   /            motd  scratch  sub/  other/
///   /sub         inner  deeper/          <- the granted capability is here
///   /sub/deeper  leaf
///   /other       secret                  <- never reachable from the grant
/// ```
pub mod tree {
    /// The directory a directory grant designates.
    pub const SUB: &str = "sub";
    /// A file in [`SUB`], pinned by the host check after a run.
    pub const INNER: &str = "inner";
    /// What [`INNER`] holds.
    pub const INNER_BODY: &[u8] = b"CRK47-INNER: a file inside the granted subtree\n";
    /// A directory in [`SUB`], for a second descent.
    pub const DEEPER: &str = "deeper";
    /// A file in [`DEEPER`], two descents below the grant.
    pub const LEAF: &str = "leaf";
    /// What [`LEAF`] holds.
    pub const LEAF_BODY: &[u8] = b"CRK47-LEAF: two descents below the granted directory\n";
    /// The sibling of [`SUB`], which a capability to [`SUB`] must not reach.
    pub const OTHER: &str = "other";
    /// A file in [`OTHER`], pinned by the host check after a run.
    pub const SECRET: &str = "secret";
    /// [`INNER`] as an absolute path, which the navigating witness opens.
    pub const ABSOLUTE_INNER: &str = "/inner";
    /// [`SECRET`] as an absolute path, which the navigating witness opens.
    pub const ABSOLUTE_SECRET: &str = "/secret";
    /// What [`SECRET`] holds.
    pub const SECRET_BODY: &[u8] = b"CRK47-SECRET: in a sibling of the granted directory\n";

    /// The prefix of the file the writable attacker creates in [`SUB`], followed by a run index.
    /// The host check looks for it in [`SUB`] and not in the root.
    pub const MADE: &str = "made-by-atk";
    /// What the attacker writes into [`MADE`] and reads back.
    pub const MADE_BODY: &[u8] = b"CRK47: written through a directory capability\n";
    /// The directory the writable attacker makes in [`SUB`] with `MKDIR`.
    pub const MADE_DIRECTORY: &str = "dir-by-atk";
    /// What the attacker renames [`MADE`] to when its capability carries
    /// [`file_system_contract::directory::rights::REMOVE`]. The host check looks for both names.
    pub const MOVED: &str = "moved-by-atk";

    /// The file the navigating shell creates and leaves in its root, run-indexed.
    pub const NAVIGATION_KEPT: &str = "nav-kept";
    /// The file the navigating shell writes and then unlinks while its handle is open, run-indexed.
    /// The host check finds it gone.
    pub const NAVIGATION_GONE: &str = "nav-gone";
    /// What goes into [`NAVIGATION_GONE`], and what the open handle still reads after the unlink.
    pub const NAVIGATION_BODY: &[u8] = b"CRK47-NAV: a name can go while the bytes stay\n";
    /// The directory the navigating shell makes with `mkdir` and leaves behind, run-indexed.
    pub const NAVIGATION_DIRECTORY: &str = "nav-dir";
    /// The directory the navigating shell makes, fills with [`NAVIGATION_INSIDE`] and removes with
    /// `RMDIR` once empty, run-indexed. The host check finds it gone.
    pub const NAVIGATION_EMPTY: &str = "nav-empty";
    /// The one name in [`NAVIGATION_EMPTY`], which makes the first `RMDIR` a refusal.
    pub const NAVIGATION_INSIDE: &str = "in";
    /// The file the navigating shell touches twice, once absent and once holding
    /// [`NAVIGATION_TOUCH_BODY`], run-indexed and left behind for the host check.
    pub const NAVIGATION_TOUCH: &str = "nav-touch";
    /// What [`NAVIGATION_TOUCH`] holds before its second touch, and must still hold after it.
    pub const NAVIGATION_TOUCH_BODY: &[u8] = b"CRK47-TOUCH: a second touch must not disturb this\n";

    /// The time `touch -t` sets in the modification-time probes, 2030-01-01T00:00:00Z, as the
    /// text of Request for Comments 3339 the command line carries.
    pub const NAVIGATION_TOUCH_AT_RFC3339: &str = "2030-01-01T00:00:00Z";
    /// [`NAVIGATION_TOUCH_AT_RFC3339`] in Unix seconds, the value the wire and a read-back use.
    pub const NAVIGATION_TOUCH_AT_UNIX_SECONDS: u64 = 1_893_456_000;
    /// The directory the navigating witness makes with `MKDIR` and attenuates to `WRITE` without
    /// `SET_MODIFICATION_TIME_AT`, run-indexed.
    pub const NAVIGATION_TOUCH_NO_SETTIME: &str = "nav-notime";

    /// The directory in [`NAVIGATION_DIRECTORY`] that `bind`'s witness binds to
    /// [`NAVIGATION_BIND_ALIAS`].
    pub const NAVIGATION_BIND_NESTED: &str = "nav-bind-nest";
    /// The file in [`NAVIGATION_BIND_NESTED`] that a listing through the bound name must show.
    pub const NAVIGATION_BIND_MARKER: &str = "marker";
    /// The top-level name [`NAVIGATION_BIND_NESTED`] is bound to, never a real entry.
    pub const NAVIGATION_BIND_ALIAS: &str = "nav-bound";

    /// The directory the redirection witness is granted, a sibling of [`SUB`].
    ///
    /// ```text
    ///   /redir      alpha  beta        <- the redirection witness is granted this
    /// ```
    pub const REDIRECTION: &str = "redir";
    /// The first file in [`REDIRECTION`].
    pub const REDIRECTION_ONE: &str = "alpha";
    /// The second file in [`REDIRECTION`].
    pub const REDIRECTION_TWO: &str = "beta";
    /// What both files in [`REDIRECTION`] hold.
    pub const REDIRECTION_BODY: &[u8] = b"CRK50: a file the redirection witness lists\n";

    /// The directory the `rm` program is granted, a sibling of [`SUB`].
    ///
    /// ```text
    ///   /rmtree                 rm-keep  rm-solo  rm-doomed/     <- the grant is here
    ///   /rmtree/rm-doomed       rm-one  rm-two  rm-nested/
    ///   /rmtree/rm-doomed/rm-nested   rm-leaf
    /// ```
    #[doc(alias = "rm")]
    pub const RMTREE: &str = "rmtree";
    /// A file in [`RMTREE`] that nothing names, so it stays. The control.
    #[doc(alias = "rm")]
    pub const RM_KEEP: &str = "rm-keep";
    /// What [`RM_KEEP`] holds.
    #[doc(alias = "rm")]
    pub const RM_KEEP_BODY: &[u8] = b"CRK47-RM: inside the grant, and never named\n";
    /// The directory `rm -r` removes.
    #[doc(alias = "rm")]
    pub const RM_DOOMED: &str = "rm-doomed";
    /// The first file at the top of [`RM_DOOMED`].
    #[doc(alias = "rm")]
    pub const RM_ONE: &str = "rm-one";
    /// The second file at the top of [`RM_DOOMED`].
    #[doc(alias = "rm")]
    pub const RM_TWO: &str = "rm-two";
    /// A directory in [`RM_DOOMED`], so `rm -r` has to recurse.
    #[doc(alias = "rm")]
    pub const RM_NESTED: &str = "rm-nested";
    /// The file in [`RM_NESTED`], the deepest `rm -r` goes.
    #[doc(alias = "rm")]
    pub const RM_LEAF: &str = "rm-leaf";
    /// What every file under [`RM_DOOMED`], and [`RM_SOLO`], holds.
    #[doc(alias = "rm")]
    pub const RM_BODY: &[u8] = b"CRK47-RM: a file `rm -r` is expected to take away\n";
    /// A file in [`RMTREE`] that a plain `rm` removes, which must exit zero and print nothing.
    #[doc(alias = "rm")]
    pub const RM_SOLO: &str = "rm-solo";
    /// A name not on the image: `rm` of it fails with a diagnostic, and `rm -f` of it succeeds in
    /// silence.
    #[doc(alias = "rm")]
    pub const RM_MISSING: &str = "rm-nothing";

    /// The directory the globbing tests plan their set grant in, a sibling of [`RMTREE`].
    ///
    /// ```text
    ///   /globset            gl-one.txt  gl-two.txt  gl-three.log  gl-dir/
    ///   /globset/gl-dir     gl-inner
    /// ```
    #[doc(alias = "glob")]
    pub const GLOBSET: &str = "globset";
    /// The pattern the shell expands, which matches exactly [`GLOB_ONE`] and [`GLOB_TWO`]. A host
    /// test holds it against the matcher.
    #[doc(alias = "glob")]
    pub const GLOB_PATTERN: &[u8] = b"gl-*.txt";
    /// The first name [`GLOB_PATTERN`] matches.
    #[doc(alias = "glob")]
    pub const GLOB_ONE: &str = "gl-one.txt";
    /// The second name [`GLOB_PATTERN`] matches.
    #[doc(alias = "glob")]
    pub const GLOB_TWO: &str = "gl-two.txt";
    /// A file in [`GLOBSET`] the pattern does not match, which a set capability must not reach.
    #[doc(alias = "glob")]
    pub const GLOB_MISS: &str = "gl-three.log";
    /// A directory in [`GLOBSET`] the pattern does not match.
    #[doc(alias = "glob")]
    pub const GLOB_DIRECTORY: &str = "gl-dir";
    /// A file in [`GLOB_DIRECTORY`], so that directory is not empty.
    #[doc(alias = "glob")]
    pub const GLOB_INNER: &str = "gl-inner";
    /// What every file under [`GLOBSET`] holds.
    #[doc(alias = "glob")]
    pub const GLOB_BODY: &[u8] = b"CRK47-GLOB: a name a pattern either matched or did not\n";
    /// A pattern that matches nothing in [`GLOBSET`], for the empty-expansion refusal.
    #[doc(alias = "glob")]
    pub const GLOB_NOMATCH: &[u8] = b"gl-*.rs";
    /// A line with no pattern and two consecutive spaces, which `echo` must print unchanged.
    #[doc(alias = "glob")]
    pub const GLOB_PLAIN: &[u8] = b"two  spaces";

    /// The directory that holds more matching names than one grant carries, for `xargs`; a sibling
    /// of [`GLOBSET`].
    #[doc(alias = "glob")]
    pub const GLOBMANY: &str = "globmany";
    /// The pattern that matches every one of [`MANY_NAMES`] and not [`MANY_MISS`]. A host test
    /// holds it against the matcher.
    pub const MANY_PATTERN: &[u8] = b"m-*.txt";
    /// The eleven matching files in [`GLOBMANY`]: one full batch of eight and one short batch.
    pub const MANY_NAMES: [&str; 11] = [
        "m-00.txt", "m-01.txt", "m-02.txt", "m-03.txt", "m-04.txt", "m-05.txt", "m-06.txt",
        "m-07.txt", "m-08.txt", "m-09.txt", "m-10.txt",
    ];
    /// The first batch as the prompt prints it: the eight smallest matches, separated by spaces.
    pub const MANY_FIRST_BATCH: &str =
        "m-00.txt m-01.txt m-02.txt m-03.txt m-04.txt m-05.txt m-06.txt m-07.txt";
    /// The second batch as the prompt prints it, the short one.
    pub const MANY_SECOND_BATCH: &str = "m-08.txt m-09.txt m-10.txt";
    /// A file in [`GLOBMANY`] the pattern does not match.
    pub const MANY_MISS: &str = "m-keep.log";
    /// What every file in [`GLOBMANY`] holds.
    pub const MANY_BODY: &[u8] = b"CRK109-XARGS: a name in a match too large to hand over\n";

    /// The names the image root must still hold after a run, which the host check asserts by
    /// containment.
    pub const ROOT_ENTRIES: [&str; 9] = [
        GLOBMANY,
        GLOBSET,
        crate::MOTD_NAME,
        OTHER,
        REDIRECTION,
        RMTREE,
        crate::SCRATCH_NAME,
        SUB,
        crate::walk::ROOT,
    ];
}

/// The tree the walk pricing is granted and walks: four regions under [`ROOT`](walk::ROOT), each
/// priced over one variable.
pub mod walk {
    /// The directory the pricing is granted, at the image root.
    pub const ROOT: &str = "walk";
    /// The depth region's top directory.
    pub const CHAIN: &str = "chain";
    /// The name of every directory below [`CHAIN`].
    pub const LEVEL: &str = "c";
    /// How many directories nest below [`CHAIN`].
    pub const DEPTH: usize = 8;
    /// The file at every level of the chain, [`CHAIN`] included.
    pub const LEVEL_FILE: &str = "f";
    /// The width region: one directory holding [`WIDE_FILES`] files.
    pub const WIDE: &str = "wide";
    /// Files in [`WIDE`], few enough that the listing fits one page of
    /// [`file_system_contract::directory_entry`] records.
    pub const WIDE_FILES: usize = 128;
    /// The control: [`WIDE`]'s shape with one file.
    pub const NARROW: &str = "narrow";
    /// The byte region.
    pub const BYTE_REGION: &str = "sizes";
    /// The files in [`BYTE_REGION`] and their sizes, each byte from [`sized_byte`].
    pub const BYTE_REGION_FILES: [(&str, usize); 3] =
        [("s4k", 4096), ("s64k", 65_536), ("s256k", 262_144)];
    /// What every file in [`CHAIN`], [`WIDE`] and [`NARROW`] holds.
    pub const SMALL_BODY: &[u8] = b"walk entry body\n";

    /// The byte at offset `i` of a file in [`BYTE_REGION`]: a lowercase letter, with a newline
    /// every 64 bytes.
    pub const fn sized_byte(i: usize) -> u8 {
        if i % 64 == 63 {
            b'\n'
        } else {
            b'a' + (i % 26) as u8
        }
    }

    /// The name of file `i` in [`WIDE`], and of the one file in [`NARROW`]: `n` and three digits.
    /// `i` is below [`WIDE_FILES`].
    pub const fn wide_name(i: usize) -> [u8; 4] {
        debug_assert!(i < WIDE_FILES);
        [
            b'n',
            b'0' + (i / 100 % 10) as u8,
            b'0' + (i / 10 % 10) as u8,
            b'0' + (i % 10) as u8,
        ]
    }

    /// Directories a full walk from [`ROOT`] lists: the four regions and the chain's levels.
    pub const WALK_DIRECTORIES: usize = 4 + DEPTH;
    /// Files a full walk opens.
    pub const WALK_FILES: usize = (DEPTH + 1) + WIDE_FILES + 1 + BYTE_REGION_FILES.len();
    /// Entries `read_dir` hands a full walk, each directory and file once. The priced program
    /// asserts it.
    pub const WALK_ENTRIES: usize = WALK_DIRECTORIES + WALK_FILES;
    /// Bytes a full walk reads.
    pub const WALK_BYTES: usize = {
        let mut sized = 0;
        let mut i = 0;
        while i < BYTE_REGION_FILES.len() {
            sized += BYTE_REGION_FILES[i].1;
            i += 1;
        }
        (WALK_FILES - BYTE_REGION_FILES.len()) * SMALL_BODY.len() + sized
    };
    /// Path components a full walk resolves: each entry's depth below [`ROOT`], summed, computed
    /// from the shape.
    pub const WALK_COMPONENTS: usize = {
        // The four regions, at depth 1.
        let mut sum = 4;
        // The chain: level `i` (the chain itself is level 0, at depth 1) holds its entries at
        // depth `i + 2`, two of them at every level but the last, which holds only the file.
        let mut i = 0;
        while i <= DEPTH {
            let here = if i < DEPTH { 2 } else { 1 };
            sum += here * (i + 2);
            i += 1;
        }
        // The wide, narrow and sized files, at depth 2.
        sum + 2 * (WIDE_FILES + 1 + BYTE_REGION_FILES.len())
    };

    /// The `ripgrep` command line the guest and a host both run over the tree, with one thread,
    /// no memory map and its own timing.
    #[doc(alias = "rg")]
    pub const RG_SEARCH: &str = "rg --threads 1 --no-mmap --stats 'walk entry'";
    /// Lines [`RG_SEARCH`] matches: one in every file holding [`SMALL_BODY`].
    pub const RG_MATCHES: usize = WALK_FILES - BYTE_REGION_FILES.len();
    /// The same pattern naming one file, which needs no `ENUMERATE`: the control for a refused
    /// walk.
    #[doc(alias = "rg")]
    pub const RG_NAMED: &str = "rg 'walk entry' narrow/n000";

    /// [`RG_SEARCH`] without `--threads 1`, so `ripgrep` picks its own thread count.
    #[doc(alias = "rg")]
    pub const RG_SEARCH_THREADED: &str = "rg --no-mmap --stats 'walk entry'";
    /// [`RG_SEARCH`] told four threads.
    #[doc(alias = "rg")]
    pub const RG_SEARCH_FOUR_THREADS: &str = "rg --threads 4 --no-mmap --stats 'walk entry'";
    /// The thread count [`RG_SEARCH_FOUR_THREADS`] names.
    pub const RG_FOUR: usize = 4;

    /// The size of the buffer [`rg_match_path`] writes into, longer than every path it writes.
    pub const MATCH_PATH_BYTES: usize = 32;

    /// Writes the path of the `i`th file [`RG_SEARCH`] matches into `buf`, relative to [`ROOT`] as
    /// `ripgrep` prints it, and returns its length. `i` is below [`RG_MATCHES`].
    pub fn rg_match_path(i: usize, buf: &mut [u8; MATCH_PATH_BYTES]) -> usize {
        debug_assert!(i < RG_MATCHES);
        let mut len = 0;
        let mut put = |bytes: &[u8], len: &mut usize| {
            buf[*len..*len + bytes.len()].copy_from_slice(bytes);
            *len += bytes.len();
        };
        if i <= DEPTH {
            put(CHAIN.as_bytes(), &mut len);
            put(b"/", &mut len);
            for _ in 0..i {
                put(LEVEL.as_bytes(), &mut len);
                put(b"/", &mut len);
            }
            put(LEVEL_FILE.as_bytes(), &mut len);
        } else {
            let (directory, n) = if i - (DEPTH + 1) < WIDE_FILES {
                (WIDE, i - (DEPTH + 1))
            } else {
                (NARROW, 0)
            };
            put(directory.as_bytes(), &mut len);
            put(b"/", &mut len);
            put(&wide_name(n), &mut len);
        }
        len
    }
}

/// The directory attacker's verdict bits, which the test compares with an exact expected set. The
/// attacker holds a grant to [`tree::SUB`].
pub mod directory_escape {
    /// It opened a file only the granted directory's parent holds. Never allowed.
    pub const REACHED_PARENT: u64 = 1;
    /// It reached anything in the granted directory's sibling. Never allowed.
    pub const REACHED_SIBLING: u64 = 1 << 1;
    /// `..` resolved to something. Never allowed.
    pub const WALKED_UP: u64 = 1 << 2;
    /// It asked for a right its capability did not carry, and got it. Never allowed.
    pub const WIDENED: u64 = 1 << 3;
    /// It enumerated the granted directory. Expected only with
    /// [`file_system_contract::directory::rights::ENUMERATE`].
    pub const ENUMERATED: u64 = 1 << 4;
    /// It descended one level inside the grant. Expected only with
    /// [`file_system_contract::directory::rights::DESCEND`].
    pub const DESCENDED: u64 = 1 << 5;
    /// It created a name inside the grant. Expected only with
    /// [`file_system_contract::directory::rights::CREATE`].
    pub const CREATED: u64 = 1 << 6;
    /// It made a directory inside the grant and got a capability to it. Expected only with
    /// [`file_system_contract::directory::rights::CREATE`] and
    /// [`file_system_contract::directory::rights::DESCEND`] together.
    pub const MADE_A_DIRECTORY: u64 = 1 << 12;
    /// Its write to a file inside the grant was accepted and read back. Expected only with
    /// [`file_system_contract::directory::rights::WRITE`].
    pub const WROTE: u64 = 1 << 7;
    /// It renamed a name inside its grant. Expected only with
    /// [`file_system_contract::directory::rights::REMOVE`] and
    /// [`file_system_contract::directory::rights::CREATE`] together.
    pub const RENAMED: u64 = 1 << 8;
    /// A listing it was allowed to make named something outside the granted directory. Never
    /// allowed.
    pub const ENUMERATED_A_STRANGER: u64 = 1 << 9;
    /// It reached something with a handle it was never given. Never allowed.
    pub const FORGED_HANDLE: u64 = 1 << 10;
    /// It opened the file inside its own grant. A control bit.
    pub const OPENED_ITS_OWN: u64 = 1 << 13;
    /// Something it should be able to do failed, so the refusals prove nothing. A control bit.
    pub const GRANTED_ACCESS_FAILED: u64 = 1 << 11;
    /// It listed the extended attributes of the file inside its grant. Expected wherever
    /// [`file_system_contract::directory::rights::READ`] is held.
    pub const READ_ATTRIBUTES: u64 = 1 << 14;
    /// It set an extended attribute on that file and read it back with its type code. Expected
    /// only with [`file_system_contract::directory::rights::WRITE`].
    pub const SET_AN_ATTRIBUTE: u64 = 1 << 15;
    /// It read a name in the granted directory that the grant's name set does not carry. Never
    /// allowed.
    pub const REACHED_AN_UNMATCHED_NAME: u64 = 1 << 16;

    /// Every bit of this witness, in declaration order, for a consumer that checks a word holds
    /// no others.
    pub const ALL: [u64; 17] = [
        REACHED_PARENT,
        REACHED_SIBLING,
        WALKED_UP,
        WIDENED,
        ENUMERATED,
        DESCENDED,
        CREATED,
        MADE_A_DIRECTORY,
        WROTE,
        RENAMED,
        ENUMERATED_A_STRANGER,
        FORGED_HANDLE,
        OPENED_ITS_OWN,
        GRANTED_ACCESS_FAILED,
        READ_ATTRIBUTES,
        SET_AN_ATTRIBUTE,
        REACHED_AN_UNMATCHED_NAME,
    ];
}

/// The two-directory witness's verdict bits: one process holding two subtree caretakers in two
/// capability table slots, compared with an exact expected set.
pub mod endpoint_boundary {
    /// It opened the first grant's file through the first grant's endpoint. A control bit.
    pub const OPENED_A: u64 = 1;
    /// It opened the second grant's file through the second grant's endpoint. A control bit.
    pub const OPENED_B: u64 = 1 << 1;
    /// It opened the second grant's file through the first grant's endpoint. Never allowed.
    pub const REACHED_B_VIA_A: u64 = 1 << 2;
    /// It opened the first grant's file through the second grant's endpoint. Never allowed.
    pub const REACHED_A_VIA_B: u64 = 1 << 3;
    /// Client-side resolution did not refuse a path that ascends out of one grant and into the
    /// other, such as `/a/../b`. Never allowed.
    pub const DOT_DOT_CROSSED_MOUNTS: u64 = 1 << 4;
    /// Something it should be able to do failed, so the refusals prove nothing. A control bit.
    pub const GRANTED_ACCESS_FAILED: u64 = 1 << 5;

    /// Every bit of this witness, in declaration order, for a consumer that checks a word holds
    /// no others.
    pub const ALL: [u64; 6] = [
        OPENED_A,
        OPENED_B,
        REACHED_B_VIA_A,
        REACHED_A_VIA_B,
        DOT_DOT_CROSSED_MOUNTS,
        GRANTED_ACCESS_FAILED,
    ];
}

/// The verdict bits of a shell holding two trees, [`tree::SUB`] as its root and [`tree::OTHER`]
/// mounted at [`MOUNT`](mounted_tree::MOUNT), compared with an exact expected set. Provisional
/// names, all of them.
pub mod mounted_tree {
    /// Where the witness mounts the second tree, as the shell's own path.
    pub const MOUNT: &str = "/second";
    /// `pwd` printed `/` at the start.
    pub const PWD_STARTS_AT_ROOT: u64 = 1;
    /// `ls /` named [`crate::tree::INNER`] and the mount point, and not [`crate::tree::SECRET`].
    pub const LISTED_ROOT: u64 = 1 << 1;
    /// `cd` to the mount point moved into the second tree, and `pwd` printed the mount path.
    pub const MOVED_INTO_MOUNT: u64 = 1 << 2;
    /// `ls` inside the mount named [`crate::tree::SECRET`] and not [`crate::tree::INNER`].
    pub const LISTED_MOUNT: u64 = 1 << 3;
    /// A relative name opened inside the mount, through the second tree's endpoint.
    pub const OPENED_RELATIVE_IN_MOUNT: u64 = 1 << 4;
    /// `/inner` opened from inside the mount without moving the shell.
    pub const OPENED_ROOT_FROM_MOUNT: u64 = 1 << 5;
    /// `cd ..` at the mount point moved to its parent in the first tree, and a relative `cd` came
    /// back.
    pub const UP_FROM_MOUNT_IS_PARENT: u64 = 1 << 6;
    /// `..` past the root was refused and nothing moved, from the root and from inside the mount.
    pub const ROOT_CLAMPED: u64 = 1 << 7;
    /// `<` from a file under the mount read [`crate::tree::SECRET_BODY`].
    pub const REDIRECTED_FROM_MOUNT: u64 = 1 << 9;
    /// A bare `cd` came back to `/` from inside the mount.
    pub const HOME_IS_ROOT: u64 = 1 << 10;
    /// A name bound to the mount point listed the second tree's files, and binding the mount
    /// point's own name was refused.
    pub const BOUND_INTO_MOUNT: u64 = 1 << 11;
    /// `rm` under the mount was planned and then refused at delivery rather than sent.
    pub const RM_IN_MOUNT_REFUSED: u64 = 1 << 12;

    /// A refused move moved the shell. Never allowed.
    pub const MOVED_ON_REFUSAL: u64 = 1 << 16;
    /// `..` past the root was not refused. Never allowed.
    pub const CLIMBED_OUT: u64 = 1 << 17;
    /// A name that exists only in one tree opened through the other. Never allowed.
    pub const REACHED_ACROSS: u64 = 1 << 18;
    /// Nothing reachable was reached, so nothing above proves anything. A control bit.
    pub const TWO_TREES_FAILED: u64 = 1 << 19;

    /// Every bit of this witness, in declaration order, for a consumer that checks a word holds
    /// no others.
    pub const ALL: [u64; 16] = [
        PWD_STARTS_AT_ROOT,
        LISTED_ROOT,
        MOVED_INTO_MOUNT,
        LISTED_MOUNT,
        OPENED_RELATIVE_IN_MOUNT,
        OPENED_ROOT_FROM_MOUNT,
        UP_FROM_MOUNT_IS_PARENT,
        ROOT_CLAMPED,
        REDIRECTED_FROM_MOUNT,
        HOME_IS_ROOT,
        BOUND_INTO_MOUNT,
        RM_IN_MOUNT_REFUSED,
        MOVED_ON_REFUSAL,
        CLIMBED_OUT,
        REACHED_ACROSS,
        TWO_TREES_FAILED,
    ];
}

/// The navigating shell's verdict bits, compared with an exact expected set. Two shells rooted in
/// [`tree::SUB`] and [`tree::OTHER`] run the same script, and the test reads the pair of reports.
pub mod navigation {
    /// `pwd` printed `/` before anything moved.
    pub const PWD_IS_ROOT: u64 = 1;
    /// `cd ..` at the root was refused and nothing moved.
    pub const CLAMPED_AT_ROOT: u64 = 1 << 1;
    /// `cd ..` at the root moved the shell. Never allowed.
    pub const WALKED_UP: u64 = 1 << 2;
    /// From one level down, `cd /` came back to the shell's own root and `pwd` printed `/`.
    pub const ABSOLUTE_IS_MY_ROOT: u64 = 1 << 3;
    /// It opened [`crate::tree::INNER`], which only [`crate::tree::SUB`] holds.
    pub const REACHED_INNER: u64 = 1 << 4;
    /// It opened [`crate::tree::SECRET`], which only [`crate::tree::OTHER`] holds.
    pub const REACHED_SECRET: u64 = 1 << 5;
    /// `ls` of its own root returned a listing.
    pub const LISTED: u64 = 1 << 6;
    /// That listing named [`crate::tree::INNER`].
    pub const SAW_INNER: u64 = 1 << 7;
    /// That listing named [`crate::tree::SECRET`].
    pub const SAW_SECRET: u64 = 1 << 8;
    /// `cd` into a child directory worked and `pwd` then printed the child's path.
    pub const DESCENDED: u64 = 1 << 9;
    /// `cd ..` from there came back, and `pwd` printed `/` again.
    pub const RETURNED: u64 = 1 << 10;
    /// It created both of its files.
    pub const CREATED: u64 = 1 << 11;
    /// `mkdir` made a directory in its root.
    pub const MADE_DIRECTORY: u64 = 1 << 12;
    /// `rm` removed the name it had made.
    pub const UNLINKED: u64 = 1 << 13;
    /// After that `rm`, a read through the handle it still held returned the bytes.
    pub const HOLDER_KEPT_READING: u64 = 1 << 14;
    /// After that `rm`, the removed name no longer opens.
    pub const NAME_GONE_AFTER_UNLINK: u64 = 1 << 15;
    /// `UNLINK` of a directory was refused.
    pub const UNLINK_REFUSED_A_DIRECTORY: u64 = 1 << 16;
    /// Something that should have worked did not, so the refusals prove nothing. A control bit.
    pub const NAVIGATION_FAILED: u64 = 1 << 17;
    /// `RMDIR` of a directory with a name in it was refused with `ENOTEMPTY`.
    pub const RMDIR_REFUSED_NON_EMPTY: u64 = 1 << 18;
    /// `RMDIR` removed that directory once it was empty.
    pub const RMDIR_REMOVED_EMPTY: u64 = 1 << 19;
    /// It opened [`crate::tree::ABSOLUTE_INNER`], which only [`crate::tree::SUB`] holds.
    pub const ABSOLUTE_REACHED_INNER: u64 = 1 << 20;
    /// It opened [`crate::tree::ABSOLUTE_SECRET`], which only [`crate::tree::OTHER`] holds.
    pub const ABSOLUTE_REACHED_SECRET: u64 = 1 << 21;
    /// `cd /..` was refused and nothing moved.
    pub const ABSOLUTE_CLAMPED_AT_ROOT: u64 = 1 << 22;
    /// `touch` on an absent name created it, empty.
    pub const TOUCH_CREATED: u64 = 1 << 23;
    /// `touch` on [`crate::tree::NAVIGATION_TOUCH`] left [`crate::tree::NAVIGATION_TOUCH_BODY`] in
    /// it.
    pub const TOUCH_PRESERVED: u64 = 1 << 24;
    /// A second bare `touch` read back a strictly later modification time through
    /// `GET_MODIFICATION_TIME_UNIX_SECONDS`.
    pub const TOUCH_MODIFICATION_TIME_ADVANCED: u64 = 1 << 25;
    /// `touch -t` read back exactly [`crate::tree::NAVIGATION_TOUCH_AT_UNIX_SECONDS`].
    pub const TOUCH_AT_ROUND_TRIPPED: u64 = 1 << 26;
    /// A handle with `WRITE` and without `SET_MODIFICATION_TIME_AT` set a name's modification time
    /// to now.
    pub const TOUCH_NOW_NEEDS_ONLY_WRITE: u64 = 1 << 27;
    /// The same handle's `touch -t` was refused.
    pub const TOUCH_AT_REFUSED_WITHOUT_SETTIME: u64 = 1 << 28;
    /// `ls` of [`crate::tree::NAVIGATION_BIND_ALIAS`] listed
    /// [`crate::tree::NAVIGATION_BIND_MARKER`].
    pub const BIND_REACHED_TARGET: u64 = 1 << 29;
    /// One `cd ..` from inside the bound name printed [`crate::tree::NAVIGATION_DIRECTORY`]'s real
    /// path.
    pub const BIND_ASCEND_REACHES_REAL_PARENT: u64 = 1 << 30;
    /// A further `cd ..` was refused at the shell's own root.
    pub const BIND_STOPS_AT_TRUE_ROOT: u64 = 1 << 31;

    /// Every bit of this witness, in declaration order, for a consumer that checks a word holds
    /// no others.
    pub const ALL: [u64; 32] = [
        PWD_IS_ROOT,
        CLAMPED_AT_ROOT,
        WALKED_UP,
        ABSOLUTE_IS_MY_ROOT,
        REACHED_INNER,
        REACHED_SECRET,
        LISTED,
        SAW_INNER,
        SAW_SECRET,
        DESCENDED,
        RETURNED,
        CREATED,
        MADE_DIRECTORY,
        UNLINKED,
        HOLDER_KEPT_READING,
        NAME_GONE_AFTER_UNLINK,
        UNLINK_REFUSED_A_DIRECTORY,
        NAVIGATION_FAILED,
        RMDIR_REFUSED_NON_EMPTY,
        RMDIR_REMOVED_EMPTY,
        ABSOLUTE_REACHED_INNER,
        ABSOLUTE_REACHED_SECRET,
        ABSOLUTE_CLAMPED_AT_ROOT,
        TOUCH_CREATED,
        TOUCH_PRESERVED,
        TOUCH_MODIFICATION_TIME_ADVANCED,
        TOUCH_AT_ROUND_TRIPPED,
        TOUCH_NOW_NEEDS_ONLY_WRITE,
        TOUCH_AT_REFUSED_WITHOUT_SETTIME,
        BIND_REACHED_TARGET,
        BIND_ASCEND_REACHES_REAL_PARENT,
        BIND_STOPS_AT_TRUE_ROOT,
    ];
}

/// The globbing witness's verdict bits, compared with an exact expected set. The shell runs over
/// [`tree::GLOBSET`] through a real subtree caretaker.
pub mod globbing {
    /// `echo` expanded the pattern into a non-empty set.
    pub const EXPANDED: u64 = 1;
    /// The names `echo` printed and the names the grant carries are the same bytes.
    pub const AGREED: u64 = 1 << 1;
    /// The expansion holds neither [`crate::tree::GLOB_MISS`] nor [`crate::tree::GLOB_DIRECTORY`].
    pub const EXCLUDED_A_STRANGER: u64 = 1 << 2;
    /// A pattern that matched nothing was refused rather than printed as itself.
    pub const NO_MATCH_REFUSED: u64 = 1 << 3;
    /// A pattern in a leading path component was refused.
    pub const PATTERN_IN_PATH_REFUSED: u64 = 1 << 4;
    /// [`crate::tree::GLOB_PLAIN`] came out of `echo` byte for byte.
    pub const TEXT_UNTOUCHED: u64 = 1 << 5;
    /// Something that should have worked did not, so nothing above proves anything. A control
    /// bit.
    pub const GLOB_FAILED: u64 = 1 << 6;

    /// Every bit of this witness, in declaration order, for a consumer that checks a word holds
    /// no others.
    pub const ALL: [u64; 7] = [
        EXPANDED,
        AGREED,
        EXCLUDED_A_STRANGER,
        NO_MATCH_REFUSED,
        PATTERN_IN_PATH_REFUSED,
        TEXT_UNTOUCHED,
        GLOB_FAILED,
    ];
}

/// The extended-attribute witness's verdict bits, the third word of the filesystem client's report,
/// compared with [`EXPECTED`](attributes::EXPECTED); and the attribute it writes.
pub mod attributes {
    /// An attribute was set and read back with its type code and its bytes.
    pub const SET_AND_READ_BACK: u64 = 1;
    /// The listing named that attribute and nothing else.
    pub const LISTED: u64 = 1 << 1;
    /// The attribute followed its file across a rename.
    pub const SURVIVED_RENAME: u64 = 1 << 2;
    /// After a remove, reading it answers `ENODATA`.
    pub const GONE_AFTER_REMOVE: u64 = 1 << 3;
    /// A file made again at a name whose previous file had attributes has none.
    pub const GONE_AFTER_UNLINK: u64 = 1 << 4;
    /// A value longer than [`file_system_contract::extended_attribute::MAXIMUM_VALUE_BYTES`] was
    /// refused with [`file_system_contract::extended_attribute::E2BIG`].
    pub const OVERSIZE_REFUSED: u64 = 1 << 5;
    /// The store's own directory could not be opened, created or descended into.
    pub const STORE_UNNAMEABLE: u64 = 1 << 6;
    /// A full, non-empty listing of the root did not name the store's directory.
    pub const STORE_UNLISTED: u64 = 1 << 7;
    /// Something that should have worked did not, so nothing above proves anything. A control
    /// bit.
    pub const ATTRIBUTES_FAILED: u64 = 1 << 8;

    /// Every bit of this witness, in declaration order, for a consumer that checks a word holds
    /// no others.
    pub const ALL: [u64; 9] = [
        SET_AND_READ_BACK,
        LISTED,
        SURVIVED_RENAME,
        GONE_AFTER_REMOVE,
        GONE_AFTER_UNLINK,
        OVERSIZE_REFUSED,
        STORE_UNNAMEABLE,
        STORE_UNLISTED,
        ATTRIBUTES_FAILED,
    ];

    /// What a correct run reports: bits 0 through 7. A test holds it equal to their union.
    pub const EXPECTED: u64 = 0xFF;

    /// The file the witness makes, sets an attribute on and renames.
    pub const PROBE: &str = "attr-probe";
    /// What the witness renames [`PROBE`] to.
    pub const PROBE_MOVED: &str = "attr-moved";
    /// The name of the attribute the witness writes, in the form Samba uses.
    pub const NAME: &[u8] = b"user.com.apple.metadata";
    /// The attribute's value: opaque bytes.
    pub const VALUE: &[u8] = b"CRK57: an opaque byte string\n";
    /// The attribute's type code, `'CSTR'` as the `BeOS` filesystem spells it.
    pub const KIND: u32 = 0x4353_5452;
}

/// The crash test's vocabulary: the file it writes on its own disk, the two payloads of equal
/// length, and the words the server and the verifier send.
pub mod crash {
    /// The file the crash driver writes.
    pub const NAME: &str = "cut";
    /// What the crash disk's [`NAME`] holds before the run.
    pub const INITIAL: &[u8] = b"CRK37-INITIAL: the value the host tool wrote before this boot ran";
    /// Payload A, written and acknowledged before the server is killed. It must survive.
    pub const A: &[u8] = b"CRK37-PAYLOAD-A: acknowledged before the kill, must be whole after";
    /// Payload B, the write the server dies in. Whole or absent, never partial.
    pub const B: &[u8] = b"CRK37-PAYLOAD-B: the write the server was killed in the middle of!";

    /// The word the filesystem server sends on its readiness endpoint just before it traps.
    pub const CUT: u64 = 0x0C07_DEAD;

    /// The verifier's report when [`NAME`] held [`A`] after recovery.
    pub const SAW_A: u64 = 0x0037_00A0;
    /// The verifier's report when [`NAME`] held [`B`] after recovery.
    pub const SAW_B: u64 = 0x0037_00B0;
    /// The verifier's report when [`NAME`] held anything else. The failure the test exists for.
    pub const SAW_NEITHER: u64 = 0x0037_00FF;
}

/// The blank disk the partitioner and the filesystem maker write, and the layout they and the host
/// check agree on. It arrives as zeros with no partition table.
pub mod blank {
    /// The disk's size in 512-byte logical blocks: 64 MiB.
    pub const DISK_BLOCKS: u64 = 131_072;
    /// The logical block size the table counts in, the contract's.
    pub const LOGICAL_BLOCK_BYTES: u64 = file_system_contract::block::LOGICAL_BLOCK_BYTES;

    /// The Extensible Firmware Interface system partition: first and last logical block,
    /// inclusive. Left unformatted.
    pub const EFI_SYSTEM_PARTITION: (u64, u64) = (2048, 10239);
    /// The Linux filesystem partition, left unformatted.
    pub const LINUX: (u64, u64) = (10240, 30719);
    /// The data partition, which the filesystem maker formats. Its start and length are whole
    /// RedoxFS blocks.
    pub const DATA: (u64, u64) = (30720, 129_023);

    /// How many partitions the table carries.
    pub const PARTITIONS: usize = 3;

    /// The name written into each partition entry, in order.
    pub const NAMES: [&str; PARTITIONS] = ["boot", "root", "data"];

    /// The file the filesystem maker creates on the new filesystem, which the host reads back.
    pub const MADE_NAME: &str = "made-on-target";
    /// What [`MADE_NAME`] holds, checked byte for byte on the host after the run.
    pub const MADE_BODY: &[u8] = b"CRK57: this filesystem was created on the target\n";
}

#[cfg(test)]
mod tests;
