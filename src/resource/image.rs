use std::io::{Result,Error,ErrorKind,Cursor};
use image::{
    DynamicImage,
    ImageError,
    io::Reader as ImageReader
};

#[derive(Clone)]
pub struct Image {
    pub channels: usize,
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Image {

    pub fn new() -> Self {
        return Self::default();
    }

    pub fn load(path: String) -> Result<Self> {
        let img: DynamicImage = match ImageReader::open(path)?.decode() {
            Ok(value) => value,
            Err(error) => return Err(Error::new(ErrorKind::InvalidData, format!("{}", error))),
        };
        return Ok(Self {
            channels: img.color().channel_count() as usize,
            width: img.width() as usize,
            height: img.height() as usize,
            data: img.into_bytes(),
        });
    }

    pub fn at(&self, x: usize, y: usize) -> &u8 {
        return &self.data[(y * self.width + x) * self.channels];
    }

    pub fn save(&self, path: String) {
        todo!()
    }

}

impl Default for Image {

    fn default() -> Self {
        return Image {
            channels: 4,
            width: 0,
            height: 0,
            data: Vec::with_capacity(0),
        };
    }

}

impl From<()> for Image {

    fn from(_: ()) -> Self {
        Self::default()
    }

}

impl From<(&Image, usize)> for Image {

    fn from((image, channels): (&Image, usize)) -> Self {
        let mut cloned: Image = image.clone();
        cloned.channels = channels;
        return cloned;
    }

}

impl From<(usize, usize, usize)> for Image {

    fn from((width, height, channels): (usize, usize, usize)) -> Self {
        return Self {
            channels,
            width,
            height,
            ..Self::default()
        };
    }

}
