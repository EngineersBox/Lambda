use std::collections::HashMap;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::fs::File;
use bit_set::BitSet;
use lazy_static::lazy_static;

use crate::map::bsp30::{self, TextureInfo};
use crate::map::wad::{Wad, MipmapTexture};
use crate::resource::image::Image;
use crate::resource::resource::Resource;
use crate::scene::entity::Entity;
use crate::util::mathutil::{point_in_plane, point_in_box};

#[derive(Default, Clone)]
pub struct FaceTexCoords {
    pub tex_coords: Vec<glm::Vec2>,
    pub lightmap_coords: Vec<glm::Vec2>,
}

pub struct Decal {
    pub tex_index: u32,
    pub normal: glm::Vec3,
    pub vec: [glm::Vec3; 4],
}

pub struct Hull {
    pub clip_nodes: Vec<bsp30::ClipNode>,
    pub planes: Vec<bsp30::Plane>,
    pub first_clip_node: isize,
    pub last_clip_node: isize,
    pub clip_mins: glm::Vec3,
    pub clip_maxs: glm::Vec3,
}

impl Hull {

    pub fn new() -> Self {
        return Hull {
            clip_nodes: Vec::with_capacity(0),
            planes: Vec::with_capacity(0),
            first_clip_node: 0,
            last_clip_node: 0,
            clip_mins: glm::vec3(0.0, 0.0, 0.0),
            clip_maxs: glm::vec3(0.0, 0.0, 0.0),
        };
    }

}

impl Clone for Hull {

    fn clone(&self) -> Self {
        return Hull {
            clip_nodes: self.clip_nodes.iter()
                .map(|cn: &bsp30::ClipNode| cn.clone())
                .collect(),
            planes: self.planes.iter()
                .map(|plane: &bsp30::Plane| plane.clone())
                .collect(),
            first_clip_node: self.first_clip_node,
            last_clip_node: self.last_clip_node,
            clip_mins: self.clip_mins.clone(),
            clip_maxs: self.clip_maxs.clone(),
        };
    }

}
#[derive(Clone)]
pub struct Model {
    pub model: bsp30::Model,
    pub hulls: [Hull; bsp30::MAX_MAP_HULLS],
}

impl Model {

    pub fn new() -> Self {
        return Model {
            model: bsp30::Model::new(),
            hulls: (0..bsp30::MAX_MAP_HULLS).map(|_| Hull::new())
                .collect::<Vec<_>>()
                .try_into()
                .ok()
                .unwrap(),
        };
    }

}

pub struct BSP {
    pub header: bsp30::Header,
    pub vertices: Vec<bsp30::Vertex>,
    pub edges: Vec<bsp30::Edge>,
    pub surface_edges: Vec<bsp30::SurfaceEdge>,
    pub nodes: Vec<bsp30::Node>,
    pub leaves: Vec<bsp30::Leaf>,
    pub mark_surfaces: Vec<bsp30::MarkSurface>,
    pub planes: Vec<bsp30::Plane>,
    pub faces: Vec<bsp30::Face>,
    pub clip_nodes: Vec<bsp30::ClipNode>,
    pub texture_header: bsp30::TextureHeader,
    pub mip_textures: Vec<bsp30::MipTex>,
    pub mip_texture_offsets: Vec<bsp30::MipTexOffset>,
    pub texture_infos: Vec<bsp30::TextureInfo>,
    pub face_tex_coords: Vec<FaceTexCoords>,
    pub entities: Vec<Entity>,
    pub brush_entities: Vec<usize>,
    pub special_entities: Vec<usize>,
    pub wad_files: Vec<Wad>,
    pub decal_wads: Vec<Wad>,
    pub m_decals: Vec<Decal>,
    pub vis_lists: Vec<BitSet<u8>>,
    pub m_textures: Vec<MipmapTexture>,
    pub m_lightmaps: Vec<Image>,
    pub hull_0_clip_nodes: Vec<bsp30::ClipNode>,
    pub models: Vec<Model>,
}

lazy_static!{
    static ref WAD_DIR: String = String::from("../data/wads");
}

impl BSP {

    pub fn new(path: String) -> Self {
        todo!()
    }

    pub fn find_entity<'a>(&self, name: &String) -> Option<&'a Entity> {
        todo!()
    }
    
