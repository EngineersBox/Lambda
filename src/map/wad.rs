use std::io::{self, Read, Seek, SeekFrom, BufReader};
use std::fs::{File, OpenOptions};
use std::collections::HashMap;
use byteorder::{ReadBytesExt, BigEndian};
use bitter::{BitReader, BigEndianReader};

use crate::map::bsp30;
use crate::resource::resource::Resource;
use crate::resource::image::Image;

pub const WAD2_MAGIC: [u8; 4] = [b'W', b'A', b'D', b'2'];
pub const WAD3_MAGIC: [u8; 4] = [b'W', b'A', b'D', b'3'];

#[derive(Debug)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub n_dir: i32,
    pub dir_offset: i32,
}

impl Resource for WadHeader {

    type T = BigEndian;

    fn from_reader(mut reader: &mut BufReader<impl ReadBytesExt>) -> io::Result<Self> {
        let magic: [u8; 4] = [
            reader.read_u8().unwrap(),
            reader.read_u8().unwrap(),
            reader.read_u8().unwrap(),
            reader.read_u8().unwrap(),
        ];
        let n_dir: i32 = reader.read_i32::<Self::T>().unwrap();
        let dir_offset: i32 = reader.read_i32::<Self::T>().unwrap();
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

    fn from_reader(mut reader: &mut BufReader<impl ReadBytesExt>) -> io::Result<Self> {
        let n_file_pos: i32 = reader.read_i32::<Self::T>().unwrap();
        let n_disk_size: i32 = reader.read_i32::<Self::T>().unwrap();
        let n_size: u32 = reader.read_u32::<Self::T>().unwrap();
        let r#type: u8 = reader.read_u8().unwrap();
        let mut next_bytes: [u8; 1 + 2 + bsp30::MAX_TEXTURE_NAME] = [0; 1 + 2 + bsp30::MAX_TEXTURE_NAME];
        let read_bytes: usize = reader.read(&mut next_bytes).unwrap();
        if read_bytes < 3 {
            panic!("Expected at least 3 bytes to read for compressed flag and n_dummy");
        }
        let mut bit_reader: BigEndianReader = BigEndianReader::new(&next_bytes);
        let compressed: bool = bit_reader.read_bit().unwrap();
        let n_dummy: i16 = bit_reader.read_i16().unwrap();
        let mut name: [u8; bsp30::MAX_TEXTURE_NAME] = [0; bsp30::MAX_TEXTURE_NAME];
        for i in 0..bsp30::MAX_TEXTURE_NAME {
            match bit_reader.read_u8() {
                Some(0) => break,
                Some(value) => name[i] = value,
                None => panic!("Expected a name for WadDirEntry, got none"),
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
            img: [(); bsp30::MIP_LEVELS].map(|_| Image::new()),
        };
    }

}

pub struct Wad {
    pub (crate) wad_file: BufReader<File>,
    pub (crate) dir_entries: HashMap<String, WadDirEntry>,
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
            wad_file: BufReader::new(wad_file),
            dir_entries: HashMap::new(),
        };
        wad.load_directory();
        return wad;
    }

    pub fn load_texture(&self, name: String) -> Option<MipmapTexture> {
        let raw_texture: Vec<u8> = self.get_texture(name);
        if raw_texture.is_empty() {
            return None;
        }
        return Some(Self::create_mip_texture(&raw_texture));
    }

    pub fn load_decal_texture(&self, name: String) -> Option<MipmapTexture> {
        let raw_texture: Vec<u8> = self.get_texture(name);
        if raw_texture.is_empty() {
            return None;
        }
        return Some(self.create_decal_texture(&raw_texture));
    }

    fn load_directory(&mut self) {
        let header: WadHeader = match WadHeader::from_reader(&mut self.wad_file) {
            Ok(header) => header,
            Err(error) => panic!("Unable to read WAD header: {}", error),
        };
        match header.magic {
            [b'W', b'A', b'D', b'2' | b'3'] => {},
            other => panic!("Invalid WAD magic string: {:?}", other)
        };
        // self.dir_entries.resize_with(header.n_dir as usize, Default::default);
        self.wad_file.seek(SeekFrom::Start(header.dir_offset as u64)).unwrap();
        for _ in 0..header.n_dir as usize {
            match WadDirEntry::from_reader(&mut self.wad_file) {
                Ok(entry) => self.dir_entries.insert(
                    String::from_utf8_lossy(&entry.name).to_string(),
                    entry,
                ),
                Err(error) => panic!("Unable to parse WadDirEntry {}: {}", i, error),
            };
        }
    }

