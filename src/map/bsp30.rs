use std::io::{Result, Error, ErrorKind, BufReader};
use byteorder::{BigEndian, ReadBytesExt};
use crate::resource::resource::{Resource, read_char_array};

// ==== BSP FORMAT LAYOUT ====

pub const MAX_MAP_HULLS: usize = 4;

pub const MAX_MAP_MODELS: usize = 400;
pub const MAX_MAP_BRUSHES: usize = 4096;
pub const MAX_MAP_ENTITIES: usize = 1024;
pub const MAX_MAP_ENTSTRING: usize = 128 * 1024;

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

impl Resource for Lump {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let offset: i32 = reader.read_i32::<Self::T>().unwrap();
        let length: i32 = reader.read_i32::<Self::T>().unwrap();
        return Ok(Lump {
            offset,
            length,
        });
    }

}

pub struct Header {
    pub version: i32,
    pub lump: [Lump; LumpType::HeaderLumps as usize],
}

impl Resource for Header {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let version: i32 = reader.read_i32::<Self::T>().unwrap();
        let lump: Vec<Lump> = Vec::with_capacity(LumpType::HeaderLumps as usize);
        for i in 0..LumpType::HeaderLumps as usize {
            lump.push(Lump::from_reader(reader).unwrap());
        }
        return Ok(Header {
            version,
            lump: lump.try_into().ok().unwrap(),
        });
    }

}

pub struct Node {
    pub plane_index: u32,
    pub child_index: [i16; 2], 
    pub lower: [i16; 3],
    pub upper: [i16; 3],
    pub first_face: u16,
    pub last_face: u16,
}

impl Resource for Node {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let plane_index: u32 = reader.read_u32::<Self::T>().unwrap();
        let child_index: [i16; 2] = [
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
        ];
        let lower: [i16; 3] = [
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
        ];
        let upper: [i16; 3] = [
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
        ];
        let first_face: u16 = reader.read_u16::<Self::T>().unwrap();
        let last_face: u16 = reader.read_u16::<Self::T>().unwrap();
        return Ok(Node {
            plane_index,
            child_index,
            lower,
            upper,
            first_face,
            last_face,
        });
    }

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

impl Resource for Leaf {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let content: i32 = reader.read_i32::<Self::T>().unwrap();
        let vis_offset: i32 = reader.read_i32::<Self::T>().unwrap();
        let lower: [i16; 3] = [
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
        ];
        let upper: [i16; 3] = [
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
        ];
        let first_mark_surface: u16 = reader.read_u16::<Self::T>().unwrap();
        let mark_surface_count: u16 = reader.read_u16::<Self::T>().unwrap();
        let ambient_levels: [u8; 4] = [
            reader.read_u8().unwrap(),
            reader.read_u8().unwrap(),
            reader.read_u8().unwrap(),
            reader.read_u8().unwrap(),
        ];
        return Ok(Leaf {
            content,
            vis_offset,
            lower,
            upper,
            first_mark_surface,
            mark_surface_count,
            ambient_levels,
        });
    }

}

pub type MarkSurface = u16;

impl Resource for MarkSurface {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let mark_surface: MarkSurface = reader.read_u16::<Self::T>().unwrap() as MarkSurface;
        return Ok(mark_surface);
    }

}

#[derive(Copy, Clone)]
pub struct Plane {
    pub normal: glm::Vec3,
    pub dist: f32,
    pub r#type: i32,
}

impl Resource for Plane {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let normal: glm::Vec3 = glm::vec3(
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
        );
        let dist: f32 = reader.read_f32::<Self::T>().unwrap();
        let r#type: i32 = reader.read_i32::<Self::T>().unwrap();
        return Ok(Plane {
            normal,
            dist,
            r#type,
        });
    }

}

pub type Vertex = glm::Vec3;

impl Resource for Vertex {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let vertex: glm::Vec3 = glm::vec3(
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
        );
        return Ok(vertex);
    }

}

pub struct Edge {
    pub vertex_index: [u16; 2],
}

impl Resource for Edge {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let vertex_index: [u16; 2] = [
            reader.read_u16::<Self::T>().unwrap(),
            reader.read_u16::<Self::T>().unwrap(),
        ];
        return Ok(Edge {
            vertex_index,
        });
    }

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

impl Resource for Face {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let plane_index: u16 = reader.read_u16::<Self::T>().unwrap();
        let plane_size: u16 = reader.read_u16::<Self::T>().unwrap();
        let first_edge_index: u32 = reader.read_u32::<Self::T>().unwrap();
        let edge_count: u16 = reader.read_u16::<Self::T>().unwrap();
        let texture_info: u16 = reader.read_u16::<Self::T>().unwrap();
        let styles: [u8; 4] = [
            reader.read_u8().unwrap(),
            reader.read_u8().unwrap(),
            reader.read_u8().unwrap(),
            reader.read_u8().unwrap(),
        ];
        let lightmap_offset: u32 = reader.read_u32::<Self::T>().unwrap();
        return Ok(Face {
            plane_index,
            plane_size,
            first_edge_index,
            edge_count,
            texture_info,
            styles,
            lightmap_offset,
        });
    }

}

