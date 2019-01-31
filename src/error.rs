//             DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyright (C) 2018 Thomas Bailleux <thomas@bailleux.me>
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.
//
// Author: zadig <thomas chr(0x40) bailleux.me>

use std;

/// Errors related to the process of parsing.
#[derive(Debug)]
pub enum Error {
    /// This happens when filesize is null, or to big to fit into an usize.
    BadFileSize,

    /// Classic std::io::Error.
    IOError(std::io::Error),

    /// Something is not implemented yet ?
    NotImplementedYet,

    /// This is not a valid OLE file.
    InvalidOLEFile,

    /// Something has a bad size.
    BadSizeValue(&'static str),

    /// MSAT is empty.
    EmptyMasterSectorAllocationTable,

    /// Malformed SAT.
    NotSectorUsedBySAT,

    /// Unknown node type.
    NodeTypeUnknown,

    /// Root storage has a bad size.
    BadRootStorageSize,

    /// User query an empty entry
    EmptyEntry,
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BadFileSize => "Filesize is null or too big.",
            Error::IOError(ref e) => e.description(),
            Error::NotImplementedYet => "Method not implemented yet",
            Error::InvalidOLEFile => "Invalid OLE File",
            Error::BadSizeValue(ref e) => e,
            Error::EmptyMasterSectorAllocationTable => "MSAT is empty",
            Error::NotSectorUsedBySAT => "Sector is not a sector used by the SAT.",
            Error::NodeTypeUnknown => "Unknown node type",
            Error::BadRootStorageSize => "Bad RootStorage size",
            Error::EmptyEntry => "Empty entry"
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::IOError(ref e) => Some(e),
            _ => None
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}