    fn get_texture(&self, name: String) -> Vec<u8> {
        let option_entry: Option<&WadDirEntry> = self.dir_entries.get(&name);
        if let Some(entry) = option_entry {
            if entry.compressed {
                panic!("Cannot load compressed WAD texture {}", name);
            }
            self.wad_file.seek(SeekFrom::Start(entry.n_file_pos as u64)).unwrap();
            let mut texture_bytes: Vec<u8> = Vec::with_capacity(entry.n_size as usize);
            self.wad_file.read_exact(&mut texture_bytes);
            return texture_bytes;
        } else {
            return Vec::with_capacity(0);
        }
    }
    
    pub fn create_mip_texture(raw_texture: &Vec<u8>) -> MipmapTexture {
        let reader: BufReader<&[u8]> = BufReader::new(raw_texture.as_slice());
        let raw_mip_tex: bsp30::MipTex = bsp30::MipTex::from_reader(&mut reader).unwrap();
        let mut width: u32 = raw_mip_tex.width;
        let mut height: u32 = raw_mip_tex.height;
        let palette_offset: usize = raw_mip_tex.offsets[3] as usize + (width / 8) as usize * (height / 8) as usize + 2;
        let mip_tex: MipmapTexture = MipmapTexture::new();
        for level in 0..bsp30::MIP_LEVELS {
            let pixel_index: usize = raw_mip_tex.offsets[level] as usize;
            let img: Image = mip_tex.img[level];
            img.channels = 4;
            img.width = width as usize;
            img.height = height as usize;
            img.data.resize(width as usize * height as usize * 4, 0);
            for i in 0..(height * width) as usize {
                let palette_index: usize = (pixel_index + i) * 3;
                img.data[i * 4 + 0] = raw_texture[palette_offset + palette_index + 0];
                img.data[i * 4 + 1] = raw_texture[palette_offset + palette_index + 1];
                img.data[i * 4 + 2] = raw_texture[palette_offset + palette_index + 2];
                img.data[i * 4 + 3] = 255u8;
            }
            apply_alpha_sections(&mut mip_tex.img[level]);
            width /= 2;
            height /= 2;
        }
        return mip_tex;
    } 

    fn create_decal_texture(&self, raw_texture: &Vec<u8>) -> MipmapTexture {
        let reader: BufReader<&[u8]> = BufReader::new(raw_texture.as_slice());
        let raw_mip_tex: bsp30::MipTex = bsp30::MipTex::from_reader(&mut reader).unwrap();
        let mut width: u32 = raw_mip_tex.width;
        let mut height: u32 = raw_mip_tex.height;
        let palette_offset: usize = raw_mip_tex.offsets[3] as usize + (width / 8) as usize * (height / 8) as usize + 2;
        let mip_tex: MipmapTexture = MipmapTexture::new();
        let colour: usize = palette_offset + 255 * 3;
        for level in 0..bsp30::MIP_LEVELS {
            let pixel_index: usize = raw_mip_tex.offsets[level] as usize;
            let img: Image = mip_tex.img[level];
            img.channels = 4;
            img.width = width as usize;
            img.height = height as usize;
            img.data.resize(width as usize * height as usize * 4, 0);
            for i in 0..(height * width) as usize {
                let palette_index: usize = (pixel_index + 1) * 3; 
                img.data[i * 4 + 0] = raw_texture[colour + 0];
                img.data[i * 4 + 1] = raw_texture[colour + 1];
                img.data[i * 4 + 2] = raw_texture[colour + 2];
                img.data[i * 4 + 3] = 255 - raw_texture[palette_offset + palette_index];
            }
            apply_alpha_sections(&mut mip_tex.img[level]);
            width /= 2;
            height /= 2;
        }
        return mip_tex;
    }

}

