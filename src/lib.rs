//! # compiledfiles
//!
//! A library to get a list of all files that were used to compile the given
//! binary.
//!
//! This library currently only supports the following formats:
//!
//! * ELF files
//! * PDB files
//!
//! The following file formats are a work in progress
//!
//! * Mach-O files
//!
//! This library currently only supports files generated by the following compilers:
//!
//! * GCC
//! * LLVM
//! * MSVC
//!
//! This library currently only has been tested with the following languages:
//!
//! * C/C++
//!
//! The following languages are a work in progress
//!
//! * Rust
//! * Go
//!
//! Help is welcome for supporting any future formats.
//!
//! # Examples
//!
//! ```no_run
//! let elf_file = std::fs::File::open("path_to_binary").unwrap();
//! let files = compiledfiles::parse(elf_file).unwrap();
//! for file in files {
//!     println!("{:?}", file);
//! }
//! ```
use gimli::DwarfSections;
use object::{Object, ObjectSection};
use pdb::FallibleIterator;

use std::cmp::Ordering;
use std::io::Read;
use std::io::Seek;
use std::path::PathBuf;
use std::{borrow::Cow, path::Path};

/// Checksum of the source file's content
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub enum FileCheckSum {
    Md5([u8; 16]),
    Sha1([u8; 20]),
    Sha256([u8; 32]),
}

/// Basic information stored for each source file. Only the path is required.
#[derive(Debug, PartialEq, Eq)]
pub struct FileInfo {
    /// Recorded path to the source file
    pub path: PathBuf,

    /// Size of the source file in bytes
    pub size: Option<u64>,

    /// Last modified timestamp of the source file
    pub timestamp: Option<u64>,

    /// Checksum of the source file
    pub checksum: Option<FileCheckSum>,
}

impl PartialOrd for FileInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

/// Possible errors for attempting to list all sources
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The binary file is a valid file format, but does not contain debug
    /// symbols.
    #[error("File was missing debug symbols")]
    MissingDebugSymbols,

    /// The format of the file past is a unknown format
    #[error("File format was unrecognized")]
    UnrecognizedFileFormat,

    /// An IO error occurred
    #[error("Error occured reading input data")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// There was an error parsing the Dwarf information
    #[error("Error occured while parsing Dwarf information")]
    Dwarf {
        #[from]
        source: gimli::Error,
    },

    /// There was an error parsing an ELF or Mach-O file
    #[error("Error occured while parsing ELF or Macho-O file")]
    Object {
        #[from]
        source: object::Error,
    },

    /// There was an error parsing a PDB file
    #[error("Error occured while parsing PDB file")]
    Pdb {
        #[from]
        source: pdb::Error,
    },
}

type Result<T> = ::std::result::Result<T, Error>;

fn convert_pdb_checksum_to_checksum(pdb_checksum: pdb::FileChecksum) -> Option<FileCheckSum> {
    match pdb_checksum {
        pdb::FileChecksum::Md5(data) => {
            let mut hash: [u8; 16] = [0; 16];
            hash.copy_from_slice(data);
            Some(FileCheckSum::Md5(hash))
        }
        pdb::FileChecksum::Sha1(data) => {
            let mut hash: [u8; 20] = [0; 20];
            hash.copy_from_slice(data);
            Some(FileCheckSum::Sha1(hash))
        }
        pdb::FileChecksum::Sha256(data) => {
            let mut hash: [u8; 32] = [0; 32];
            hash.copy_from_slice(data);
            Some(FileCheckSum::Sha256(hash))
        }
        pdb::FileChecksum::None => None,
    }
}

/// Parses out the source file information from a file
///
/// # Arguments
///
/// * `source` - The source from which to read the bytes want to parse
///
/// # Example
///
/// ```no_run
/// let elf_file = std::fs::File::open("path_to_binary").unwrap();
/// let files = compiledfiles::parse(elf_file).unwrap();
/// for file in files {
///     println!("{:?}", file);
/// }
/// ```
pub fn parse<S: Read + Seek + std::fmt::Debug>(mut source: S) -> Result<Vec<FileInfo>> {
    // try parsing a PDB first
    match pdb::PDB::open(&mut source) {
        Ok(pdb) => return parse_pdb(pdb),
        Err(e) => match e {
            pdb::Error::UnrecognizedFileFormat => {
                // continue
            }
            _ => return Err(Error::Pdb { source: e }),
        },
    };

    source.rewind()?;

    // Now try elf or mach-o
    let mut contents = vec![];
    source.read_to_end(&mut contents)?;

    match object::File::parse(&contents[..]) {
        Ok(obj) => parse_object(&obj),
        Err(e) => Err(Error::Object { source: e }),
    }
}

