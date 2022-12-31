use std::fs::{File, OpenOptions};
use crate::map::bsp30;
use crate::resource::image::Image;

pub struct WadHeader {
    pub magic: [u8; 4],
    pub n_dir: i32,
    pub dir_offset: i32,
}

pub struct WadDirEntry {
    pub n_file_pos: i32,
    pub n_disk_size: i32,
    pub n_size: u32,
    pub r#type: u8,
    pub compressed: bool,
    pub n_dummy: i16,
    pub name: String,
}

pub struct MipmapTexture {
    pub img: [Image; bsp30::MIP_LEVELS],
}

impl MipmapTexture {

    pub fn new() -> MipmapTexture {
        return MipmapTexture {
            img: []
        };
    }

}

pub struct Wad {
    pub (crate) wad_file: File,
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
            wad_file,
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
        let header;
    }

    fn get_texture(&self, name: &str) -> Vec<u8> {

    }

    fn create_decal_texture(&self, raw_texture: &Vec<u8>) -> MipmapTexture {
        
    }

}
