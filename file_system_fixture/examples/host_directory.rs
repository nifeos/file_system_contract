//! Stages the test image into a host directory and reads it back.
//!
//! ```sh
//! cargo run --example host_directory -- stage DIRECTORY
//! cargo run --example host_directory -- verify DIRECTORY
//! ```
//!
//! `stage` writes the image into `DIRECTORY`, which must be empty or absent, and then verifies it.
//! `verify` only reads, so it checks an image something else staged: a filesystem under test
//! mounted at `DIRECTORY`, or a disk image a host tool wrote and the system under test mounted.

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_system_fixture::image::{self, Kind, Mismatch, Reader};

/// `root` joined with each name of `path`. Every name on the image is ASCII.
fn join(root: &Path, path: &[&[u8]]) -> PathBuf {
    let mut joined = root.to_path_buf();
    for name in path {
        joined.push(String::from_utf8_lossy(name).as_ref());
    }
    joined
}

/// Writes every directory and file of the plan below `root`, which must be empty or absent.
pub fn stage(root: &Path) -> io::Result<()> {
    fs::create_dir_all(root)?;
    if fs::read_dir(root)?.next().is_some() {
        return Err(io::Error::other(format!("{} is not empty", root.display())));
    }
    image::for_each(|entry| {
        let path = join(root, entry.path);
        match entry.kind {
            Kind::Directory => fs::create_dir(&path),
            Kind::File(body) => {
                let mut bytes = vec![0; body.len()];
                body.fill(0, &mut bytes);
                fs::write(&path, bytes)
            }
        }
    })
}

/// A host directory read through `std::fs`.
struct Host<'a>(&'a Path);

impl Reader for Host<'_> {
    type Error = io::Error;

    fn list(&mut self, path: &[&[u8]], name: &mut dyn FnMut(&[u8])) -> io::Result<()> {
        for entry in fs::read_dir(join(self.0, path))? {
            name(entry?.file_name().as_encoded_bytes());
        }
        Ok(())
    }

    fn read(&mut self, path: &[&[u8]], offset: usize, buf: &mut [u8]) -> io::Result<usize> {
        let mut file = File::open(join(self.0, path))?;
        file.seek(SeekFrom::Start(offset as u64))?;
        file.read(buf)
    }
}

/// The path of entry `index` below the image root, for a report.
fn describe(index: usize) -> String {
    let mut found = String::new();
    let _ = image::for_each(|entry| {
        if entry.index != index {
            return Ok(());
        }
        let names: Vec<_> = entry
            .path
            .iter()
            .map(|n| String::from_utf8_lossy(n))
            .collect();
        found = names.join("/");
        Err(())
    });
    found
}

/// Checks the image below `root` against the plan and says where it first differs.
pub fn verify(root: &Path) -> Result<(), String> {
    image::verify(&mut Host(root)).map_err(|mismatch| match mismatch {
        Mismatch::Failed { entry: None, error } => format!("the root: {error}"),
        Mismatch::Failed {
            entry: Some(entry),
            error,
        } => format!("{}: {error}", describe(entry)),
        Mismatch::Unexpected { directory } => {
            format!(
                "{} lists a name the image does not hold",
                describe(directory)
            )
        }
        Mismatch::Repeated { entry } => format!("{} is listed twice", describe(entry)),
        Mismatch::Missing { entry } => format!("{} is missing", describe(entry)),
        Mismatch::Short {
            entry,
            length_bytes,
        } => {
            format!("{} ends after {length_bytes} bytes", describe(entry))
        }
        Mismatch::Long { entry } => format!("{} is longer than its body", describe(entry)),
        Mismatch::Differs { entry, offset } => {
            format!("{} differs at byte {offset}", describe(entry))
        }
    })
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (stage_first, root) = match arguments.as_slice() {
        [verb, root] if verb == "stage" => (true, Path::new(root)),
        [verb, root] if verb == "verify" => (false, Path::new(root)),
        _ => {
            eprintln!("usage: host_directory stage|verify DIRECTORY");
            return ExitCode::from(2);
        }
    };
    if stage_first && let Err(error) = stage(root) {
        eprintln!("cannot stage {}: {error}", root.display());
        return ExitCode::FAILURE;
    }
    match verify(root) {
        Ok(()) => {
            println!(
                "{}: {} entries match the image",
                root.display(),
                image::ENTRIES
            );
            ExitCode::SUCCESS
        }
        Err(report) => {
            eprintln!("{}: {report}", root.display());
            ExitCode::FAILURE
        }
    }
}
