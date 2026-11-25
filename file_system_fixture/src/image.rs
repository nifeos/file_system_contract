//! The test image as a plan a harness can run. [`for_each`] visits every directory and file the
//! image holds, parents first, so a harness can stage it through any filesystem. [`verify`] reads a
//! staged image back through a [`Reader`] and reports the first place it differs from the plan.
//! `examples/host_directory.rs` does both against a host directory with `std::fs`.

use crate::{tree, walk};

/// What a file on the image holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    /// Exactly these bytes.
    Bytes(&'static [u8]),
    /// This many bytes, the byte at offset `i` being [`walk::sized_byte`] of `i`.
    Sized(usize),
}

impl Body {
    /// The file's length in bytes.
    pub const fn len(&self) -> usize {
        match self {
            Body::Bytes(bytes) => bytes.len(),
            Body::Sized(len) => *len,
        }
    }

    /// Whether the file is empty. No file on the image is.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The byte at offset `i`, which is below [`len`](Body::len).
    pub const fn byte(&self, i: usize) -> u8 {
        match self {
            Body::Bytes(bytes) => bytes[i],
            Body::Sized(_) => walk::sized_byte(i),
        }
    }

    /// Copies the bytes from `offset` on into `buf` and returns how many it copied, 0 at the end.
    pub fn fill(&self, offset: usize, buf: &mut [u8]) -> usize {
        let n = buf.len().min(self.len().saturating_sub(offset));
        for (k, slot) in buf[..n].iter_mut().enumerate() {
            *slot = self.byte(offset + k);
        }
        n
    }
}

/// Whether an entry is a directory or a file, and what the file holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A directory, created empty before anything inside it.
    Directory,
    /// A file holding this body.
    File(Body),
}

/// One directory or file on the image, as [`for_each`] visits it.
#[derive(Clone, Copy, Debug)]
pub struct Entry<'a> {
    /// The entry's position in the visit order, which a [`Mismatch`] names.
    pub index: usize,
    /// The entry's path below the image root, one name per component.
    pub path: &'a [&'a [u8]],
    /// What the entry is.
    pub kind: Kind,
}

/// How many entries [`for_each`] visits: the root's files and trees, then the walk tree.
pub const ENTRIES: usize =
    4 + 4 + 2 + 3 + 8 + 6 + (tree::MANY_NAMES.len() + 2) + 1 + walk::WALK_ENTRIES;

/// The most entries one directory may hold, which bounds the bitmap [`verify`] keeps per listing.
pub const DIRECTORY_SLOTS: usize = 256;

const _: () = assert!(walk::WIDE_FILES <= DIRECTORY_SLOTS);

