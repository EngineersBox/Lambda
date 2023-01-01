use std::io::{BufReader, Read, Seek, SeekFrom};
use std::fs::File;
use bit_set::BitSet;

use crate::map::bsp30::{self, TextureInfo};
use crate::map::wad::{Wad, MipmapTexture};
use crate::resource::image::Image;
use crate::scene::entity::Entity;

#[derive(Default, Clone)]
pub struct FaceTexCoords {
    pub tex_coords: Vec<glm::Vec2>,
    pub lightmap_coords: Vec<glm::Vec2>,
}

pub struct Decal {
    pub tex_index: u32,
    pub normal: glm::Vec3,
    pub vec: [glm::Vec3; 4],
}

pub struct Hull {
    pub clip_nodes: Vec<bsp30::ClipNode>,
    pub planes: Vec<bsp30::Plane>,
    pub first_clip_node: isize,
    pub last_clip_node: isize,
    pub clip_mins: glm::Vec3,
    pub clip_maxs: glm::Vec3,
}

pub struct Model {
    pub model: bsp30::Model,
    pub hulls: [Hull; bsp30::MAX_MAP_HULLS],
}

pub struct BSP {
    pub header: bsp30::Header,
    pub vertices: Vec<bsp30::Vertex>,
    pub edges: Vec<bsp30::Edge>,
    pub surface_edges: Vec<bsp30::SurfaceEdge>,
    pub nodes: Vec<bsp30::Node>,
    pub leaves: Vec<bsp30::Leaf>,
    pub mark_surfaces: Vec<bsp30::MarkSurface>,
    pub planes: Vec<bsp30::Plane>,
    pub faces: Vec<bsp30::Face>,
    pub clip_nodes: Vec<bsp30::ClipNode>,
    pub texture_header: bsp30::TextureHeader,
    pub mip_textures: Vec<bsp30::MipTex>,
    pub mip_texture_offsets: Vec<bsp30::MipTexOffset>,
    pub texture_infos: Vec<bsp30::TextureInfo>,
    pub face_tex_coords: Vec<FaceTexCoords>,
    pub entities: Vec<Entity>,
    pub brush_entities: Vec<usize>,
    pub special_entities: Vec<usize>,
    pub wad_files: Vec<Wad>,
    pub decal_wads: Vec<Wad>,
    pub m_decals: Vec<Decal>,
    pub vis_lists: Vec<BitSet<u8>>,
    pub m_textures: Vec<MipmapTexture>,
    pub m_lightmaps: Vec<Image>,
    pub hull_0_clip_nodes: Vec<bsp30::ClipNode>,
    pub models: Vec<Model>,
}

const WAD_DIR: String = String::from("../data/wads");

impl BSP {

    pub fn new(path: String) -> Self {
        todo!()
    }

    pub fn find_entity<'a>(&self, name: String) -> Option<&'a Entity> {
        todo!()
    }
    
    pub fn find_entities<'a>(&self, name: String) -> Vec<&'a Entity> {
        todo!()
    }

    pub fn load_skybox(&self) -> Option<[Image; 6]> {
        todo!()
    }

    pub (crate) fn load_wad_files(&mut self, wad_str: &String) {
        let wad_string: String = wad_str.replace("\\", "/");
        let mut wad_count: usize = 0;
        let mut pos: usize = 0;
        loop {
            pos += 1;
            let next: Option<usize> = wad_string[pos..].find(';');
            if next.is_none() {
                break;
            }
            let mut path: String = wad_string[pos..(next.unwrap() - pos)].to_string();
            if let Some(it) = path.rfind('/') {
                if let Some(it2) = path[0..it - 1].rfind('/') {
                    path = path[(it2 + 1)..].to_string();
                }
            }
            self.wad_files.push(Wad::new((WAD_DIR + path.as_str()).as_str()));
            wad_count += 1;
            info!(&crate::LOGGER, "Loaded {}", wad_count);
            pos = next.unwrap();
        }
        info!(&crate::LOGGER, "Loaded {} WADs", wad_count);
    }

    pub (crate) fn unload_wad_files(&mut self) {
        self.wad_files.clear();
    }

