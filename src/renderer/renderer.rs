
//
//pub trait Renderer {
//    fn resize_viewport(&self, width: usize, height: usize);
//    fn clear(&self);
//    fn create_texture(&self, mipmaps: &Vec<Image>) -> Texture;
//    fn create_cube_texture(&self, sides: [Image; 6]) -> Texture;
//    fn create_buffer(&self, size: usize, data: &[u8]) -> Buffer;
//    fn create_input_layout(&self, buffer: &Buffer, layout: &Vec<AttributeLayout>) -> InputLayout;
//    fn render_coords(&self, matrix: &glm::Mat4);
//    fn render_skybox(&self, cubemap: &Texture, matrix: &glm::Mat4);
//    fn render_static(&self, entities: &Vec<EntityData>,
//                     decals: &Vec<Decal>,
//                     static_layout: &InputLayout
//                     decal_layout: &InputLayout,
//                     textures: Vec<Texture>,
//                     lightmaps_atlas: &Texture,
//                     settings: &RenderSettings);
//    fn render_imgui(&self, data: &imgui::DrawData);
//    fn screenshot(&self) -> Image;
//}
//
//pub trait Platform {
//    fn create_window_and_context(&self, width: usize, height: usize, title: String, monitor: usize) -> glium::Display;
//    fn create_renderer(&self) -> Renderer;
//    fn swap_buffers(&self);
//}
//