/// Visits every directory and file on the image, each parent before anything inside it, and stops
/// at the first error `visit` returns. The root itself is not an entry.
pub fn for_each<E>(mut visit: impl FnMut(&Entry<'_>) -> Result<(), E>) -> Result<(), E> {
    let mut index = 0;
    let mut put = |path: &[&[u8]], kind: Kind| {
        let entry = Entry { index, path, kind };
        index += 1;
        visit(&entry)
    };
    root_files(&mut put)?;
    shell_trees(&mut put)?;
    walk_tree(&mut put)
}

const DIRECTORY: Kind = Kind::Directory;

const fn file(bytes: &'static [u8]) -> Kind {
    Kind::File(Body::Bytes(bytes))
}

fn root_files<E>(put: &mut impl FnMut(&[&[u8]], Kind) -> Result<(), E>) -> Result<(), E> {
    use crate::*;
    put(&[MOTD_NAME.as_bytes()], file(MOTD))?;
    put(&[SCRATCH_NAME.as_bytes()], file(SCRATCH_PLACEHOLDER))?;
    put(&[SHARED_VICTIM_NAME.as_bytes()], file(SHARED_VICTIM_BODY))?;
    put(&[SHARED_USURPER_NAME.as_bytes()], file(SHARED_USURPER_BODY))
}

fn shell_trees<E>(put: &mut impl FnMut(&[&[u8]], Kind) -> Result<(), E>) -> Result<(), E> {
    use tree::*;
    let sub = SUB.as_bytes();
    let deeper = DEEPER.as_bytes();
    put(&[sub], DIRECTORY)?;
    put(&[sub, INNER.as_bytes()], file(INNER_BODY))?;
    put(&[sub, deeper], DIRECTORY)?;
    put(&[sub, deeper, LEAF.as_bytes()], file(LEAF_BODY))?;
    put(&[OTHER.as_bytes()], DIRECTORY)?;
    put(&[OTHER.as_bytes(), SECRET.as_bytes()], file(SECRET_BODY))?;

    let redirection = REDIRECTION.as_bytes();
    put(&[redirection], DIRECTORY)?;
    put(
        &[redirection, REDIRECTION_ONE.as_bytes()],
        file(REDIRECTION_BODY),
    )?;
    put(
        &[redirection, REDIRECTION_TWO.as_bytes()],
        file(REDIRECTION_BODY),
    )?;

    let (rmtree, doomed, nested) = (
        RMTREE.as_bytes(),
        RM_DOOMED.as_bytes(),
        RM_NESTED.as_bytes(),
    );
    put(&[rmtree], DIRECTORY)?;
    put(&[rmtree, RM_KEEP.as_bytes()], file(RM_KEEP_BODY))?;
    put(&[rmtree, RM_SOLO.as_bytes()], file(RM_BODY))?;
    put(&[rmtree, doomed], DIRECTORY)?;
    put(&[rmtree, doomed, RM_ONE.as_bytes()], file(RM_BODY))?;
    put(&[rmtree, doomed, RM_TWO.as_bytes()], file(RM_BODY))?;
    put(&[rmtree, doomed, nested], DIRECTORY)?;
    put(&[rmtree, doomed, nested, RM_LEAF.as_bytes()], file(RM_BODY))?;

    let (globset, globdir) = (GLOBSET.as_bytes(), GLOB_DIRECTORY.as_bytes());
    put(&[globset], DIRECTORY)?;
    for name in [GLOB_ONE, GLOB_TWO, GLOB_MISS] {
        put(&[globset, name.as_bytes()], file(GLOB_BODY))?;
    }
    put(&[globset, globdir], DIRECTORY)?;
    put(&[globset, globdir, GLOB_INNER.as_bytes()], file(GLOB_BODY))?;

    let globmany = GLOBMANY.as_bytes();
    put(&[globmany], DIRECTORY)?;
    for name in MANY_NAMES {
        put(&[globmany, name.as_bytes()], file(MANY_BODY))?;
    }
    put(&[globmany, MANY_MISS.as_bytes()], file(MANY_BODY))
}

fn walk_tree<E>(put: &mut impl FnMut(&[&[u8]], Kind) -> Result<(), E>) -> Result<(), E> {
    use walk::*;
    let small = file(SMALL_BODY);
    let root = ROOT.as_bytes();
    put(&[root], DIRECTORY)?;

    // The chain: `chain`, `chain/c`, and so on, with the file `f` at every level.
    let mut path: [&[u8]; DEPTH + 3] = [b""; DEPTH + 3];
    path[0] = root;
    path[1] = CHAIN.as_bytes();
    for level in 0..=DEPTH {
        let directory = 2 + level;
        if level > 0 {
            path[directory - 1] = LEVEL.as_bytes();
        }
        put(&path[..directory], DIRECTORY)?;
        path[directory] = LEVEL_FILE.as_bytes();
        put(&path[..=directory], small)?;
    }

    put(&[root, WIDE.as_bytes()], DIRECTORY)?;
    for i in 0..WIDE_FILES {
        put(&[root, WIDE.as_bytes(), &wide_name(i)], small)?;
    }
    put(&[root, NARROW.as_bytes()], DIRECTORY)?;
    put(&[root, NARROW.as_bytes(), &wide_name(0)], small)?;
    put(&[root, BYTE_REGION.as_bytes()], DIRECTORY)?;
    for (name, len) in BYTE_REGION_FILES {
        put(
            &[root, BYTE_REGION.as_bytes(), name.as_bytes()],
            Kind::File(Body::Sized(len)),
        )?;
    }
    Ok(())
}

/// Read access to a staged image, which [`verify`] checks against the plan. A path is a list of
/// names below the image root, and the empty path is the root.
pub trait Reader {
    /// What a failed list or read returns.
    type Error;

    /// Calls `name` once for every entry in the directory at `path`, in any order, leaving out `.`
    /// and `..`.
    fn list(&mut self, path: &[&[u8]], name: &mut dyn FnMut(&[u8])) -> Result<(), Self::Error>;

    /// Reads the file at `path` from byte `offset` into `buf` and returns how many bytes it read,
    /// 0 at the end of the file.
    fn read(&mut self, path: &[&[u8]], offset: usize, buf: &mut [u8])
    -> Result<usize, Self::Error>;
}

/// The first place a staged image differs from the plan. Each `entry` is an [`Entry::index`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mismatch<E> {
    /// The reader failed on this entry, or on the root when `entry` is `None`.
    Failed {
        /// The entry the reader failed on.
        entry: Option<usize>,
        /// What the reader returned.
        error: E,
    },
    /// This directory listed a name the plan does not put there. The root may hold names the plan
    /// does not, because other tests share it, so this never names the root.
    Unexpected {
        /// The directory.
        directory: usize,
    },
    /// The entry appeared twice in its parent's listing.
    Repeated {
        /// The entry.
        entry: usize,
    },
    /// The entry was missing from its parent's listing.
    Missing {
        /// The entry.
        entry: usize,
    },
    /// The file ended after `length_bytes` bytes, before its body did.
    Short {
        /// The file.
        entry: usize,
        /// How many bytes it held.
        length_bytes: usize,
    },
    /// The file held bytes past the end of its body.
    Long {
        /// The file.
        entry: usize,
    },
    /// The file's byte at `offset` differs from its body's.
    Differs {
        /// The file.
        entry: usize,
        /// The first offset that differs.
        offset: usize,
    },
}