    pub fn find_entities<'a>(&self, name: &String) -> Vec<&'a Entity> {
        todo!()
    }

    pub fn load_skybox(&self) -> Option<[Image; 6]> {
        todo!()
    }

    pub (crate) fn load_wad_files(&mut self, wad_str: &String) {
        let wad_string: String = wad_str.replace("\\", "/");
        let mut wad_count: usize = 0;
        let mut pos: usize = 0;
        loop {
            pos += 1;
            let next: Option<usize> = wad_string[pos..].find(';');
            if next.is_none() {
                break;
            }
            let mut path: String = wad_string[pos..(next.unwrap() - pos)].to_string();
            if let Some(it) = path.rfind('/') {
                if let Some(it2) = path[0..it - 1].rfind('/') {
                    path = path[(it2 + 1)..].to_string();
                }
            }
            self.wad_files.push(Wad::new((WAD_DIR.clone() + path.as_str()).as_str()));
            wad_count += 1;
            info!(&crate::LOGGER, "Loaded {}", wad_count);
            pos = next.unwrap();
        }
        info!(&crate::LOGGER, "Loaded {} WADs", wad_count);
    }

    pub (crate) fn unload_wad_files(&mut self) {
        self.wad_files.clear();
    }

    pub (crate) fn load_textures(&mut self, reader: &mut BufReader<File>) {
        info!(&crate::LOGGER, "Loading texture WADs...");
        if let Some(world_spawn) = self.find_entity(&"world_spawn".to_string()) {
            if let Some(wad) = world_spawn.find_property(String::from("wad")) {
                self.load_wad_files(wad);
            }
        }
        info!(&crate::LOGGER, "Loading textures...");
        self.m_textures.resize_with(self.texture_header.mip_texture_count as usize, || MipmapTexture::new());
        let mut errors: usize = 0;
        for i in 0..self.texture_header.mip_texture_count as usize {
            let mip_tex: &bsp30::MipTex = &self.mip_textures[i];
            if mip_tex.offsets[0] == 0 {
                // External texture
                if let Some(tex) = self.load_texture_from_wads(&String::from_utf8_lossy(&mip_tex.name).to_string()) {
                    self.m_textures[i] = tex;
                }  else {
                    error!(&crate::LOGGER, "Failed to load texture {} from WAD files", String::from_utf8_lossy(&mip_tex.name));
                    errors += 1;
                    continue;
                }
            } else {
                // Internal texture
                let data_size: usize = std::mem::size_of::<u8>() * (mip_tex.offsets[3] + (mip_tex.height / 8) * (mip_tex.width / 8) + 2 + 768) as usize;
                let mut img_data: Vec<u8> = vec![0; data_size];
                reader.seek(SeekFrom::Start(self.header.lump[bsp30::LumpType::LumpTextures as usize].offset as u64 + self.mip_texture_offsets[i] as u64));
                reader.read_exact(&mut img_data);
                self.m_textures[i] = Wad::create_mip_texture(&img_data);
            }
        }
        self.unload_wad_files();
        info!(&crate::LOGGER, "Loaded {} textures, {} failed", self.texture_header.mip_texture_count, errors);
        self.face_tex_coords.resize_with(self.faces.len(), Default::default);
        for i in 0..self.faces.len() {
            self.face_tex_coords[i].tex_coords.resize(self.faces[i].edge_count as usize, glm::vec2(0.0,0.0));
            let cur_tex_info: &TextureInfo = &self.texture_infos[self.faces[i].texture_info as usize];
            for j in 0..self.faces[i].edge_count as usize {
                let mut edge_index: i32 = self.surface_edges[self.faces[i].texture_info as usize + j];
                if edge_index > 0 {
                    self.face_tex_coords[i].tex_coords[j].x = (glm::dot(
                        self.vertices[self.edges[edge_index as usize].vertex_index[0] as usize],
                        cur_tex_info.s
                    ) + cur_tex_info.s_shift) / self.mip_textures[cur_tex_info.mip_tex_index as usize].width as f32;
                    self.face_tex_coords[i].tex_coords[j].y = (glm::dot(
                        self.vertices[self.edges[edge_index as usize].vertex_index[0] as usize],
                        cur_tex_info.t
                    ) + cur_tex_info.t_shift) / self.mip_textures[cur_tex_info.mip_tex_index as usize].height as f32;
                } else {
                    edge_index *= -1;
                    self.face_tex_coords[i].tex_coords[j].x = (glm::dot(
                        self.vertices[self.edges[edge_index as usize].vertex_index[1] as usize],
                        cur_tex_info.s
                    ) + cur_tex_info.s_shift) / self.mip_textures[cur_tex_info.mip_tex_index as usize].width as f32;
                    self.face_tex_coords[i].tex_coords[j].y = (glm::dot(
                        self.vertices[self.edges[edge_index as usize].vertex_index[1] as usize],
                        cur_tex_info.t
                    ) + cur_tex_info.t_shift) / self.mip_textures[cur_tex_info.mip_tex_index as usize].height as f32;
                }
            }
        }
    }

