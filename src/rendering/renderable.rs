#[derive(Default, Debug, Clone, Copy)]
pub struct RenderSettings {
    pub projection: glm::Mat4,
    pub pitch: f32,
    pub yaw: f32,
    pub view: glm::Mat4,
}

pub trait Renderable {

    fn render(&mut self, settings: &RenderSettings);

}
