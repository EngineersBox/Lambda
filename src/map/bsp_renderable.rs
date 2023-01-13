use std::boxed::Box;
use std::io::{Result, Error, ErrorKind};
use bit_set::BitSet;

use crate::rendering::renderer::{Renderer,Texture,InputLayout,Buffer,FaceRenderInfo};
use crate::rendering::renderable::{Renderable,RenderSettings};
use crate::rendering::view::camera::Camera;
use crate::map::bsp::{BSP,FaceTexCoords};
use crate::map::bsp30;
use crate::map::wad::MipmapTexture;
use crate::resource::image::Image;

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

pub struct TextureAtlas {
    allocated: Vec<usize>,
    pub m_image: Image,
}

impl TextureAtlas {

    pub fn new(width: usize, height: usize, channels: usize) -> Self {
        return TextureAtlas {
            allocated: Vec::new(),
            m_image: Image::from((width, height, channels)),
        };
    }

    pub fn store(&mut self, image: &Image) -> Result<glm::UVec2> {
        if image.channels != self.m_image.channels {
            return Err(Error::new(ErrorKind::InvalidData, format!("Image and atlas channel count mismatch {} != {}", image.channels, self.m_image.channels)));
        }
        let loc: Option<glm::UVec2> = self.alloc_lightmap(image.width, image.height);
        if loc.is_none() {
            return Err(Error::new(ErrorKind::InvalidData, "Atlas is full"));
        }
        let coord: glm::UVec2 = loc.unwrap();
        for y in 0..image.height {
            let src: usize = (y * image.width) * image.channels;
            let dst: usize = ((coord.y as usize + y) * self.m_image.width + coord.x as usize) * image.channels;
            for i in 0..(image.width * image.channels) {
                self.m_image.data[dst + i] = image.data[src + i];
            }
        }
        return Ok(coord);
    }

    pub fn convert_coord(&self, image: &Image, stored_pos: glm::UVec2, coord: glm::Vec2) -> glm::Vec2 {
        return (glm::vec2(stored_pos.x as f32, stored_pos.y as f32) + coord * glm::vec2(image.width as f32, image.height as f32)) / glm::vec2(self.m_image.width as f32, self.m_image.height as f32);
    }

    fn alloc_lightmap(&mut self, lm_width: usize, lm_height: usize) -> Option<glm::UVec2> {
        let mut pos: glm::UVec2 = glm::vec2(0u32, 0u32);
        let mut best: usize = self.m_image.height;
        for i in 0..(self.m_image.width - lm_width) {
            let mut best2: usize = 0;
            let mut j_result: usize = 0;
            for j in 0..lm_width {
                j_result = j;
                if self.allocated[i + j] >= best {
                    break;
                }
                if self.allocated[i + j] > best2 {
                    best2 = self.allocated[i + j];
                }
            }
            if j_result == lm_width {
                pos.x = i as u32;
                best = best2;
                pos.y = best as u32;
            }
        }
        if best + lm_height > self.m_image.height {
            return None;
        }
        for i in 0..lm_width {
            self.allocated[pos.x as usize + i] = best + lm_height;
        }
        return Some(pos);
    }

}

pub struct BSPRenderable {
    m_renderer: Box<dyn Renderer>,
    m_bsp: Box<BSP>,
    m_settings: Box<RenderSettings>,
    m_skybox_tex: Option<Box<dyn Texture>>,
    m_textures: Vec<Box<dyn Texture>>,
    m_lightmap_atlas: Box<dyn Texture>,
    m_static_geometry_vbo: Box<dyn Buffer>,
    m_decal_vbo: Box<dyn Buffer>,
    m_static_geometry_vao: Box<dyn InputLayout>,
    m_decal_vao: Box<dyn InputLayout>,
    vertex_offsets: Vec<usize>,
    faces_drawn: Vec<bool>,
}

impl BSPRenderable {

    pub fn new(renderer: Box<dyn Renderer>, bsp: Box<BSP>, camera: Box<Camera>) -> Result<Self> {
        let m_skybox_tex: Option<Box<dyn Texture>> = bsp.load_skybox()
            .map(|images: [Image; 6]| renderer.create_cube_texture(images));
        let m_textures: Vec<Box<dyn Texture>> = BSPRenderable::load_textures(&renderer, &bsp.m_textures);
        let (lm_coords, m_lightmap_atlas): (Vec<Vec<glm::Vec2>>, Box<dyn Texture>) = BSPRenderable::load_lightmaps(
            &bsp.m_lightmaps,
            bsp.faces.len(),
            &bsp.face_tex_coords,
            &renderer,
        )?;
        todo!()
    }