    pub (crate) fn load_texture_from_wads(&self, name: &String) -> Option<MipmapTexture> {
        for wad in self.wad_files.iter() {
            if let Some(p_mipmap_tex) = wad.load_texture(name) {
                return Some(p_mipmap_tex);
            }
        }
        return None;
    }

    pub (crate) fn load_decal_texture(&self, name: &String) -> Option<MipmapTexture> {
        for decal_wad in self.decal_wads.iter() {
            if let Some(p_mipmap_tex) = decal_wad.load_texture(name) {
                return Some(p_mipmap_tex);
            }
        }
        return None;
    }

    pub (crate) fn load_decals(&mut self) {
        self.decal_wads.push(Wad::new((WAD_DIR.clone() + "valve/decals.wad").as_str()));
        self.decal_wads.push(Wad::new((WAD_DIR.clone() + "cstrike/decals.wad").as_str()));
        let info_decals: Vec<&Entity> = self.find_entities(&"infodecal".to_string());
        if info_decals.is_empty() {
            info!(&crate::LOGGER, "No decals to load, skipping");
            return;
        }
        let mut loaded_tex: HashMap<String, usize> = HashMap::new();
        for info_decal in info_decals.iter() {
            let origin_str: Option<&String> = info_decal.find_property("origin".to_string());
            if origin_str.is_none() {
                continue;
            }
            let split_origin: Vec<&str> = origin_str.unwrap().split(" ").collect();
            if split_origin.len() != 3 {
                error!(&crate::LOGGER, "Expected 3D origin, got {}D, skipping", split_origin.len());
                continue;
            }
            let origin: glm::Vec3 = glm::vec3(
                split_origin[0].parse::<f32>().unwrap(),
                split_origin[1].parse::<f32>().unwrap(),
                split_origin[2].parse::<f32>().unwrap(),
            );
            let leaf = self.find_leaf(origin, 0);
            if leaf.is_none() {
                error!(&crate::LOGGER, "Cannot find decal leaf, skipping");
                continue;
            }
            let current_leaf = self.leaves.get(leaf.unwrap());
            if current_leaf.is_none() {
                error!(&crate::LOGGER, "Cannot find leaf, skipping");
                continue;
            }
            for j in 0..current_leaf.unwrap().mark_surface_count as usize {
                let face: &bsp30::Face = &self.faces[self.mark_surfaces[current_leaf.unwrap().first_mark_surface as usize + j] as usize];
                let normal: glm::Vec3 = self.planes[face.plane_index as usize].normal;
                let vertex: glm::Vec3;
                let edge_index: i32 = self.surface_edges[face.first_edge_index as usize];
                if edge_index > 0 {
                    vertex = self.vertices[self.edges[edge_index as usize].vertex_index[0] as usize];
                } else {
                    vertex = self.vertices[self.edges[(-edge_index) as usize].vertex_index[1] as usize];
                }
                if !point_in_plane(origin, normal, glm::dot(normal, vertex)) {
                    continue;
                }
                let tex_name: Option<&String> = info_decal.find_property("texture".to_string());
                if tex_name.is_none() {
                    error!(&crate::LOGGER, "Unable to retrieve texture name from decal");
                    break;
                }
                let it: Option<&usize> = loaded_tex.get(tex_name.unwrap());
                let mut it_val: usize = 0;
                if it.is_none() {
                    if self.load_decal_texture(&tex_name.unwrap()).is_none() {
                        error!(&crate::LOGGER, "Unable to load mipmap texture for {}", &tex_name.unwrap());
                        break;
                    }
                    it_val = self.m_textures.len();
                    loaded_tex.insert(tex_name.unwrap().clone(), self.m_textures.len());
                    self.m_textures.push(self.load_decal_texture(&tex_name.unwrap()).unwrap());
                }
                let img_0: &Image = &self.m_textures[it_val].img[0];
                let h2: f32 = img_0.height as f32 / 2.0;
                let w2: f32 = img_0.width as f32 / 2.0;
                let s: glm::Vec3 = self.texture_infos[face.texture_info as usize].s;
                let t: glm::Vec3 = self.texture_infos[face.texture_info as usize].t;
                self.m_decals.push(Decal {
                    normal,
                    tex_index: it_val as u32,
                    vec: [
                        origin - t * h2 - s * w2,
                        origin - t * h2 + s * w2,
                        origin + t * h2 + s * w2,
                        origin + t * h2 - s * w2,
                    ],
                });
                break;
            }
        }
        info!(&crate::LOGGER, "Loaded {} decals, {} decal textures", self.m_decals.len(), loaded_tex.len());
    }

