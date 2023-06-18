use std::io::Result;
use std::boxed::Box;
use glium::backend::Facade;
use glium::VertexBuffer;
use glium::texture::{SrgbTexture2d, SrgbCubemap};

use crate::resource::image::Image;
use crate::rendering::renderable::RenderSettings;
use crate::map::bsp30;
use crate::map::bsp::Decal;

pub trait Texture {}
pub trait Buffer {}
pub trait InputLayout {}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Default for Vertex {

    fn default() -> Self {

        return Self {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0],
        };
    }

}

implement_vertex!(Vertex, position, normal, tex_coord);

#[derive(Clone, Copy)]
pub struct VertexWithLM {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
    pub lightmap_coord: [f32; 2],
}

impl Default for VertexWithLM {
    
    fn default() -> Self {
        return Self {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0],
            lightmap_coord: [0.0, 0.0],
        };
    }

}

implement_vertex!(VertexWithLM, position, normal, tex_coord, lightmap_coord);

pub struct FaceRenderInfo {
    pub tex: Option<SrgbTexture2d>,
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
    fn create_texture(&self, mipmaps: &Vec<&Image>) -> Result<SrgbTexture2d>;
    fn create_cube_texture(&self, sides: [Image; 6]) -> Result<SrgbCubemap>;
    //fn create_buffer(&self, data: &[T]) -> Box<dyn Buffer>;
    //fn create_input_layout(&self, buffer: &dyn Buffer, layout: &Vec<AttributeLayout>) -> dyn InputLayout;
    fn render_coords(&self, matrix: &glm::Mat4);
    fn render_skybox(&self, cubemap: &SrgbCubemap, matrix: &glm::Mat4);
    fn render_static(&self, entities: &Vec<EntityData>,
                     decals: &Vec<Decal>,
                     static_layout: &VertexBuffer<VertexWithLM>,
                     decal_layout: &VertexBuffer<Vertex>,
                     textures: &Vec<SrgbTexture2d>,
                     lightmaps_atlas: &SrgbTexture2d,
                     settings: &RenderSettings);
    fn render_imgui(&self, data: &imgui::DrawData);
    fn provide_facade(&self) -> &dyn Facade;
    fn screenshot(&self) -> Image;
}

pub trait Platform {
    fn create_window_and_context(&self, width: usize, height: usize, title: String, monitor: usize) -> glium::Display;
    fn create_renderer() -> Box<dyn Renderer>;
    fn swap_buffers(&self);
}