    fn load_textures(renderer: &Box<dyn Renderer>, bsp_m_textures: &Vec<MipmapTexture>) -> Vec<Box<dyn Texture>> {
        let mut m_textures: Vec<Box<dyn Texture>> = Vec::with_capacity(bsp_m_textures.len());
        for mip_tex in bsp_m_textures {
            m_textures.push(renderer.create_texture(&vec![
                &mip_tex.img[0],
                &mip_tex.img[4],
            ]));
        }
        return m_textures;
    }

    fn load_lightmaps(bsp_m_lightmaps: &Vec<Image>, bsp_faces_len: usize, bsp_face_tex_coords: &Vec<FaceTexCoords>, renderer: &Box<dyn Renderer>) -> Result<(Vec<Vec<glm::Vec2>>, Box<dyn Texture>)> {
        let mut atlas: TextureAtlas = TextureAtlas::new(1024, 1024, 3);
        let mut lm_positions: Vec<glm::UVec2> = Vec::with_capacity(bsp_m_lightmaps.len());
        for lm in bsp_m_lightmaps.iter() {
            if lm.width == 0 || lm.height == 0 {
                lm_positions.push(glm::vec2(0u32, 0u32));
                continue;
            }
            lm_positions.push(atlas.store(lm)?);
        }
        atlas.m_image.save("lm_atlas.pmg".to_string());
        let mut lm_coords: Vec<Vec<glm::Vec2>> = Vec::with_capacity(bsp_faces_len);
        for i in 0..lm_coords.capacity() {
            let coords: &FaceTexCoords = &bsp_face_tex_coords[i];
            let sub_coords: Vec<glm::Vec2> = coords.lightmap_coords.iter()
                .map(|coord| atlas.convert_coord(
                    &bsp_m_lightmaps[i],
                    lm_positions[i],
                    coord.clone(),
                )).collect();
            lm_coords.push(sub_coords);
        }
        let m_lightmap_atlas: Box<dyn Texture> = renderer.create_texture(&vec![&atlas.m_image]);
        return Ok((lm_coords, m_lightmap_atlas));
    }

    fn render_skybox(&self) {
        todo!()
    }

    fn render_static_geometry(&self, pos: glm::Vec3) -> Vec<FaceRenderInfo> {
        todo!()
    }

    fn render_leaf(&self, leaf_index: isize, face_render_info: &Vec<FaceRenderInfo>) {
        todo!()
    }

    fn render_bsp(&self, node: isize, vis_list: BitSet<u8>, pos: glm::Vec3, face_render_info: Vec<FaceRenderInfo>) {
        todo!()
    }

    fn build_buffers(lm_coords: &Vec<Vec<glm::Vec2>>,
                     renderer: Box<dyn Renderer>,
                     bsp_faces: &Vec<bsp30::Face>,
                     bsp_face_tex_coords: &Vec<FaceTexCoords>,
                     bsp_planes: &Vec<bsp30::Plane>,
                     bsp_surface_edges:& Vec<bsp30::SurfaceEdge>,
                     bsp_vertices: &Vec<bsp30::Vertex>,
                     bsp_edges: &Vec<bsp30::Edge>) {
        let mut vertices: Vec<VertexWithLM> = Vec::new();
        for (face_index, face) in bsp_faces.iter().enumerate() {
            let coords: &FaceTexCoords = &bsp_face_tex_coords[face_index];
            let first_index: usize = vertices.len();
            for i in 0..face.edge_count as usize {
                if i > 2 {
                    let first: VertexWithLM = vertices[i].clone();
                    let prev: VertexWithLM = vertices.last().unwrap().clone();
                    vertices.push(first);
                    vertices.push(prev);
                }
                let mut v: VertexWithLM = VertexWithLM::default();
                v.tex_coord = coords.tex_coords[i].clone();
                v.lightmap_coord = if lm_coords[face_index].is_empty() {
                    glm::vec2(0.0, 0.0)
                } else {
                    lm_coords[face_index][i].clone()
                };
                v.normal = bsp_planes[face.plane_index as usize].normal.clone();
                if face.plane_side != 0 {
                    v.normal = -v.normal;
                }
                let edge: bsp30::SurfaceEdge = bsp_surface_edges[face.first_edge_index as usize + i];
                if edge > 0 {
                    v.position = bsp_vertices[bsp_edges[edge as usize].vertex_index[0] as usize].clone();
                } else {
                    v.position = bsp_vertices[bsp_edges[-edge as usize].vertex_index[1] as usize].clone();
                }
                vertices.push(v);
            }
        }
        let m_static_geometry_vbo = renderer.create_buffer(&vertices[..],);
        todo!()
    }

}

impl Renderable for BSPRenderable {

    fn render(settings: &RenderSettings) {
        todo!()
    }

}