    pub (crate) fn load_light_maps(&mut self, p_light_map_data: Vec<u8>) {
        let mut loaded_bytes: usize = 0;
        let mut loaded_lightmaps: usize = 0;
        for i in 0..self.faces.len() {
            if self.faces[i].styles[0] != 0 || (self.faces[i].lightmap_offset as isize) < -1 {
                self.m_lightmaps.push(Image::new());
                continue;
            }
            self.face_tex_coords[i].lightmap_coords.resize(self.faces[i].edge_count as usize, glm::vec2(0.0, 0.0));
            // Start QRAD
            let mut f_min_u: f32 = 999999.0;
            let mut f_min_v: f32 = 999999.0;
            let mut f_max_u: f32 = -99999.0;
            let mut f_max_v: f32 = -99999.0;
            let tex_info: &TextureInfo = &self.texture_infos[self.faces[i].texture_info as usize];
            for j in 0..self.faces[i].edge_count as usize {
                let edge_index: i32 = self.surface_edges[self.faces[i].first_edge_index as usize + j];
                let vertex: glm::Vec3 = if edge_index >= 0 {
                    self.vertices[self.edges[edge_index as usize].vertex_index[0] as usize]
                } else {
                    self.vertices[self.edges[(-edge_index) as usize].vertex_index[1] as usize]
                };
                let f_u: f32 = glm::dot(tex_info.s, vertex) + tex_info.s_shift;
                if f_u < f_min_u {
                    f_min_u = f_u;
                }
                if f_u > f_max_u {
                    f_max_u = f_u;
                }
                let f_v: f32 = glm::dot(tex_info.t, vertex) + tex_info.t_shift;
                if f_v < f_min_v {
                    f_min_v = f_v;
                }
                if f_v > f_max_v {
                    f_max_v = f_v;
                }
            }
            let f_tex_min_u: f32 = (f_min_u / 16.0).floor();
            let f_tex_min_v: f32 = (f_min_v / 16.0).floor();
            let f_tex_max_u: f32 = (f_max_u / 16.0).ceil();
            let f_tex_max_v: f32 = (f_max_v / 16.0).ceil();
            let n_width: i32 = (f_tex_max_u - f_tex_min_u) as i32 + 1;
            let n_height: i32 = (f_tex_max_v - f_tex_min_v) as i32 + 1;
            // End QRAD
            let f_mid_poly_u: f32 = (f_min_u + f_max_u) / 2.0;
            let f_mid_poly_v: f32 = (f_min_v + f_max_v) / 2.0;
            let f_mid_tex_u: f32 = n_width as f32 / 2.0;
            let f_mid_tex_v: f32 = n_height as f32 / 2.0;
            for j in 0..self.faces[i].edge_count as usize {
                let edge_index: i32 = self.surface_edges[self.faces[i].first_edge_index as usize + j];
                let vertex: glm::Vec3 = if edge_index >= 0 {
                    self.vertices[self.edges[edge_index as usize].vertex_index[0] as usize]
                } else {
                    self.vertices[self.edges[(-edge_index) as usize].vertex_index[1] as usize]
                };
                let f_u: f32 = glm::dot(tex_info.s, vertex) + tex_info.s_shift;
                let f_v: f32 = glm::dot(tex_info.t, vertex) + tex_info.t_shift;
                let f_lightmap_u: f32 = f_mid_tex_u + (f_u - f_mid_poly_u) / 16.0;
                let f_lightmap_v: f32 = f_mid_tex_v + (f_v + f_mid_poly_v) / 16.0;
                self.face_tex_coords[i].lightmap_coords[j].x = f_lightmap_u / n_width as f32;
                self.face_tex_coords[i].lightmap_coords[j].y = f_lightmap_v / n_height as f32;
            }
            let mut image: Image = Image {
                channels: 3,
                width: n_width as usize,
                height: n_height as usize,
                data: Vec::from_iter(p_light_map_data[self.faces[i].lightmap_offset as usize..(n_width + n_height * 3) as usize].iter().cloned()),
            };
            self.m_lightmaps.push(image);
            loaded_lightmaps += 1;
            loaded_bytes += (n_width + n_height * 3) as usize;
        }
        info!(
            &crate::LOGGER,
            "Loaded {} lightmaps, lightmap data diff: {} bytes",
            loaded_lightmaps,
            loaded_bytes - self.header.lump[bsp30::LumpType::LumpLighting as usize].length as usize
        );
    }

