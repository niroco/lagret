//! This mod is meant to contain the Store trait.
//! However, the only storage as of now is the S3 one.

use bytes::Bytes;

pub struct CrateFile {
    pub data: Bytes,
    pub size: usize,
}
