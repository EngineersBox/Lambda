use bit_set::BitSet;
use glium::texture::{SrgbCubemap, SrgbTexture2d};
use glium::vertex::VertexBuffer;
use std::boxed::Box;
use std::io::{Error, ErrorKind, Result};

use crate::map::bsp::{Decal, FaceTexCoords, BSP};
use crate::map::bsp30;
use crate::map::wad::MipmapTexture;
use crate::rendering::renderable::{RenderSettings, Renderable};
use crate::rendering::renderer::{
    EntityData, FaceRenderInfo, Renderer, Texture, Vertex, VertexWithLM,
};
use crate::rendering::view::camera::Camera;
use crate::resource::image::Image;
use crate::scene::entity::Entity;

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
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Image and atlas channel count mismatch {} != {}",
                    image.channels, self.m_image.channels
                ),
            ));
        }
        let loc: Option<glm::UVec2> = self.alloc_lightmap(image.width, image.height);
        if loc.is_none() {
            return Err(Error::new(ErrorKind::InvalidData, "Atlas is full"));
        }
        let coord: glm::UVec2 = loc.unwrap();
        for y in 0..image.height {
            let src: usize = (y * image.width) * image.channels;
            let dst: usize =
                ((coord.y as usize + y) * self.m_image.width + coord.x as usize) * image.channels;
            for i in 0..(image.width * image.channels) {
                self.m_image.data[dst + i] = image.data[src + i];
            }
        }
        return Ok(coord);
    }

    pub fn convert_coord(
        &self,
        image: &Image,
        stored_pos: glm::UVec2,
        coord: glm::Vec2,
    ) -> glm::Vec2 {
        return (glm::vec2(stored_pos.x as f32, stored_pos.y as f32)
            + glm::vec2(image.width as f32, image.height as f32).component_mul(&coord))
        .component_div(&glm::vec2(
            self.m_image.width as f32,
            self.m_image.height as f32,
        ));
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
    m_camera: Box<Camera>,
    m_settings: Box<RenderSettings>,
    m_skybox_tex: Option<SrgbCubemap>,
    m_textures: Vec<SrgbTexture2d>,
    m_lightmap_atlas: SrgbTexture2d,
    m_static_geometry_vbo: VertexBuffer<VertexWithLM>,
    m_decal_vbo: VertexBuffer<Vertex>,
    vertex_offsets: Vec<usize>,
    faces_drawn: Vec<bool>,
}

impl BSPRenderable {
    pub fn new(renderer: Box<dyn Renderer>, bsp: Box<BSP>, camera: Box<Camera>) -> Result<Self> {
        let m_skybox_tex: Option<SrgbCubemap> = bsp
            .load_skybox()
            .map(|images: [Image; 6]| renderer.create_cube_texture(images).unwrap()); //FIXME:
                                                                                      //Handle this
                                                                                      //result
                                                                                      //properly
        let m_textures: Vec<SrgbTexture2d> =
            BSPRenderable::load_textures(&renderer, &bsp.m_textures);
        let (lm_coords, m_lightmap_atlas): (Vec<Vec<glm::Vec2>>, SrgbTexture2d) =
            BSPRenderable::load_lightmaps(
                &bsp.m_lightmaps,
                bsp.faces.len(),
                &bsp.face_tex_coords,
                &renderer,
            )?;
        let (m_static_geometry_vbo, m_decal_vbo): (
            VertexBuffer<VertexWithLM>,
            VertexBuffer<Vertex>,
        ) = BSPRenderable::build_buffers(
            &lm_coords,
            &renderer,
            &bsp.faces,
            &bsp.face_tex_coords,
            &bsp.planes,
            &bsp.surface_edges,
            &bsp.vertices,
            &bsp.edges,
            &bsp.m_decals,
        )?;
        let faces_drawn: Vec<bool> = Vec::with_capacity(bsp.faces.len());
        return Ok(BSPRenderable {
            m_renderer: renderer, // TODO: Change to Box<Rc<Renderer>> and create a new reference here
            m_bsp: bsp,           // TODO: Same here with Box<Rc<BSP>>
            m_camera: camera,
            m_settings: Box::new(RenderSettings::default()),
            m_skybox_tex,
            m_textures,
            m_lightmap_atlas,
            m_static_geometry_vbo,
            m_decal_vbo,
            vertex_offsets: Vec::new(),
            faces_drawn,
        });
    }

    fn load_textures(
        renderer: &Box<dyn Renderer>,
        bsp_m_textures: &Vec<MipmapTexture>,
    ) -> Vec<SrgbTexture2d> {
        let mut m_textures: Vec<SrgbTexture2d> = Vec::with_capacity(bsp_m_textures.len());
        for mip_tex in bsp_m_textures {
            m_textures.push(
                renderer
                    .create_texture(&vec![&mip_tex.img[0], &mip_tex.img[4]])
                    .unwrap(),
            ); // FIXME: Handle this result type properly
        }
        return m_textures;
    }

