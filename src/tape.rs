use std::path::Path;

use crate::{format, mt};

pub struct Drive {
    mt: mt::MagneticTape,
}

impl Drive {
    pub fn new(path: &Path) -> Result<Self, mt::Error> {
        Ok(Self {
            mt: mt::MagneticTape::new(path)?,
        })
    }

    pub fn load_media(&self) -> Result<Media, mt::Error> {
        self.mt.rewind()?;

        Ok(Media { drive: self })
    }

    pub fn init_media(&self) -> Result<(), mt::Error> {
        todo!("initialize media");
    }
}

pub struct Media<'a> {
    drive: &'a Drive,
}

impl Media<'_> {
    pub fn init() {}

    pub fn append_archive(&self) -> Result<Archive, mt::Error> {
        Ok(Archive::new(self.drive.load_media()?))
    }
}

impl<'a> Iterator for Media<'a> {
    type Item = Archive<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct Archive<'a> {
    media: Media<'a>,
}

impl<'a> Archive<'a> {
    pub fn new(cartridge: Media<'a>) -> Self {
        Self { media: cartridge }
    }

    pub fn write_object(&self, _object: &[u8]) -> Result<(), mt::Error> {
        todo!("write object to tape");
    }
}

impl<'a> Iterator for Archive<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct Object<'a> {
    header: &'a format::ObjectHeader<'a>,
    data: &'a [u8],
}
