use crate::input::r#move::PlayerMove;

pub struct Camera {
    player_move: Box<PlayerMove>,
    pub viewport_width: usize,
    pub viewport_height: usize,
    pub fov_y: usize,
}

impl Camera {

    pub fn new(player_move: Box<PlayerMove>) -> Self {
        return Camera {
            player_move,
            viewport_width: 0,
            viewport_height: 0,
            fov_y: 60,
        };
    }

    pub fn position(&self) -> glm::Vec3 {
        return self.player_move.origin;
    }

    pub fn pitch(&self) -> f32 {
        return self.player_move.angles.x;
    }

    pub fn yaw(&self) -> f32 {
        return self.player_move.angles.y;
    }

    pub fn view_vector() -> glm::Vec3 {
        todo!()
    }

    pub fn view_matrix() -> glm::Mat4 {
        todo!()
    }

    pub fn projection_matrix() -> glm::Mat4 {
        todo!()
    }

}