    pub (crate) fn load_models(&mut self, reader: &mut BufReader<File>) {
        let mut sub_models: Vec<bsp30::Model> = Vec::with_capacity(
            self.header.lump[bsp30::LumpType::LumpModels as usize].length as usize / std::mem::size_of::<bsp30::Model>()
        );
        reader.seek(SeekFrom::Start(self.header.lump[bsp30::LumpType::LumpModels as usize].offset as u64));
        for _ in 0..sub_models.len() {
            sub_models.push(bsp30::Model::from_reader(reader).unwrap());
        }
        self.hull_0_clip_nodes = self.nodes.iter().map(|node: &bsp30::Node| -> bsp30::ClipNode {
            let mut clipnode: bsp30::ClipNode = Default::default();
            clipnode.plane_index = node.plane_index as i32;
            for j in 0..2 {
                if node.child_index[j] < 0 {
                    clipnode.child_index[j] = self.leaves[!node.child_index[j] as usize].content as i16;
                } else {
                    clipnode.child_index[j] = node.child_index[j];
                }
            }
            return clipnode;
        }).collect();
        let mut model_0: Model = Model::new();
        let mut hull_0: &mut Hull = &mut model_0.hulls[0];
        hull_0.clip_nodes = self.hull_0_clip_nodes.iter()
            .map(|cn: &bsp30::ClipNode| bsp30::ClipNode {
                plane_index: cn.plane_index,
                child_index: [cn.child_index[0], cn.child_index[1]],
            }).collect();
        hull_0.first_clip_node = 0;
        hull_0.last_clip_node = self.hull_0_clip_nodes.len() as isize - 1isize;
        hull_0.planes = self.planes.iter()
            .map(|plane: &bsp30::Plane| bsp30::Plane {
                normal: plane.normal,
                dist: plane.dist,
                r#type: plane.r#type,
            }).collect();
        for i in 1..=3 {
            let mut hull: &mut Hull = &mut model_0.hulls[i];
            hull.clip_nodes = self.clip_nodes.iter()
                .map(|cn: &bsp30::ClipNode| bsp30::ClipNode {
                    plane_index: cn.plane_index,
                    child_index: [cn.child_index[0], cn.child_index[1]],
                }).collect();
            hull.first_clip_node = 0;
            hull.last_clip_node = self.clip_nodes.len() as isize - 1isize;
            hull.planes = self.planes.iter()
                .map(|plane: &bsp30::Plane| bsp30::Plane {
                    normal: plane.normal,
                    dist: plane.dist,
                    r#type: plane.r#type,
                }).collect();
        }
        let hull_1: &mut Hull = &mut model_0.hulls[1];
        hull_1.clip_mins[0] = -16.0;
        hull_1.clip_mins[1] = -16.0;
        hull_1.clip_mins[1] = -36.0;
        hull_1.clip_maxs[0] = 16.0;
        hull_1.clip_maxs[1] = 16.0;
        hull_1.clip_maxs[2] = 36.0;

        let hull_2: &mut Hull = &mut model_0.hulls[2];
        hull_2.clip_mins[0] = -32.0;
        hull_2.clip_mins[1] = -32.0;
        hull_2.clip_mins[2] = -32.0;
        hull_2.clip_maxs[0] = 32.0;
        hull_2.clip_maxs[1] = 32.0;
        hull_2.clip_maxs[2] = 32.0;

        let hull_3: &mut Hull = &mut model_0.hulls[3];
        hull_3.clip_mins[0] = -16.0;
        hull_3.clip_mins[1] = -16.0;
        hull_3.clip_mins[1] = -18.0;
        hull_3.clip_maxs[0] = 16.0;
        hull_3.clip_maxs[1] = 16.0;
        hull_3.clip_maxs[2] = 18.0;
        for i in 0..sub_models.len() {
            if i != 0 {
                self.models.push(self.models.last().unwrap().clone())
            }
            let mut model: &mut Model = &mut self.models[self.models.len() - 1];
            model.model = sub_models[i];
        }
        todo!()
    }

