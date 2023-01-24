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

    fn create_texture(&self, mipmaps: &Vec<&crate::resource::image::Image>) -> Box<dyn super::renderer::Texture> {
        todo!()
    }

    fn create_cube_texture(&self, sides: [crate::resource::image::Image; 6]) -> Box<dyn super::renderer::Texture> {
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
