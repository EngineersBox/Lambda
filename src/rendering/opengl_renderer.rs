use std::io::{Result, Error, ErrorKind}; 
use glium::texture::srgb_texture2d::SrgbTexture2d;
use glium::texture::{RawImage2d, MipmapsOption};

use crate::rendering::renderer::Renderer;

 struct OpenGLRenderer {
    display: glium::Display,
 }

 impl Renderer for OpenGLRenderer {

    fn resize_viewport(&self, width: usize, height: usize) {
        todo!()
    }

    fn clear(&self) {
        todo!()
    }

    fn create_texture(&self, mipmaps: &Vec<&crate::resource::image::Image>) -> Result<Box<dyn super::renderer::Texture>> {
        if mipmaps.len() < 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "At least one image must be provided to create a texture"));
        }
        let raw = RawImage2d::from_raw_rgba_reversed(
            &mipmaps[0].data,
            (mipmaps[0].width as u32, mipmaps[0].height as u32)
        );
        let texture: SrgbTexture2d = SrgbTexture2d::with_mipmaps(&self.display,)

        todo!()
    }

    fn create_cube_texture(&self, sides: [crate::resource::image::Image; 6]) -> Result<Box<dyn super::renderer::Texture>> {
        todo!()
    }

    fn render_coords(&self, matrix: &glm::Mat4) {
        todo!()
    }

    fn render_skybox(&self, cubemap: &Box<dyn super::renderer::Texture>, matrix: &glm::Mat4) {
        todo!()
    }

    fn render_static(&self, entities: &Vec<super::renderer::EntityData>,
                     decals: &Vec<crate::map::bsp::Decal>,
                     static_layout: &glium::VertexBuffer<super::renderer::VertexWithLM>,
                     decal_layout: &glium::VertexBuffer<super::renderer::Vertex>,
                     textures: &Vec<Box<dyn super::renderer::Texture>>,
                     lightmaps_atlas: &Box<dyn super::renderer::Texture>,
                     settings: &super::renderable::RenderSettings) {
        todo!()
    }

    fn render_imgui(&self, data: &imgui::DrawData) {
        todo!()
    }

    fn provide_facade(&self) -> &dyn glium::backend::Facade {
        todo!()
    }

    fn screenshot(&self) -> crate::resource::image::Image {
        todo!()
    }

}