    fn is_brush_entity(entity: &Entity) -> bool {
        if entity.find_property("model".to_string()).is_none() {
            return false;
        }
        let classname: &String = match entity.find_property("classname".to_string()) {
            Some(value) => value,
            None => return false,
        };
        return match classname.as_str() {
            "func_door_rotating"
                | "func_door"
                | "func_illusionary"
                | "func_wall"
                | "func_breakable"
                | "func_button" => true,
            _ => false,
        };
    }

    pub (crate) fn parse_entities(&mut self, entities_string: &String) {
        let mut pos: usize = 0;
        loop {
            pos = match entities_string[pos..].find('{') {
                Some(new_pos) => new_pos,
                None => break,
            };
            let end: usize = match entities_string[pos..].find('}') {
                Some(end_pos) => end_pos,
                None => {
                    error!(&crate::LOGGER, "Cannot find ending brace for entity, skipping.");
                    continue;
                },
            };
            self.entities.push(Entity::new(&entities_string[(pos + 1)..(end - pos - 1)].to_string()));
            pos = end + 1;
        }
    }

    pub (crate) fn count_vis_leaves(&self, i_node: i16) -> usize {
        if i_node < 0 {
            if i_node == -1 || self.leaves[!(i_node as usize)].content == bsp30::ContentType::ContentsSolid as i32 {
                return 0;
            }
            return 1;
        }
        let left_node_count: usize = self.count_vis_leaves(self.nodes[i_node as usize].child_index[0]);
        let right_node_count: usize = self.count_vis_leaves(self.nodes[i_node as usize].child_index[1]);
        return left_node_count + right_node_count;
    }

    pub (crate) fn decompress_vis(&self, leaf: usize, compresed_vis: Vec<u8>) -> BitSet {
        let mut pvs: BitSet = BitSet::new();
        pvs.reserve_len(self.leaves.len() - 1);
        let mut read: usize = self.leaves[leaf].vis_offset as usize;
        let row: usize = (self.vis_lists.len() + 7) / 8;
        while pvs.len() / 8 < row {
            if read > compresed_vis.len() {
                pvs.insert(0usize);
            } else {
                read += 1;
                for _ in 0..read {
                    pvs.insert(0x00);
                    if pvs.len() / 8 >= row {
                        break;
                    }
                }
            }
            read += 1;
        }
        return pvs;
    }
    
    fn array_to_vec3(arr: [i16; 3]) -> glm::Vec3 {
        return glm::vec3(
            arr[0] as f32,
            arr[1] as f32,
            arr[2] as f32,
        );
    }

    pub (crate) fn find_leaf(&self, pos: glm::Vec3, node: usize) -> Option<i16> {
        for child_index in self.nodes[node].child_index {
            if child_index >= 0 && point_in_box(
                pos,
                BSP::array_to_vec3(self.nodes[child_index as usize].lower),
                BSP::array_to_vec3(self.nodes[child_index as usize].upper),
            ) {
                return self.find_leaf(pos, child_index as usize);
            } else if (!child_index) != 0 && point_in_box(
                pos,
                BSP::array_to_vec3(self.leaves[!child_index as usize].lower),
                BSP::array_to_vec3(self.leaves[!child_index as usize].upper),
            ) {
                return Some(!child_index);
            }
        }
        return None;
    }

}
