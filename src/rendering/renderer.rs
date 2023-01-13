use std::boxed::Box;
use glium::vertex::Vertex;

use crate::resource::image::Image;
use crate::rendering::renderable::RenderSettings;
use crate::map::bsp30;
use crate::map::bsp::Decal;

pub trait Texture {}
pub trait Buffer {}
pub trait InputLayout {}

pub struct FaceRenderInfo {
    pub tex: Box<dyn Texture>,
    pub offset: usize,
    pub count: usize,
}

pub enum AttributeLayoutType {
    Float,
}

pub struct AttributeLayout {
    pub semantic: Vec<u8>,
    pub semantic_index: usize,
    pub size: usize,
    pub r#type: AttributeLayoutType,
    pub stride: usize,
    pub offset: usize,
}

pub struct EntityData {
    pub face_render_info: Vec<FaceRenderInfo>,
    pub origin: glm::Vec3,
    pub alpha: f32,
    pub render_mode: bsp30::RenderMode,
}

pub trait Renderer {
    fn resize_viewport(&self, width: usize, height: usize);
    fn clear(&self);
    fn create_texture(&self, mipmaps: &Vec<&Image>) -> Box<dyn Texture>;
    fn create_cube_texture(&self, sides: [Image; 6]) -> Box<dyn Texture>;
    fn create_buffer(&self, data: &[dyn Vertex]) -> Box<dyn Buffer>;
    fn create_input_layout(&self, buffer: &dyn Buffer, layout: &Vec<AttributeLayout>) -> dyn InputLayout;
    fn render_coords(&self, matrix: &glm::Mat4);
    fn render_skybox(&self, cubemap: &dyn Texture, matrix: &glm::Mat4);
    fn render_static(&self, entities: &Vec<EntityData>,
                     decals: &Vec<Decal>,
                     static_layout: &dyn InputLayout,
                     decal_layout: &dyn InputLayout,
                     textures: Vec<Box<dyn Texture>>,
                     lightmaps_atlas: &dyn Texture,
                     settings: &RenderSettings);
    fn render_imgui(&self, data: &imgui::DrawData);
    fn screenshot(&self) -> Image;
}

pub trait Platform {
    fn create_window_and_context(&self, width: usize, height: usize, title: String, monitor: usize) -> glium::Display;
    fn create_renderer() -> Box<dyn Renderer>;
    fn swap_buffers(&self);
}