    fn load_lightmaps(
        bsp_m_lightmaps: &Vec<Image>,
        bsp_faces_len: usize,
        bsp_face_tex_coords: &Vec<FaceTexCoords>,
        renderer: &Box<dyn Renderer>,
    ) -> Result<(Vec<Vec<glm::Vec2>>, SrgbTexture2d)> {
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
            let sub_coords: Vec<glm::Vec2> = coords
                .lightmap_coords
                .iter()
                .map(|coord: &glm::Vec2| {
                    atlas.convert_coord(&bsp_m_lightmaps[i], lm_positions[i], coord.clone())
                })
                .collect();
            lm_coords.push(sub_coords);
        }
        let m_lightmap_atlas: SrgbTexture2d = renderer.create_texture(&vec![&atlas.m_image])?;
        return Ok((lm_coords, m_lightmap_atlas));
    }

    fn render(
        &mut self,
        render_settings: &RenderSettings,
        render_skybox: bool,
        render_static_bsp: bool,
        render_brush_entities: bool,
        render_leaf_outlines: bool,
        use_textures: bool,
    ) {
        self.m_settings = Box::new(*render_settings);
        if self.m_skybox_tex.is_some() && render_skybox {
            self.render_skybox();
        }
        let camera_pos: glm::Vec3 = self.m_camera.position();
        if render_static_bsp || render_brush_entities {
            self.faces_drawn = self
                .faces_drawn
                .iter()
                .map(|_| false)
                .collect::<Vec<bool>>();
        }
        let mut entities: Vec<EntityData> = Vec::new();
        if render_static_bsp {
            // This take is black magic. Glorious stuff.
            let mut vis_list = std::mem::take(&mut self.m_bsp.vis_lists);
            entities.push(EntityData {
                face_render_info: self.render_static_geometry(
                    camera_pos.clone(),
                    self.m_bsp.find_leaf(camera_pos, 0),
                    &mut vis_list,
                ),
                origin: glm::vec3(0.0, 0.0, 0.0),
                alpha: 1.0,
                render_mode: bsp30::RenderMode::RenderModeNormal,
            });
            self.m_bsp.vis_lists = vis_list;
        }
        if render_brush_entities {
            for i in 0..self.m_bsp.brush_entities.len() {
                let entity: &Entity = &self.m_bsp.entities[self.m_bsp.brush_entities[i]];
                let model: isize = entity.find_property(&"model".to_string()).unwrap()[1..]
                    .parse::<isize>()
                    .unwrap();
                let alpha: f32 =
                    if let Some(render_amt) = entity.find_property(&"renderamt".to_string()) {
                        render_amt.parse::<f32>().unwrap() / 255.0
                    } else {
                        1.0
                    };
                let render_mode: bsp30::RenderMode = if let Some(psz_render_mode) =
                    entity.find_property(&"rendermode".to_string())
                {
                    num::FromPrimitive::from_u64(psz_render_mode.parse::<u64>().unwrap()).unwrap()
                } else {
                    bsp30::RenderMode::RenderModeNormal
                };
                let mut face_render_infos: Vec<FaceRenderInfo> = Vec::new();
                self.render_bsp(
                    self.m_bsp.models[model as usize].model.head_nodes_index[0] as isize,
                    &mut BitSet::<u8>::default(),
                    camera_pos.clone(),
                    use_textures,
                    &mut face_render_infos,
                );
                entities.push(EntityData {
                    face_render_info: face_render_infos,
                    origin: self.m_bsp.models[model as usize].model.origin.clone(),
                    alpha,
                    render_mode,
                });
            }
        }
        self.m_renderer.render_static(
            &entities,
            &self.m_bsp.m_decals,
            &self.m_static_geometry_vbo,
            &self.m_decal_vbo,
            &self.m_textures,
            &self.m_lightmap_atlas,
            render_settings,
        );
        if render_leaf_outlines {
            // TODO: Render outlines
        }
    }

    fn render_skybox(&self) {
        const DEG_90: f32 = 90.0f32;
        let matrix: glm::Mat4 = self.m_settings.projection
            * BSPRenderable::euler_angle_xzx(
                (self.m_settings.pitch - 90.0).to_radians(),
                (-self.m_settings.yaw).to_radians(),
                DEG_90.to_radians(),
            );
        self.m_renderer
            .render_skybox(&self.m_skybox_tex.as_ref().unwrap(), &matrix);
    }

    fn euler_angle_xzx(t1: f32, t2: f32, t3: f32) -> glm::Mat4 {
        let c1: f32 = t1.cos();
        let s1: f32 = t1.sin();
        let c2: f32 = t2.cos();
        let s2: f32 = t2.sin();
        let c3: f32 = t3.cos();
        let s3: f32 = t3.sin();
        return glm::mat4(
            c2,
            c1 * s2,
            s1 * s2,
            0.0,
            -c3 * s2,
            c1 * c2 * c3 - s1 * s3,
            c1 * s3 + c2 * c3 * s1,
            0.0,
            s2 * s3,
            -c3 * s1 - c1 * c2 * s3,
            c1 * c3 - c2 * s1 * s3,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );
    }

    fn render_static_geometry(
        &mut self,
        pos: glm::Vec3,
        leaf: Option<i16>,
        bsp_vis_lists: &mut Vec<BitSet<u8>>,
    ) -> Vec<FaceRenderInfo> {
        let mut face_render_infos: Vec<FaceRenderInfo> = Vec::new();
        let mut bit_set: BitSet<u8> = BitSet::<u8>::default();
        let mut vis_list: &mut BitSet<u8> = if leaf.is_none() || bsp_vis_lists.is_empty() {
            &mut bit_set
        } else {
            &mut bsp_vis_lists[leaf.unwrap() as usize - 1]
        };
        self.render_bsp(
            0,
            &mut vis_list,
            pos,
            true, // TODO: Make this into a method parameter
            &mut face_render_infos,
        );
        return face_render_infos;
    }

    fn render_leaf(
        &mut self,
        leaf_index: isize,
        use_textures: bool,
        face_render_infos: &mut Vec<FaceRenderInfo>,
        bsp_leaves: &Vec<bsp30::Leaf>,
        bsp_mark_surfaces: &Vec<bsp30::MarkSurface>,
        bsp_faces: &Vec<bsp30::Face>,
        bsp_header: &bsp30::Header,
        bsp_texture_infos: &Vec<bsp30::TextureInfo>,
    ) {
        for i in 0..bsp_leaves[leaf_index as usize].mark_surface_count as usize {
            let face_index: usize = bsp_mark_surfaces
                [bsp_leaves[leaf_index as usize].first_mark_surface as usize + i]
                as usize;
            if self.faces_drawn[face_index] {
                continue;
            }
            self.faces_drawn[face_index] = true;
            let face: &bsp30::Face = &bsp_faces[face_index];
            if face.styles[0] == 0xFF {
                continue;
            }
            let lightmap_available: bool = (face.lightmap_offset as isize) != -1
                && bsp_header.lump[bsp30::LumpType::LumpLighting as usize].length > 0;
            let face_render_info: FaceRenderInfo = FaceRenderInfo {
                tex: if use_textures {
                    Some(bsp_texture_infos[face.texture_info as usize].mip_tex_index as usize)
                } else {
                    None
                },
                offset: self.vertex_offsets[face_index],
                count: (face.edge_count as usize - 2) * 3,
            };
            face_render_infos.push(face_render_info);
        }
    }

    fn render_bsp(
        &mut self,
        node: isize,
        vis_list: &mut BitSet<u8>,
        pos: glm::Vec3,
        use_textures: bool,
        face_render_infos: &mut Vec<FaceRenderInfo>,
    ) {
        if node == -1 {
            return;
        }
        if node < 0 {
            let leaf: isize = !node;

            if vis_list.is_empty() && !vis_list.get_ref()[leaf as usize - 1] {
                return;
            }
            // TODO: Create a macro that takes a sequence of fields to take and
            //       restore after the block/code given has completed.
            let leaves = std::mem::take(&mut self.m_bsp.leaves);
            let mark_surfaces = std::mem::take(&mut self.m_bsp.mark_surfaces);
            let faces = std::mem::take(&mut self.m_bsp.faces);
            let header = std::mem::take(&mut self.m_bsp.header);
            let texture_infos = std::mem::take(&mut self.m_bsp.texture_infos);
            // NOTE: If we are always calling render_leaf with self.m_bsp fields
            //       then remove those parameters and reference them directly from
            //       within the call via mutable self reference
            self.render_leaf(
                leaf,
                use_textures,
                face_render_infos,
                &leaves,
                &mark_surfaces,
                &faces,
                &header,
                &texture_infos,
            );
            self.m_bsp.leaves = leaves;
            self.m_bsp.mark_surfaces = mark_surfaces;
            self.m_bsp.faces = faces;
            self.m_bsp.header = header;
            self.m_bsp.texture_infos = texture_infos;
            return;
        }
        let plane: bsp30::Plane =
            self.m_bsp.planes[self.m_bsp.nodes[node as usize].plane_index as usize];
        let dist: f32 = match plane.r#type {
            v if v == bsp30::PlaneType::PlaneX as i32 => pos.x - plane.dist,
            v if v == bsp30::PlaneType::PlaneY as i32 => pos.y - plane.dist,
            v if v == bsp30::PlaneType::PlaneZ as i32 => pos.z - plane.dist,
            _ => glm::dot(&plane.normal, &pos) - plane.dist,
        };
        let child1: usize = if dist > 0.0 { 1 } else { 0 };
        let child2: usize = if dist > 0.0 { 0 } else { 1 };
        self.render_bsp(
            self.m_bsp.nodes[node as usize].child_index[child1] as isize,
            vis_list,
            pos,
            use_textures,
            face_render_infos,
        );
        self.render_bsp(
            self.m_bsp.nodes[node as usize].child_index[child2] as isize,
            vis_list,
            pos,
            use_textures,
            face_render_infos,
        );
    }

    fn build_buffers(
        lm_coords: &Vec<Vec<glm::Vec2>>,
        renderer: &Box<dyn Renderer>,
        bsp_faces: &Vec<bsp30::Face>,
        bsp_face_tex_coords: &Vec<FaceTexCoords>,
        bsp_planes: &Vec<bsp30::Plane>,
        bsp_surface_edges: &Vec<bsp30::SurfaceEdge>,
        bsp_vertices: &Vec<bsp30::Vertex>,
        bsp_edges: &Vec<bsp30::Edge>,
        bsp_decals: &Vec<Decal>,
    ) -> Result<(VertexBuffer<VertexWithLM>, VertexBuffer<Vertex>)> {
        let mut static_vertices: Vec<VertexWithLM> = Vec::new();
        for (face_index, face) in bsp_faces.iter().enumerate() {
            let coords: &FaceTexCoords = &bsp_face_tex_coords[face_index];
            for i in 0..face.edge_count as usize {
                if i > 2 {
                    let first: VertexWithLM = static_vertices[i].clone();
                    let prev: VertexWithLM = static_vertices.last().unwrap().clone();
                    static_vertices.push(first);
                    static_vertices.push(prev);
                }
                let mut v: VertexWithLM = VertexWithLM::default();
                v.tex_coord = coords.tex_coords[i].clone().into();
                v.lightmap_coord = if lm_coords[face_index].is_empty() {
                    [0.0, 0.0]
                } else {
                    lm_coords[face_index][i].clone().into()
                };
                v.normal = bsp_planes[face.plane_index as usize].normal.clone().into();
                if face.plane_side != 0 {
                    v.normal = [-v.normal[0], -v.normal[1], -v.normal[2]];
                }
                let edge: bsp30::SurfaceEdge =
                    bsp_surface_edges[face.first_edge_index as usize + i];
                if edge > 0 {
                    v.position = bsp_vertices[bsp_edges[edge as usize].vertex_index[0] as usize]
                        .clone()
                        .into();
                } else {
                    v.position = bsp_vertices[bsp_edges[-edge as usize].vertex_index[1] as usize]
                        .clone()
                        .into();
                }
                static_vertices.push(v);
            }
        }
        let m_static_geometry_vbo: VertexBuffer<VertexWithLM> =
            match VertexBuffer::new(renderer.provide_facade(), &static_vertices[..]) {
                Ok(buf) => buf,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot create static and brush geometry: {}", error),
                    ))
                }
            };
        let mut decal_vertices: Vec<Vertex> = Vec::new();
        for decal in bsp_decals.iter() {
            for i in 0..6 {
                let mut vertex: Vertex = Vertex::default();
                vertex.normal = decal.normal.clone().into();
                if i == 0 || i == 3 {
                    vertex.position = decal.vec[0].clone().into();
                    vertex.tex_coord = [0.0, 0.0];
                } else if i == 1 {
                    vertex.position = decal.vec[1].clone().into();
                    vertex.tex_coord = [1.0, 0.0];
                } else if i == 2 || i == 4 {
                    vertex.position = decal.vec[2].clone().into();
                    vertex.tex_coord = [1.0, 1.0];
                } else if i == 5 {
                    vertex.position = decal.vec[3].clone().into();
                    vertex.tex_coord = [0.0, 1.0];
                }
                decal_vertices.push(vertex);
            }
        }
        let m_decal_vbo: VertexBuffer<Vertex> =
            match VertexBuffer::new(renderer.provide_facade(), &decal_vertices[..]) {
                Ok(buf) => buf,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot create decal VBO: {}", error),
                    ))
                }
            };
        return Ok((m_static_geometry_vbo, m_decal_vbo));
    }
}

impl Renderable for BSPRenderable {
    fn render(settings: &RenderSettings) {
        todo!()
    }
}