fn apply_alpha_sections(p_tex: &mut Image) {
    let p_rgb_texture: Vec<u8> = Vec::with_capacity(p_tex.width * p_tex.height * 4);
    for i in 0..(p_tex.width * p_tex.height) {
        p_rgb_texture[i * 4 + 2] = 255;
    }
    for y in 0..p_tex.height {
        for x in 0..p_tex.width {
            let index: usize = y * p_tex.width + x;
            if !(p_tex.data[index * 4] == 0
                && p_tex.data[index * 4 + 1] == 0
                && p_tex.data[index * 4 + 2] == 255) {
                continue;
            }
            p_tex.data[index * 4 + 2] = 0;
            p_tex.data[index * 4 + 3] = 0;
            let mut count: usize = 0;
            let rgb_colour_sum: (usize, usize, usize) = (0, 0, 0);

            macro_rules! corner_pixel {
                ($pixel_index_expr:expr) => {
                    let pixel_index: usize = $pixel_index_expr;
                    if !(p_tex.data[pixel_index] == 0
                        && p_tex.data[pixel_index + 1] == 0
                        && p_tex.data[pixel_index + 2] == 255) {
                        rgb_colour_sum.0 += (p_tex.data[pixel_index + 0] as f32 * std::f32::consts::SQRT_2) as usize;
                        rgb_colour_sum.1 += (p_tex.data[pixel_index + 1] as f32 * std::f32::consts::SQRT_2) as usize;
                        rgb_colour_sum.2 += (p_tex.data[pixel_index + 2] as f32 * std::f32::consts::SQRT_2) as usize;
                        count += 1;
                    }
                }
            };

            macro_rules! absolute_pixel {
                ($pixel_index_expr:expr) => {
                    let pixel_index: usize = $pixel_index_expr;
                    if !(p_tex.data[pixel_index] == 0
                        && p_tex.data[pixel_index + 1] == 0
                        && p_tex.data[pixel_index + 2] == 255) {
                        rgb_colour_sum.0 += p_tex.data[pixel_index] as usize;
                        rgb_colour_sum.1 += p_tex.data[pixel_index + 1] as usize;
                        rgb_colour_sum.2 += p_tex.data[pixel_index + 2] as usize;
                        count += 1;
                    }
                }
            };
            // Top left
            if x > 0 && y > 0 {
                corner_pixel!(((y - 1) * p_tex.width + (x - 1)) * 4);
            }
            // Top
            if x >= 0 && y >= 0 {
                absolute_pixel!(((y - 1) * p_tex.width + x) * 4);
            }
            // Top right
            if x < p_tex.width && y > 0 {
                corner_pixel!(((y - 1) * p_tex.width + (x + 1)) * 4);
            }
            // Left
            if x > 0 {
                absolute_pixel!((y * p_tex.width + (x + 1)) * 4);
            }
            // Right
            if x < p_tex.width - 1 {
                absolute_pixel!((y * p_tex.width + (x + 1)) * 4);
            }
            // Bottom left
            if x > 0 && y < p_tex.height - 1 {
                corner_pixel!(((y + 1) * p_tex.width + (x - 1)) * 4);
            }
            // Bottom
            if x >= 0 && y < p_tex.height - 1 {
                absolute_pixel!(((y + 1) * p_tex.width + x) * 4);
            }
            // Bottom right
            if x < p_tex.width - 1 && y < p_tex.height - 1 {
                corner_pixel!(((y + 1) * p_tex.width + (x + 1)) * 4);
            }
            if count > 0 {
                rgb_colour_sum.0 /= count;
                rgb_colour_sum.1 /= count;
                rgb_colour_sum.2 /= count;

                p_rgb_texture[index * 4 + 0] = rgb_colour_sum.0 as u8;
                p_rgb_texture[index * 4 + 1] = rgb_colour_sum.1 as u8;
                p_rgb_texture[index * 4 + 2] = rgb_colour_sum.2 as u8;
            }
        }
    } 
    for y in 0..p_tex.height {
        for x in 0..p_tex.width {
            let index: usize = y * p_tex.width + x;
            if p_rgb_texture[index * 4] != 0
                || p_rgb_texture[index * 4 + 1] != 0
                || p_rgb_texture[index * 4 + 2] != 255
                || p_rgb_texture[index * 4 + 3] != 0 {
                p_tex.data[index * 4 + 0] = p_rgb_texture[index * 4 + 0];
                p_tex.data[index * 4 + 1] = p_rgb_texture[index * 4 + 1];
                p_tex.data[index * 4 + 2] = p_rgb_texture[index * 4 + 2];
                p_tex.data[index * 4 + 3] = p_rgb_texture[index * 4 + 3];
            }
        }
    }
}