    pub (crate) fn load_textures(&mut self, reader: &mut BufReader<File>) {
        info!(&crate::LOGGER, "Loading texture WADs...");
        if let Some(world_spawn) = self.find_entity(String::from("world_spawn")) {
            if let Some(wad) = world_spawn.find_property(String::from("wad")) {
                self.load_wad_files(wad);
            }
        }
        info!(&crate::LOGGER, "Loading textures...");
        self.m_textures.resize_with(self.texture_header.mip_texture_count as usize, || MipmapTexture::new());
        let mut errors: usize = 0;
        for i in 0..self.texture_header.mip_texture_count as usize {
            let mip_tex: &bsp30::MipTex = &self.mip_textures[i];
            if mip_tex.offsets[0] == 0 {
                // External texture
                if let Some(tex) = self.load_textures_from_wads(String::from_utf8_lossy(&mip_tex.name).to_string()) {
                    self.m_textures[i] = tex;
                }  else {
                    error!(&crate::LOGGER, "Failed to load texture {} from WAD files", String::from_utf8_lossy(&mip_tex.name));
                    errors += 1;
                    continue;
                }
            } else {
                // Internal texture
                let data_size: usize = std::mem::size_of::<u8>() * (mip_tex.offsets[3] + (mip_tex.height / 8) * (mip_tex.width / 8) + 2 + 768) as usize;
                let mut img_data: Vec<u8> = vec![0; data_size];
                reader.seek(SeekFrom::Start(self.header.lump[bsp30::LumpType::LumpTextures as usize].offset as u64 + self.mip_texture_offsets[i] as u64));
                reader.read_exact(&mut img_data);
                self.m_textures[i] = Wad::create_mip_texture(&img_data);
            }
        }
        self.unload_wad_files();
        info!(&crate::LOGGER, "Loaded {} textures, {} failed", self.texture_header.mip_texture_count, errors);
        self.face_tex_coords.resize_with(self.faces.len(), Default::default);
        for i in 0..self.faces.len() {
            self.face_tex_coords[i].tex_coords.resize(self.faces[i].edge_count as usize, glm::vec2(0.0,0.0));
            let cur_tex_info: &TextureInfo = &self.texture_infos[self.faces[i].texture_info as usize];
            for j in 0..self.faces[i].edge_count as usize {
                let mut edge_index: i32 = self.surface_edges[self.faces[i].texture_info as usize + j];
                if edge_index > 0 {
                    self.face_tex_coords[i].tex_coords[j].x = (glm::dot(
                        self.vertices[self.edges[edge_index as usize].vertex_index[0] as usize],
                        cur_tex_info.s
                    ) + cur_tex_info.s_shift) / self.mip_textures[cur_tex_info.mip_tex_index as usize].width as f32;
                    self.face_tex_coords[i].tex_coords[j].y = (glm::dot(
                        self.vertices[self.edges[edge_index as usize].vertex_index[0] as usize],
                        cur_tex_info.t
                    ) + cur_tex_info.t_shift) / self.mip_textures[cur_tex_info.mip_tex_index as usize].height as f32;
                } else {
                    edge_index *= -1;
                    self.face_tex_coords[i].tex_coords[j].x = (glm::dot(
                        self.vertices[self.edges[edge_index as usize].vertex_index[1] as usize],
                        cur_tex_info.s
                    ) + cur_tex_info.s_shift) / self.mip_textures[cur_tex_info.mip_tex_index as usize].width as f32;
                    self.face_tex_coords[i].tex_coords[j].y = (glm::dot(
                        self.vertices[self.edges[edge_index as usize].vertex_index[1] as usize],
                        cur_tex_info.t
                    ) + cur_tex_info.t_shift) / self.mip_textures[cur_tex_info.mip_tex_index as usize].height as f32;
                }
            }
        }
    }

    pub (crate) fn load_texture_from_wads(&self, name: String) -> Option<MipmapTexture> {
        for &wad in &self.wad_files {
            if let Some(p_mipmap_tex) = wad.load_texture(name) {
                return Some(p_mipmap_tex);
            }
        }
        return None;
    }

    pub (crate) fn load_decal_texture(&self, name: String) -> Option<MipmapTexture> {
        todo!()
    }

    pub (crate) fn load_decals(&self) {
        todo!()
    }

    pub (crate) fn load_light_maps(&self, p_light_map_data: Vec<u8>) {
        todo!()
    }

    pub (crate) fn load_models(&self, reader: BufReader<File>) {
        todo!()
    }

    pub (crate) fn parse_entities(&self, entities_string: String) {
        todo!()
    }

    pub (crate) fn count_vis_leaves(&self, i_node: usize) -> usize {
        todo!()
    }

    pub (crate) fn decompress_vis(&self, leaf: usize, compresed_vis: Vec<u8>) -> BitSet {
        todo!()
    }
    
    pub (crate) fn find_leaf(&self, pos: glm::Vec3, node: usize) -> Option<usize> {
        todo!()
    }

}
