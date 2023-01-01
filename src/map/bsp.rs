use bit_set::BitSet;
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