pub type SurfaceEdge = i32;

impl Resource for SurfaceEdge {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let surface_edge: SurfaceEdge = reader.read_i32::<Self::T>().unwrap() as SurfaceEdge;
        return Ok(surface_edge);
    }

}

pub struct TextureHeader {
    pub mip_texture_count: u32,
}

pub type MipTexOffset = i32;

pub const MAX_TEXTURE_NAME: usize = 16;
pub const MIP_LEVELS: usize = 4;

pub struct MipTex {
    pub name: [u8; MAX_TEXTURE_NAME],
    pub width: u32,
    pub height: u32,
    pub offsets: [u32; MIP_LEVELS],
}

impl Resource for MipTex {
   
    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let mut name: [u8; MAX_TEXTURE_NAME] = [0; MAX_TEXTURE_NAME];
        if read_char_array(reader, &mut name) == 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Expected texture name, got none"));
        }
        let width = reader.read_u32::<Self::T>().unwrap();
        let height = reader.read_u32::<Self::T>().unwrap();
        let mut offsets: [u32; MIP_LEVELS] = [0; MIP_LEVELS];
        for i in 0..MIP_LEVELS {
            match reader.read_u32::<Self::T>() {
                Ok(value) => offsets[i] = value,
                Err(error) => return Err(Error::new(ErrorKind::InvalidData, format!("Unable to read texture mip levels: {}", error))),
            }
        }
        return Ok(MipTex {
            name,
            width,
            height,
            offsets,
        });
    }

}

pub struct TextureInfo {
    pub s: glm::Vec3,
    pub s_shift: f32,
    pub t: glm::Vec3,
    pub t_shift: f32,
    pub mip_tex_index: u32,
    pub flags: u32,
}

impl Resource for TextureInfo {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let s: glm::Vec3 = glm::vec3(
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
        );
        let s_shift: f32 = reader.read_f32::<Self::T>().unwrap();
        let t: glm::Vec3 = glm::vec3(
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
        );
        let t_shift: f32 = reader.read_f32::<Self::T>().unwrap();
        let mip_tex_index: u32 = reader.read_u32::<Self::T>().unwrap();
        let flags: u32 = reader.read_u32::<Self::T>().unwrap();
        return Ok(TextureInfo {
            s,
            s_shift,
            t,
            t_shift,
            mip_tex_index,
            flags,
        });
    }

}

#[derive(Copy,Clone)]
pub struct Model {
    pub lower: glm::Vec3,
    pub upper: glm::Vec3,
    pub head_nodes_index: [i32; MAX_MAP_HULLS],
    pub vis_leaves: i32,
    pub first_face: i32,
    pub face_count: i32,
}

impl Model {

    pub fn new() -> Self {
        return Model {
            lower: glm::vec3(0.0, 0.0, 0.0),
            upper: glm::vec3(0.0, 0.0, 0.0),
            head_nodes_index: [0; MAX_MAP_HULLS],
            vis_leaves: 0,
            first_face: 0,
            face_count: 0,
        };
    }

}

impl Resource for Model {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let lower: glm::Vec3 = glm::vec3(
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
        );
        let upper: glm::Vec3 = glm::vec3(
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
            reader.read_f32::<Self::T>().unwrap(),
        );
        let mut head_nodes_index: [i32; MAX_MAP_HULLS] = [0; MAX_MAP_HULLS];
        for i in 0..MAX_MAP_HULLS {
            match reader.read_i32::<Self::T>() {
                Ok(value) => head_nodes_index[i] = value,
                Err(error) => return Err(Error::new(ErrorKind::InvalidData, format!("Unable to read model head node index: {}", error))),
            }
        }
        let vis_leaves: i32 = reader.read_i32::<Self::T>().unwrap();
        let first_face: i32 = reader.read_i32::<Self::T>().unwrap();
        let face_count: i32 = reader.read_i32::<Self::T>().unwrap();
        return Ok(Model {
            lower,
            upper,
            head_nodes_index,
            vis_leaves,
            first_face,
            face_count,
        });
    }

}

#[derive(Default,Clone,Copy)]
pub struct ClipNode {
    pub plane_index: i32,
    pub child_index: [i16; 2],
}

impl Resource for ClipNode {

    type T = BigEndian;

    fn from_reader(reader: &mut BufReader<impl byteorder::ReadBytesExt>) -> Result<Self> {
        let plane_index: i32 = reader.read_i32::<Self::T>().unwrap();
        let child_index: [i16; 2] = [
            reader.read_i16::<Self::T>().unwrap(),
            reader.read_i16::<Self::T>().unwrap(),
        ];
        return Ok(ClipNode {
            plane_index,
            child_index,
        });
    }

}
