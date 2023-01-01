use std::io::BufReader;
use std::fs::File;
use bit_set::BitSet;
use slog::

use crate::map::bsp30;
use crate::map::wad::{Wad, MipmapTexture};
use crate::resource::image::Image;
use crate::scene::entity::Entity;

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

    pub fn find_entity<'a>(&self, name: String) -> &'a Entity {
        todo!()
    }
    
    pub fn find_entities<'a>(&self, name: String) -> Vec<&'a Entity> {
        todo!()
    }

    pub fn load_skybox(&self) -> Option<[Image; 6]> {
        todo!()
    }

    pub (crate) fn load_wad_files(&self, wad_str: String) {
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
            info!(&crate::LOGGER, "Loaded WAD {}", wad_count);
        }
        todo!()
    }

    pub (crate) fn unload_wad_files() {
        todo!()
    }

    pub (crate) fn load_textures(reader: BufReader<File>) {
        todo!()
    }

    pub (crate) fn load_textures_from_wads(name: String) -> Option<MipmapTexture> {
        todo!()
    }

    pub (crate) fn load_decal_texture(name: String) -> Option<MipmapTexture> {
        todo!()
    }

    pub (crate) fn load_decals() {
        todo!()
    }

    pub (crate) fn load_light_maps(p_light_map_data: Vec<u8>) {
        todo!()
    }

    pub (crate) fn load_models(reader: BufReader<File>) {
        todo!()
    }

    pub (crate) fn parse_entities(entities_string: String) {
        todo!()
    }

    pub (crate) fn count_vis_leaves(i_node: usize) -> usize {
        todo!()
    }

    pub (crate) fn decompress_vis(leaf: usize, compresed_vis: Vec<u8>) -> BitSet {
        todo!()
    }
    
    pub (crate) fn find_leaf(pos: glm::Vec3, node: usize) -> Optional<usize> {

        todo!()
    }

}
