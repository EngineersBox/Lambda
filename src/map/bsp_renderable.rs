use std::boxed::Box;

use crate::rendering::renderer::{Renderer,Texture,InputLayout,Buffer};
use crate::rendering::renderable::RenderSettings;
use crate::map::bsp::BSP;

pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub tex_coord: glm::Vec3,
}

pub struct VertexWithLM {
    pub lightmap_coord: glm::Vec2,
}

pub struct BSPRenderable {
    m_renderer: Box<dyn Renderer>,
    m_bsp: Box<BSP>,
    m_settings: Box<RenderSettings>,
    m_skybox_tex: Option<Box<dyn Texture>>,
    m_textures: Vec<Box<dyn Texture>>,
    m_lightmap_atlas: Box<dyn Texture>,
    m_static_geometry_vbo: Box<dyn Buffer>,
    m_decal_vbo: Box<dyn Buffer>,
    m_static_geometry_vao: Box<dyn InputLayout>,
    m_decal_vao: Box<dyn InputLayout>,
    vertex_offsets: Vec<usize>,
    faces_drawn: Vec<bool>,
}
