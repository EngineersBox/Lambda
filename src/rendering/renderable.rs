pub struct RenderSettings {
    pub projection: glm::Mat4,
    pub pitch: f32,
    pub yaw: f32,
    pub view: glm::Mat4,
}

pub trait Renderable {

    fn render(settings: &RenderSettings);

}
