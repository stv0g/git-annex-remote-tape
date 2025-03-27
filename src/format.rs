struct MediaHeader {
    version: u8,
    _reserved: u8,
    header_length: u16,
    creation_time: u64,
}

struct ArchiveHeader {
    version: u8,
    _reserved: u8,
    header_length: u16,
}

struct ObjectHeader {
    version: u8,
    _reserved: u8,
    header_length: u16,
    object_length: u64,
    key: str,
}