/// Reads a staged image back through `reader` and checks it against the plan: every directory
/// lists exactly the names the plan puts in it (the root at least those), and every file holds
/// exactly its body. Returns the first mismatch in visit order.
pub fn verify<R: Reader>(reader: &mut R) -> Result<(), Mismatch<R::Error>> {
    check_listing(reader, &[], None)?;
    for_each(|entry| match entry.kind {
        Kind::Directory => check_listing(reader, entry.path, Some(entry.index)),
        Kind::File(body) => check_file(reader, entry, body),
    })
}

fn is_child(parent: &[&[u8]], path: &[&[u8]]) -> bool {
    path.len() == parent.len() + 1 && path[..parent.len()] == *parent
}

/// The slot (position among its siblings) and index of the entry named `name` in `parent`.
fn child(parent: &[&[u8]], name: &[u8]) -> Option<(usize, usize)> {
    let mut slot = 0;
    for_each(|entry| {
        if is_child(parent, entry.path) {
            if entry.path[parent.len()] == name {
                return Err((slot, entry.index));
            }
            slot += 1;
        }
        Ok(())
    })
    .err()
}

fn check_listing<R: Reader>(
    reader: &mut R,
    path: &[&[u8]],
    directory: Option<usize>,
) -> Result<(), Mismatch<R::Error>> {
    let mut seen = [0u64; DIRECTORY_SLOTS / 64];
    let mut fault = None;
    let listed = reader.list(path, &mut |name| {
        if fault.is_some() {
            return;
        }
        match child(path, name) {
            Some((slot, entry)) => {
                let bit = 1 << (slot % 64);
                if seen[slot / 64] & bit != 0 {
                    fault = Some(Mismatch::Repeated { entry });
                }
                seen[slot / 64] |= bit;
            }
            None => {
                if let Some(directory) = directory {
                    fault = Some(Mismatch::Unexpected { directory });
                }
            }
        }
    });
    listed.map_err(|error| Mismatch::Failed {
        entry: directory,
        error,
    })?;
    if let Some(fault) = fault {
        return Err(fault);
    }
    let mut slot = 0;
    for_each(|entry| {
        if is_child(path, entry.path) {
            if seen[slot / 64] & (1 << (slot % 64)) == 0 {
                return Err(Mismatch::Missing { entry: entry.index });
            }
            slot += 1;
        }
        Ok(())
    })
}

fn check_file<R: Reader>(
    reader: &mut R,
    entry: &Entry<'_>,
    body: Body,
) -> Result<(), Mismatch<R::Error>> {
    let index = entry.index;
    let mut buf = [0u8; 4096];
    let mut offset = 0;
    loop {
        let n = reader
            .read(entry.path, offset, &mut buf)
            .map_err(|error| Mismatch::Failed {
                entry: Some(index),
                error,
            })?
            .min(buf.len());
        if n == 0 {
            break;
        }
        for (k, &got) in buf[..n].iter().enumerate() {
            let at = offset + k;
            if at >= body.len() {
                return Err(Mismatch::Long { entry: index });
            }
            if got != body.byte(at) {
                return Err(Mismatch::Differs {
                    entry: index,
                    offset: at,
                });
            }
        }
        offset += n;
    }
    if offset < body.len() {
        return Err(Mismatch::Short {
            entry: index,
            length_bytes: offset,
        });
    }
    Ok(())
}
