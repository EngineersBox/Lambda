use crate::map::bsp::Model;

pub const IN_JUMP: usize = 1 << 1;
pub const IN_FORWARD: usize = 1 << 3;
pub const IN_BACK: usize = 1 << 4;
pub const IN_MOVE_LEFT: usize = 1 << 9;
pub const IN_MOVE_RIGHT: usize = 1 << 10;

pub const FL_DUCKING: usize = 1 << 14;

pub struct UserCommand {
    pub forward_move: f32,
    pub side_mode: f32,
    pub up_move: f32,
    pub buttons: isize,
    pub framte_time: f32,
    pub view_angles: glm::Vec3,
}

pub enum MoveType {
    Walk,
    Fly,
    Noclip,
}

pub struct PlayerMove {
    pub angles: glm::Vec3,
    pub forward: glm::Vec3,
    pub right: glm::Vec3,
    pub up: glm::Vec3,
    pub origin: glm::Vec3,
    pub velocity: glm::Vec3,
    pub view_ofs: glm::Vec3,
    pub frametime: f32,
    pub on_ground: isize,
    pub water_level: isize,
    pub friction: f32,
    pub water_jump_time: f32,
    pub dead: bool,
    pub cmd: UserCommand,
    pub old_buttons: isize,
    pub move_type: MoveType,
    pub gravity: f32,
    pub flags: isize,
    pub use_hull: usize,
    pub phys_entities: Vec<Box<Model>>,
    pub ladders: Vec<Box<Model>>,
}