/// Parses out the source file information from a file at a given path
///
/// # Arguments
///
/// * `path` - The path of the file to read the source info from
///
/// # Example
///
/// ```no_run
/// let elf_file = std::path::PathBuf::from("path_to_binary");
/// let files = compiledfiles::parse_path(&elf_file).unwrap();
/// for file in files {
///     println!("{:?}", file);
/// }
/// ```
pub fn parse_path<P>(path: P) -> Result<Vec<FileInfo>>
where
    P: AsRef<Path>,
{
    let file = std::fs::File::open(path)?;
    parse(file)
}

fn parse_pdb<'s, S: pdb::Source<'s> + 's>(mut pdb: pdb::PDB<'s, S>) -> Result<Vec<FileInfo>> {
    let mut files = vec![];

    let dbi = pdb.debug_information()?;
    let string_table = pdb.string_table()?;

    let mut modules = dbi.modules()?;

    while let Some(module) = modules.next()? {
        if let Some(mod_info) = pdb.module_info(&module)? {
            let line_program = mod_info.line_program()?;
            let mut mod_files = line_program.files();
            while let Some(file) = mod_files.next()? {
                let path_str = file.name.to_raw_string(&string_table)?;
                let file_checksum = file.checksum;
                let path = PathBuf::from(path_str.to_string().as_ref());
                let info = FileInfo {
                    path,
                    size: None,
                    timestamp: None,
                    checksum: convert_pdb_checksum_to_checksum(file_checksum),
                };
                files.push(info);
            }
        }
    }

    files.sort();
    files.dedup();

    Ok(files)
}

fn parse_object(file: &object::File) -> Result<Vec<FileInfo>> {
    let endianness = if file.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    if file.has_debug_symbols() {
        match file.format() {
            object::BinaryFormat::Elf => parse_elf_file(file, endianness),
            object::BinaryFormat::Coff => Err(Error::MissingDebugSymbols),
            object::BinaryFormat::MachO => parse_elf_file(file, endianness),
            object::BinaryFormat::Pe => Err(Error::MissingDebugSymbols),
            object::BinaryFormat::Wasm => unimplemented!(),
            _ => Err(Error::UnrecognizedFileFormat),
        }
    } else {
        Err(Error::MissingDebugSymbols)
    }
}

fn parse_elf_file(file: &object::File, endianness: gimli::RunTimeEndian) -> Result<Vec<FileInfo>> {
    // Load a section and return as `Cow<[u8]>`.
    let load_section = |id: gimli::SectionId| -> Result<Cow<[u8]>> {
        let data = match file.section_by_name(id.name()) {
            Some(ref section) => section.uncompressed_data().unwrap_or_default(),
            None => Default::default(),
        };
        Ok(data)
    };

    // Load all of the sections.
    let dwarf_sections = DwarfSections::load(&load_section)?;

    // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
    let borrow_section: &dyn for<'a> Fn(
        &'a Cow<[u8]>,
    ) -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
        &|section| gimli::EndianSlice::new(section, endianness);

    // Create `EndianSlice`s for all of the sections.
    let dwarf = dwarf_sections.borrow(&borrow_section);

    // Iterate over the compilation units.
    let mut iter = dwarf.units();

    let mut files = vec![];

    while let Some(header) = iter.next()? {
        let unit = dwarf.unit(header)?;

        if let Some(ref program) = unit.line_program {
            for file in program.header().file_names() {
                let dir_attr = file.directory(program.header()).unwrap();
                let dir_string = dwarf.attr_string(&unit, dir_attr)?.to_string_lossy();
                let dir_str = dir_string.as_ref();
                let mut path = PathBuf::from(dir_str);
                if path.is_relative() {
                    if let Some(ref comp_dir) = unit.comp_dir {
                        let comp_dir =
                            std::path::PathBuf::from(comp_dir.to_string_lossy().into_owned());
                        path = comp_dir.join(path);
                    }
                }
                let mut info = FileInfo {
                    path,
                    size: None,
                    timestamp: None,
                    checksum: None,
                };

                let filename_string = dwarf
                    .attr_string(&unit, file.path_name())?
                    .to_string_lossy();
                let filename_str = filename_string.as_ref();
                info.path.push(filename_str);

                if program.header().file_has_timestamp() {
                    info.timestamp = match file.timestamp() {
                        0 => None,
                        x => Some(x),
                    };
                }

                if program.header().file_has_size() {
                    info.size = match file.size() {
                        0 => None,
                        x => Some(x),
                    };
                }

                if program.header().file_has_md5() {
                    info.checksum = Some(FileCheckSum::Md5(*file.md5()));
                }

                // GCC will stick in a pseudo filename "<built-in>" for source
                // built into GCC.
                if !filename_str.starts_with('<') {
                    files.push(info);
                }
            }
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
