use serde::{Deserialize, Serialize};

static MEDIA_HEADER_MAGIC: i64 = 0x4d45444941544844;

static ARCHIVE_HEADER_VERSION: u8 = 1;
static MEDIA_HEADER_VERSION: u8 = 1;
static OBJECT_HEADER_VERSION: u8 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaHeader<'a> {
    version: u8,
    magic: i64,
    creation_time: u64,
    host: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArchiveHeader<'a> {
    version: i8,
    creation_time: u64,
    host: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectHeader<'a> {
    version: u8,
    object_length: u64,
    key: &'a str,
}
