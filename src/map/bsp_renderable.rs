use std::boxed::Box;
use bit_set::BitSet;

use crate::rendering::renderer::{Renderer,Texture,InputLayout,Buffer,FaceRenderInfo};
use crate::rendering::renderable::{Renderable,RenderSettings};
use crate::rendering::view::camera::Camera;
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

impl BSPRenderable {

    pub fn new(renderer: Box<dyn Renderer>, bsp: Box<BSP>, camera: Box<Camera>) -> Self {
        todo!()
    }

    fn load_textures(&self) {
        todo!()
    }

    fn load_lightmaps(&self) -> Vec<Vec<glm::Vec2>> {
        todo!()
    }

    fn load_sky_textures(&self) {
        todo!()
    }

    fn render_skybox(&self) {
        todo!()
    }

    fn render_static_geometry(&self, pos: glm::Vec3) -> Vec<FaceRenderInfo> {
        todo!()
    }

    fn render_leaf(&self, leaf_index: isize, face_render_info: &Vec<FaceRenderInfo>) {
        todo!()
    }

    fn render_bsp(&self, node: isize, vis_list: BitSet<u8>, pos: glm::Vec3, face_render_info: Vec<FaceRenderInfo>) {
        todo!()
    }

    fn build_buffers(lm_coords: &Vec<Vec<glm::Vec2>>) {
        todo!()
    }

}

impl Renderable for BSPRenderable {

    fn render(settings: &RenderSettings) {
        todo!()
    }

}
