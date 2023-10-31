use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Read, Seek, SeekFrom};

use crate::map::bsp30;
use crate::resource::image::Image;
use crate::resource::resource::{read_char_array, Resource};

#[derive(Debug)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub n_dir: i32,
    pub dir_offset: i32,
}

impl Resource for WadHeader {
    type T = LittleEndian;

    fn from_reader(reader: &mut BufReader<impl ReadBytesExt>) -> io::Result<Self> {
        let magic: [u8; 4] = [
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
        ];
        let n_dir: i32 = reader.read_i32::<Self::T>()?;
        let dir_offset: i32 = reader.read_i32::<Self::T>()?;
        return Ok(WadHeader {
            magic,
            n_dir,
            dir_offset,
        });
    }
}

#[derive(Debug, Default)]
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
    type T = LittleEndian;

    fn from_reader(reader: &mut BufReader<impl ReadBytesExt>) -> io::Result<Self> {
        let n_file_pos: i32 = reader.read_i32::<Self::T>()?;
        let n_disk_size: i32 = reader.read_i32::<Self::T>()?;
        let n_size: u32 = reader.read_u32::<Self::T>()?;
        let r#type: u8 = reader.read_u8()?;
        let compressed: bool = reader.read_u8()? == 1u8;
        let n_dummy: i16 = reader.read_i16::<Self::T>()?;
        let mut name: [u8; bsp30::MAX_TEXTURE_NAME] = [0; bsp30::MAX_TEXTURE_NAME];
        read_char_array(&mut name, reader)?;
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
    pub(crate) wad_file: BufReader<File>,
    pub(crate) dir_entries: HashMap<String, WadDirEntry>,
}

impl Wad {
    pub fn new(path: &String) -> Wad {
        let wad_file: File = match OpenOptions::new().read(true).open(&path) {
            Ok(file) => {
                if file.metadata().unwrap().is_dir() {
                    panic!("Cannot read WAD from path pointing to directory: {}", path);
                }
                file
            }
            Err(error) => panic!("Unable to read WAD file at {}: {}", path, error,),
        };
        let mut wad: Wad = Wad {
            wad_file: BufReader::new(wad_file),
            dir_entries: HashMap::new(),
        };
        wad.load_directory();
        return wad;
    }

    pub fn load_texture(&mut self, name: &String) -> Option<MipmapTexture> {
        let raw_texture: Vec<u8> = self.get_texture(name);
        if raw_texture.is_empty() {
            return None;
        }
        return Some(Self::create_mip_texture(&raw_texture));
    }

