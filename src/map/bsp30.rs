// ==== BSP FORMAT LAYOUT ====

pub const MAX_MAP_HULLS: usize = 4;

pub const MAX_MAP_MODELS: usize = 400;
pub const MAX_MAP_BRUSHES: usize = 4096;
pub const MAX_MAP_ENTITIES: usize = 1024;
pub const MAX_MAP_ENTSTRING: usize = (128 * 1024);

pub const MAX_MAP_PLANES: usize = 32767;
pub const MAX_MAP_NODES: usize = 32767; // Negative shorts are leaves
pub const MAX_MAP_CLIPNODES: usize = 32767;
pub const MAX_MAP_LEAFS: usize = 8192;
pub const MAX_MAP_VERTS: usize = 65535;
pub const MAX_MAP_FACES: usize = 65535;
pub const MAX_MAP_MARKSURFACES: usize = 65535;
pub const MAX_MAP_TEXINFO: usize = 8192;
pub const MAX_MAP_EDGES: usize = 256000;
pub const MAX_MAP_SURFEDGES: usize = 512000;
pub const MAX_MAP_TEXTURES: usize = 512;
pub const MAX_MAP_MIPTEX: usize = 0x200000;
pub const MAX_MAP_LIGHTING: usize = 0x200000;
pub const MAX_MAP_VISIBILITY: usize = 0x200000;

pub const MAX_MAP_PORTALS: usize = 65536;

pub const MAX_KEY: usize = 32;
pub const MAX_VALUE: usize = 1024;

pub enum LumpType {
    LumpEntities = 0,
    LumpPlanes = 1,
    LumpTextures = 2,
    LumpVertexes = 3,
    LumpVisibility = 4,
    LumpNodes = 5,
    LumpTexinfo = 6,
    LumpFaces = 7,
    LumpLighting = 8,
    LumpClipNodes = 9,
    LumpLeaves = 10,
    LumpMarkSurfaces = 11,
    LumpEdges = 12,
    LumpSurfaceEdges = 13,
    LumpModels = 14,
    HeaderLumps = 15,
}

pub enum ContentType {
    ContentsEmpty = -1,
    ContentsSolid = -2,
    ContentsWater = -3,
    ContentsSlime = -4,
    ContentsLava = -5,
    ContentsSky = -6,
    ContentsOrigin = -7,
    ContentsClip = -8,
    ContentsCurrent0 = -9,
    ContentsCurrent90 = -10,
    ContentsCurrent180 = -11,
    ContentsCurrent270 = -12,
    ContentsCurrentUp = -13,
    ContentsCurrentDown = -14,
    ContentsTranslucent = -15,
}

pub enum PlaneType {
    PlaneX = 0,
    PlaneY = 1,
    PlaneZ = 2,
    PlaneAnyX = 3,
    PlaneAnyY = 4,
    PlaneAnyZ = 5,
}

pub enum RenderMode {
    RenderModeNormal = 0,
    RenderModeColor = 1,
    RenderModeTexture = 2,
    RenderModeGlow = 3,
    RenderModeSolid = 4,
    RenderModeAdditive = 5,
}

pub struct Lump {
    pub offset: i32,
    pub length: i32,
}

pub struct Header {
    pub version: i32,
    pub lump: [Lump; LumpType::HeaderLumps as usize],
}

pub struct Node {
    pub plane_index: u32,
    pub child_index: [i16; 2], 
    pub lower: [i16; 3],
    pub upper: [i16; 3],
    pub first_face: u16,
    pub last_face: u16,
}

pub struct Leaf {
    pub content: i32,
    pub vis_offset: i32,
    pub lower: [i16; 3],
    pub upper: [i16; 3],
    pub first_mark_surface: u16,
    pub mark_surface_count: u16,
    pub ambient_levels: [u8; 4],
}

pub type MarkSurface = u16;

pub struct Plane {
    pub normal: glm::Vec3,
    pub dist: f32,
    pub r#type: i32,
}

pub type Vertex = glm::Vec3;

pub struct Edge {
    pub vertex_index: [u16; 2],
}

pub struct Face {
    pub plane_index: u16,
    pub plane_size: u16,
    pub first_edge_index: u32,
    pub edge_count: u16,
    pub texture_info: u16,
    pub styles: [u8; 4], // 0: Lighting styles for the face, 1: Range from 0xFF (dark) to 0x00 (bright), 2: Additional model, 3: Additional model
    pub lightmap_offset: u32,
}

pub type SurfaceEdge = i32;

pub struct TextureHeader {
    pub mip_texture_count: u32,
}

pub type MipTexOffset = i32;

pub const MAX_TEXTURE_NAME: usize = 16;
pub const MIP_LEVELS: usize = 4;

pub struct MipTex {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub offsets: [u32; MIP_LEVELS],
}

pub struct TextureInfo {
    pub s: glm::Vec3,
    pub s_shift: f32,
    pub t: glm::Vec3,
    pub t_shift: f32,
    pub mip_tex_index: u32,
    pub flags: u32,
}

pub struct Model {
    pub lower: glm::Vec3,
    pub upper: glm::Vec3,
    pub head_nodes_index: [i32; MAX_MAP_HULLS],
    pub vis_leaves: i32,
    pub first_face: i32,
    pub face_count: i32,
}

pub struct ClipNode {
    pub plane_index: i32,
    pub child_index: [i16; 2],
}
