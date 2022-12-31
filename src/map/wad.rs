use std::fs::{File, OpenOptions};
use std::io::{self, Read, Cursor};
use byteorder::{ReadBytesExt, BigEndian};
use bitreader::BitReader;

use crate::map::bsp30;
use crate::resource::resource::Resource;
use crate::resource::image::Image;

pub const WAD2_MAGIC: [u8; 4] = [b'W', b'A', b'D', b'2'];
pub const WAD3_MAGIC: [u8; 4] = [b'W', b'A', b'D', b'3'];
const BOOL_BIT_MASK: u8 = 0b1000_0000;

#[derive(Debug)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub n_dir: i32,
    pub dir_offset: i32,
}

impl Resource for WadHeader {

    type T = BigEndian;

    fn from_reader(mut reader: &Cursor<impl ReadBytesExt>) -> io::Result<Self> {
        let buf = reader.get_ref();
        let magic: [u8; 4] = [
            buf.read_u8().unwrap(),
            buf.read_u8().unwrap(),
            buf.read_u8().unwrap(),
            buf.read_u8().unwrap(),
        ];
        let n_dir: i32 = buf.read_i32::<Self::T>().unwrap();
        let dir_offset: i32 = buf.read_i32::<Self::T>().unwrap();
        return Ok(WadHeader {
            magic,
            n_dir,
            dir_offset,
        });
    }

}

#[derive(Default)]
pub struct WadDirEntry {
    pub n_file_pos: i32,
    pub n_disk_size: i32,
    pub n_size: u32,
    pub r#type: u8,
    pub compressed: bool,
    pub n_dummy: i16,
    pub name: [u8; bsp30::MAX_TEXTURE_NAME],
}

impl Resource for WadDirEntry {

    type T = BigEndian;

    fn from_reader(mut reader: &Cursor<impl ReadBytesExt>) -> io::Result<Self> {
        let mut buf = reader.get_ref();
        let n_file_pos: i32 = buf.read_i32::<Self::T>().unwrap();
        let n_disk_size: i32 = buf.read_i32::<Self::T>().unwrap();
        let n_size: u32 = buf.read_u32::<Self::T>().unwrap();
        let r#type: u8 = buf.read_u8().unwrap();
        let bit_reader: BitReader = BitReader::new(&[buf.read_u8().unwrap()]);
        let temp_u8: u8 = bit_reader.read_u8(1).unwrap();
        let compressed: bool = (bit_reader.read_u8(1).unwrap() & BOOL_BIT_MASK) != 0;
        // TODO: Fix this reading as it will resume just before the single bit. Needs to start
        // after that bit
        reader.set_position(reader.position() - 1);
        buf = reader.get_ref();
        let n_dummy: i16 = buf.read_i16::<Self::T>().unwrap();
        let mut name: [u8; bsp30::MAX_TEXTURE_NAME] = [0; bsp30::MAX_TEXTURE_NAME];
        for i in 0..bsp30::MAX_TEXTURE_NAME {
            match buf.read_u8() {
                Ok(0) => break,
                Ok(value) => name[i] = value,
                Err(error) => panic!("Unable to parse WadDirEntry name: {}", error),
            };
        }
        return Ok(WadDirEntry {
            n_file_pos,
            n_disk_size,
            n_size,
            r#type,
            compressed,
            n_dummy,
            name,
        });
    }

}

pub struct MipmapTexture {
    pub img: [Image; bsp30::MIP_LEVELS],
}

impl MipmapTexture {

    pub fn new() -> MipmapTexture {
        return MipmapTexture {
            img: [Default::default(); bsp30::MIP_LEVELS]
        };
    }

}

pub struct Wad {
    pub (crate) wad_file: Cursor<File>,
    pub (crate) dir_entries: Vec<WadDirEntry>,
}

impl Wad {
    
    pub fn new(path: &str) -> Wad {
        let wad_file: File = match OpenOptions::new()
            .read(true)
            .open(&path) {
            Ok(file) => file,
            Err(error) => panic!(
                "Unable to read WAD file at {}: {}",
                path,
                error,
            ),
        };
        let mut wad: Wad = Wad {
            wad_file: Cursor::new(wad_file),
            dir_entries: Vec::new(),
        };
        wad.load_directory();
        return wad;
    }

    pub fn load_texture(&self, name: &str) -> Option<MipmapTexture> {
        let raw_texture: Vec<u8> = self.get_texture(name);
        if raw_texture.is_empty() {
            return None;
        }
        return Some(Self::create_mip_texture(&raw_texture));
    }

    pub fn load_decal_texture(&self, name: &str) -> Option<MipmapTexture> {
        let raw_texture: Vec<u8> = self.get_texture(name);
        if raw_texture.is_empty() {
            return None;
        }
        return Some(self.create_decal_texture(&raw_texture));
    }

    pub fn create_mip_texture(raw_texture: &Vec<u8>) -> MipmapTexture {
        
    }

    fn load_directory(&self) {
        let header: WadHeader = match WadHeader::from_reader(&self.wad_file) {
            Ok(header) => header,
            Err(error) => panic!("Unable to read WAD header: {}", error),
        };
        match header.magic {
            [b'W', b'A', b'D', b'2' | b'3'] => {},
            other => panic!("Invalid WAD magic string: {:?}", other)
        };
        self.dir_entries.resize_with(header.n_dir as usize, Default::default);
        self.wad_file.set_position(header.dir_offset as u64);
        for i in 0..header.n_dir as usize {
            self.dir_entries[i] = match WadDirEntry::from_reader(&self.wad_file) {
                Ok(entry) => entry,
                Err(error) => panic!("Unable to parse WadDirEntry {}: {}", i, error),
            };
        }
    }

    fn get_texture(&self, name: &str) -> Vec<u8> {

    }

    fn create_decal_texture(&self, raw_texture: &Vec<u8>) -> MipmapTexture {
        
    }

}