    pub fn load_decal_texture(&mut self, name: &String) -> Option<MipmapTexture> {
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
            [b'W', b'A', b'D', b'2' | b'3'] => {}
            other => panic!("Invalid WAD magic string: {:?}", other),
        };
        // self.dir_entries.resize_with(header.n_dir as usize, Default::default);
        self.wad_file
            .seek(SeekFrom::Start(header.dir_offset as u64))
            .unwrap();
        for i in 0..header.n_dir as usize {
            match WadDirEntry::from_reader(&mut self.wad_file) {
                Ok(entry) => self.dir_entries.insert(
                    String::from_utf8_lossy(&entry.name)
                        .trim_matches(char::from(0))
                        .to_string(),
                    entry,
                ),
                Err(error) => panic!("Unable to parse WadDirEntry {}: {}", i, error),
            };
        }
    }

    fn get_texture(&mut self, name: &String) -> Vec<u8> {
        let option_entry: Option<&WadDirEntry> = self.dir_entries.get(&name.to_uppercase());
        if let Some(entry) = option_entry {
            if entry.compressed {
                panic!("Cannot load compressed WAD texture {}", name);
            }
            self.wad_file
                .seek(SeekFrom::Start(entry.n_file_pos as u64))
                .unwrap();
            let mut texture_bytes: Vec<u8> = Vec::with_capacity(entry.n_size as usize);
            for _ in 0..entry.n_size as usize {
                texture_bytes.push(self.wad_file.read_u8().unwrap());
            }
            return texture_bytes;
        } else {
            error!(
                &crate::LOGGER,
                "No such texture found with name: {}",
                name.to_uppercase()
            );
            return Vec::with_capacity(0);
        }
    }

    pub fn create_mip_texture(raw_texture: &Vec<u8>) -> MipmapTexture {
        let mut reader: BufReader<&[u8]> = BufReader::new(raw_texture.as_slice());
        let raw_mip_tex: bsp30::MipTex = bsp30::MipTex::from_reader(&mut reader).unwrap();
        let mut width: u32 = raw_mip_tex.width;
        let mut height: u32 = raw_mip_tex.height;
        let palette_offset: usize =
            raw_mip_tex.offsets[3] as usize + (width / 8) as usize * (height / 8) as usize + 2;
        let mut mip_tex: MipmapTexture = MipmapTexture::new();
        for level in 0..bsp30::MIP_LEVELS {
            let pixel_index: usize = raw_mip_tex.offsets[level] as usize;
            let mut img: &mut Image = &mut mip_tex.img[level];
            img.channels = 4;
            img.width = width as usize;
            img.height = height as usize;
            img.data.resize(width as usize * height as usize * 4, 0);
            for i in 0..(height * width) as usize {
                let palette_index: usize = raw_texture[pixel_index + i] as usize * 3;
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
        let mut reader: BufReader<&[u8]> = BufReader::new(raw_texture.as_slice());
        let raw_mip_tex: bsp30::MipTex = bsp30::MipTex::from_reader(&mut reader).unwrap();
        let mut width: u32 = raw_mip_tex.width;
        let mut height: u32 = raw_mip_tex.height;
        let palette_offset: usize =
            raw_mip_tex.offsets[3] as usize + (width / 8) as usize * (height / 8) as usize + 2;
        let mut mip_tex: MipmapTexture = MipmapTexture::new();
        let colour: usize = palette_offset + 255 * 3;
        for level in 0..bsp30::MIP_LEVELS {
            let pixel_index: usize = raw_mip_tex.offsets[level] as usize;
            let mut img: &mut Image = &mut mip_tex.img[level];
            img.channels = 4;
            img.width = width as usize;
            img.height = height as usize;
            img.data.resize(width as usize * height as usize * 4, 0);
            for i in 0..(height * width) as usize {
                let palette_index: usize = raw_texture[pixel_index + i] as usize * 3;
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
    let mut p_rgb_texture: Vec<u8> = Vec::with_capacity(p_tex.width * p_tex.height * 4);
    for _ in 0..(p_tex.width * p_tex.height) {
        p_rgb_texture.push(0);
        p_rgb_texture.push(0);
        p_rgb_texture.push(255);
        p_rgb_texture.push(0);
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
            let mut rgb_colour_sum: (usize, usize, usize) = (0, 0, 0);

            macro_rules! corner_pixel {
                ($pixel_index_expr:expr) => {
                    let pixel_index: usize = $pixel_index_expr;
                    if !(p_tex.data[pixel_index] == 0
                        && p_tex.data[pixel_index + 1] == 0
                        && p_tex.data[pixel_index + 2] == 255)
                    {
                        rgb_colour_sum.0 += (p_tex.data[pixel_index + 0] as f32
                            * std::f32::consts::SQRT_2) as usize;
                        rgb_colour_sum.1 += (p_tex.data[pixel_index + 1] as f32
                            * std::f32::consts::SQRT_2) as usize;
                        rgb_colour_sum.2 += (p_tex.data[pixel_index + 2] as f32
                            * std::f32::consts::SQRT_2) as usize;
                        count += 1;
                    }
                };
            }

            macro_rules! absolute_pixel {
                ($pixel_index_expr:expr) => {
                    let pixel_index: usize = $pixel_index_expr;
                    if !(p_tex.data[pixel_index] == 0
                        && p_tex.data[pixel_index + 1] == 0
                        && p_tex.data[pixel_index + 2] == 255)
                    {
                        rgb_colour_sum.0 += p_tex.data[pixel_index] as usize;
                        rgb_colour_sum.1 += p_tex.data[pixel_index + 1] as usize;
                        rgb_colour_sum.2 += p_tex.data[pixel_index + 2] as usize;
                        count += 1;
                    }
                };
            }

            // Top left
            if x > 0 && y > 0 {
                corner_pixel!(((y - 1) * p_tex.width + (x - 1)) * 4);
            }
            // Top
            if x >= 0 && y > 0 {
                absolute_pixel!(((y - 1) * p_tex.width + x) * 4);
            }
            // Top right
            if x < p_tex.width - 1 && y > 0 {
                corner_pixel!(((y - 1) * p_tex.width + (x + 1)) * 4);
            }
            // Left
            if x > 0 {
                absolute_pixel!((y * p_tex.width + (x - 1)) * 4);
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
                || p_rgb_texture[index * 4 + 3] != 0
            {
                p_tex.data[index * 4 + 0] = p_rgb_texture[index * 4 + 0];
                p_tex.data[index * 4 + 1] = p_rgb_texture[index * 4 + 1];
                p_tex.data[index * 4 + 2] = p_rgb_texture[index * 4 + 2];
                p_tex.data[index * 4 + 3] = p_rgb_texture[index * 4 + 3];
            }
        }
    }
}
