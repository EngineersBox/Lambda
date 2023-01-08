use std::collections::HashMap;
use std::path::Path;
use std::io::{Result, Error, ErrorKind, BufReader, Seek, SeekFrom};
use std::fs::{File, OpenOptions};
use bit_set::BitSet;
use lazy_static::lazy_static;
use byteorder::ReadBytesExt;

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
    pub vis_lists: Vec<BitSet>,
    pub m_textures: Vec<MipmapTexture>,
    pub m_lightmaps: Vec<Image>,
    pub hull_0_clip_nodes: Vec<bsp30::ClipNode>,
    pub models: Vec<Model>,
}

lazy_static!{
    static ref WAD_DIR: String = String::from("data/wads");
    static ref SKY_DIR: String = String::from("data/textures/sky");
    static ref SKY_NAME_SUFFIXES: [String; 6] = [
        String::from("ft"),
        String::from("bk"),
        String::from("up"),
        String::from("dn"),
        String::from("rt"),
        String::from("lf"),
    ];
}

impl BSP {

    pub fn from_file(path: &String) -> Result<Self> {
        let file: File = match OpenOptions::new()
            .read(true)
            .open(path) {
            Ok(f) => f,
            Err(error) => return Err(Error::new(
                error.kind(),
                format!("Failed to open BSP file for reading: {}", error.to_string())
            ))
        };
        let mut reader: BufReader<File> = BufReader::new(file);
        info!(&crate::LOGGER, "Loading BSP file: {}", path);
        let header: bsp30::Header = bsp30::Header::from_reader(&mut reader)?;
        if header.version != 30 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid BSP version {}, expected 30", header.version)
            ));
        }
        let mut bsp: BSP = BSP {
            header,
            vertices: Vec::new(),
            edges: Vec::new(),
            surface_edges: Vec::new(),
            nodes: Vec::new(),
            leaves: Vec::new(),
            mark_surfaces: Vec::new(),
            planes: Vec::new(),
            faces: Vec::new(),
            clip_nodes: Vec::new(),
            texture_header: Default::default(),
            mip_textures: Vec::new(),
            mip_texture_offsets: Vec::new(),
            texture_infos: Vec::new(),
            face_tex_coords: Vec::new(),
            entities: Vec::new(),
            brush_entities: Vec::new(),
            special_entities: Vec::new(),
            wad_files: Vec::new(),
            decal_wads: Vec::new(),
            m_decals: Vec::new(),
            vis_lists: Vec::new(),
            m_textures: Vec::new(),
            m_lightmaps: Vec::new(),
            hull_0_clip_nodes: Vec::new(),
            models: Vec::new(),
        };
        // Init and read BSP component vectors
        macro_rules! bsp_comp_init {
            ($name:ident,$lump_type:expr,$element_type:ty) => {
                bsp.$name = Vec::with_capacity(
                    bsp.header.lump[$lump_type as usize].length as usize / std::mem::size_of::<$element_type>()
                );
                reader.seek(SeekFrom::Start(bsp.header.lump[$lump_type as usize].offset as u64))?;
                for _ in 0..bsp.$name.capacity() {
                    bsp.$name.push(<$element_type>::from_reader(&mut reader)?);
                }
            }
        }
        bsp_comp_init!(nodes, bsp30::LumpType::LumpNodes, bsp30::Node);
        bsp_comp_init!(leaves, bsp30::LumpType::LumpLeaves, bsp30::Leaf);
        bsp_comp_init!(mark_surfaces, bsp30::LumpType::LumpMarkSurfaces, bsp30::MarkSurface);
        bsp_comp_init!(faces, bsp30::LumpType::LumpFaces, bsp30::Face);
        bsp_comp_init!(clip_nodes, bsp30::LumpType::LumpClipNodes, bsp30::ClipNode);
        bsp_comp_init!(surface_edges, bsp30::LumpType::LumpSurfaceEdges, bsp30::SurfaceEdge);
        bsp_comp_init!(edges, bsp30::LumpType::LumpEdges, bsp30::Edge);
        bsp_comp_init!(vertices, bsp30::LumpType::LumpVertexes, bsp30::Vertex);
        bsp_comp_init!(planes, bsp30::LumpType::LumpPlanes, bsp30::Plane);
        // Read and parse entities
        let mut entity_buffer: Vec<u8> = Vec::with_capacity(bsp.header.lump[bsp30::LumpType::LumpEntities as usize].length as usize);
        reader.seek(SeekFrom::Start(bsp.header.lump[bsp30::LumpType::LumpEntities as usize].offset as u64))?;
        for _ in 0..entity_buffer.capacity() {
            entity_buffer.push(reader.read_u8().unwrap());
        }
        bsp.entities = BSP::parse_entities(&String::from_utf8(entity_buffer).unwrap());
        debug!(&crate::LOGGER, "Parsed entities");
        // Textures
        bsp.texture_infos = Vec::with_capacity(bsp.header.lump[bsp30::LumpType::LumpTexinfo as usize].length as usize / std::mem::size_of::<bsp30::TextureInfo>());
        reader.seek(SeekFrom::Start(bsp.header.lump[bsp30::LumpType::LumpTexinfo as usize].offset as u64))?;
        for _ in 0..bsp.texture_infos.capacity() {
            bsp.texture_infos.push(bsp30::TextureInfo::from_reader(&mut reader).unwrap());
        }
        debug!(&crate::LOGGER, "Read texture infos");
        reader.seek(SeekFrom::Start(bsp.header.lump[bsp30::LumpType::LumpTextures as usize].offset as u64))?;
        bsp.texture_header = bsp30::TextureHeader::from_reader(&mut reader).unwrap();
        println!("Texture header: {:?}", bsp.texture_header);
        debug!(&crate::LOGGER, "Read texture header");
        bsp.mip_textures = Vec::with_capacity(bsp.texture_header.mip_texture_count as usize);
        bsp.mip_texture_offsets = Vec::with_capacity(bsp.texture_header.mip_texture_count as usize);
        for _ in 0..bsp.mip_texture_offsets.capacity() {
            bsp.mip_texture_offsets.push(bsp30::MipTexOffset::from_reader(&mut reader).unwrap());
        }
        debug!(&crate::LOGGER, "Read mip texture offsets");
        for i in 0..bsp.mip_textures.capacity() {
            reader.seek(SeekFrom::Start(bsp.header.lump[bsp30::LumpType::LumpTextures as usize].offset as u64 + bsp.mip_texture_offsets[i] as u64))?;
            bsp.mip_textures.push(bsp30::MipTex::from_reader(&mut reader).unwrap());
        }
        debug!(&crate::LOGGER, "Read mip textures");
        bsp.load_textures(&mut reader);
        debug!(&crate::LOGGER, "Loaded textures");
        // Lightmaps
        if bsp.header.lump[bsp30::LumpType::LumpLighting as usize].length == 0 {
            info!(&crate::LOGGER, "No lightmaps to load, skipping");
        } else {
            let mut p_lightmap_data: Vec<u8> = Vec::with_capacity(bsp.header.lump[bsp30::LumpType::LumpLighting as usize].length as usize);
            reader.seek(SeekFrom::Start(bsp.header.lump[bsp30::LumpType::LumpLighting as usize].offset as u64))?;
            for _ in 0..p_lightmap_data.capacity() {
                p_lightmap_data.push(reader.read_u8().unwrap());
            }
            bsp.load_light_maps(p_lightmap_data);
            debug!(&crate::LOGGER, "Loaded lightmaps")
        }
        // Decals
        bsp.load_decals();
        debug!(&crate::LOGGER, "Loaded decals");
        // Visibility list
        if bsp.header.lump[bsp30::LumpType::LumpVisibility as usize].length <= 0 {
            info!(&crate::LOGGER, "No visibility lists to load, skipping");
        } else {
            let mut compressed_vis: Vec<u8> = Vec::with_capacity(bsp.header.lump[bsp30::LumpType::LumpVisibility as usize].length as usize);
            reader.seek(SeekFrom::Start(bsp.header.lump[bsp30::LumpType::LumpVisibility as usize].offset as u64))?;
            for _ in 0..compressed_vis.capacity() {
                compressed_vis.push(reader.read_u8().unwrap());
            }
            info!(&crate::LOGGER, "Decompressing visibility list");
            let count: usize = bsp.count_vis_leaves(0);
            bsp.vis_lists = Vec::with_capacity(count);
            for i in 0..count {
                if bsp.leaves[i + 1].vis_offset >= 0 {
                    bsp.vis_lists[i] = bsp.decompress_vis(i + 1, &compressed_vis);
                }
            }
            debug!(&crate::LOGGER, "Loaded visibility list");
        }
        // Close file through reader
        std::mem::drop(reader);
        debug!(&crate::LOGGER, "Dropped file");
        for i in 0..bsp.entities.len() {
            let entity: &Entity = &bsp.entities[i];
            if BSP::is_brush_entity(entity) {
                bsp.brush_entities.push(i);
                if let Some(sz_origin) = entity.find_property(&"origin".to_string()) {
                    let i_model: usize = entity.find_property(&"model".to_string())
                        .unwrap()
                        .chars()
                        .nth(1)
                        .unwrap() as usize;
                    let mut origin: glm::Vec3 = bsp.models[i_model].model.origin;
                    macro_rules! scan {
                        ($string:expr, $sep:expr, $( $x:ty ),+) => {{
                            let mut iter = $string.split($sep);
                            ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
                        }}
                    }
                    let origin_points: (Option<f32>, Option<f32>, Option<f32>) = scan!(sz_origin, char::is_whitespace, f32, f32, f32);
                    origin.x = origin_points.0.unwrap();
                    origin.y = origin_points.1.unwrap();
                    origin.z = origin_points.2.unwrap();
                }
            } else {
                bsp.special_entities.push(i);
            }
        }
        debug!(&crate::LOGGER, "Loaded brush and special entities");
        std_tools::partition(
            &mut bsp.brush_entities,
            |i: &usize| -> bool {
            if let Some(sz_render_mode_1) = bsp.entities[*i].find_property(&"rendermode".to_string()) {
                if sz_render_mode_1.parse::<usize>().unwrap() == bsp30::RenderMode::RenderModeTexture as usize {
                    return true;
                }
            }
            return false;
        });
        info!(&crate::LOGGER, "Finished loading BSP");
        return Ok(bsp);
    }

    pub fn find_entity<'a>(entities: &'a Vec<Entity>, name: String) -> Option<&Entity> {
        for entity in entities.iter() {
            if let Some(classname) = entity.find_property(&"classname".to_string()) {
                if *classname == name {
                    return Some(entity);
                }
            }
        }
        return None;
    }
    
    pub fn find_entities<'a>(entities: &'a Vec<Entity>, name: String) -> Vec<&Entity> {
        let mut result: Vec<&Entity> = Vec::new();
        for entity in entities.iter() {
            if let Some(classname) = entity.find_property(&"classname".to_string()) {
                if *classname == name {
                    result.push(entity);
                }
            }
        }
        return result;
    }

    pub fn load_skybox(&self) -> Option<[Image; 6]> {
        let world_spawn: Option<&Entity> = BSP::find_entity(&self.entities, "world_spawn".to_string());
        let skyname: Option<&String> = world_spawn?.find_property(&"skyname".to_string());
        let mut result: Vec<Image> = Vec::with_capacity(6);
        for i in 0..6 {
            result.push(Image::from_path(&(
                SKY_DIR.clone()
                + "/"
                + skyname?.as_str()
                + SKY_NAME_SUFFIXES[i].clone().as_str()
                + ".tga"
            )));
        }
        return result.try_into().ok();
    }

    pub (crate) fn load_wad_files(wad_str: &String) -> Vec<Wad> {
        let wad_string: String = wad_str.replace("\\", "/");
        let mut wad_count: usize = 0;
        let mut wad_files: Vec<Wad> = Vec::new();
        for path_str in wad_string.split(";") {
            if path_str.is_empty() {
                continue;
            }
            let mut wad_path = Path::new(path_str);
            if let Ok(stripped_path) = wad_path.strip_prefix("/") {
                wad_path = stripped_path;
            }
            let mut path: String = if let Some(parent_path) = wad_path.parent() {
                Path::new(parent_path.file_name().unwrap())
                    .join(wad_path.file_name().unwrap())
                    .as_path()
                    .to_string_lossy()
                    .to_string()
            } else {
                wad_path.to_string_lossy().to_string()
            };
            path = Path::new(WAD_DIR.as_str())
                .join(path)
                .to_string_lossy()
                .to_string();
            info!(&crate::LOGGER, "({}) Loading WAD {}", wad_count, path);
            wad_files.push(Wad::new(&path));
            wad_count += 1;
        }
        info!(&crate::LOGGER, "Loaded {} WADs", wad_count);
        return wad_files;
    }

    pub (crate) fn unload_wad_files(&mut self) {
        self.wad_files.clear();
    }

    pub (crate) fn load_textures(&mut self, reader: &mut BufReader<File>) {
        if let Some(world_spawn) = BSP::find_entity(&self.entities, "worldspawn".to_string()) {
            if let Some(wad) = world_spawn.find_property(&String::from("wad")) {
                info!(&crate::LOGGER, "Loading texture WADs");
                self.wad_files.append(&mut BSP::load_wad_files(wad));
            } else {
                warn!(&crate::LOGGER, "No 'wad' property present on 'worldspawn' entity, skipping texture loading");
            }
        } else {
            error!(&crate::LOGGER, "No 'worldspawn' entity present in BSP");
        }
        info!(&crate::LOGGER, "Loading textures...");
        self.m_textures.resize_with(self.texture_header.mip_texture_count as usize, || MipmapTexture::new());
        let mut errors: usize = 0;
        for i in 0..self.texture_header.mip_texture_count as usize {
            debug!(&crate::LOGGER, "Loading texture {}", String::from_utf8_lossy(&self.mip_textures[i].name));
            if self.mip_textures[i].offsets[0] == 0 {
                // External texture
                if let Some(tex) = self.load_texture_from_wads(&String::from_utf8_lossy(&self.mip_textures[i].name).to_string()) {
                    self.m_textures[i] = tex;
                }  else {
                    error!(&crate::LOGGER, "Failed to load texture {}", String::from_utf8_lossy(&self.mip_textures[i].name));
                    errors += 1;
                    continue;
                }
            } else {
                // Internal texture
                let mip_tex: &bsp30::MipTex = &self.mip_textures[i];
                let data_size: usize = std::mem::size_of::<u8>() * (mip_tex.offsets[3] + (mip_tex.height / 8) * (mip_tex.width / 8) + 2 + 768) as usize;
                let mut img_data: Vec<u8> = Vec::with_capacity(data_size);
                reader.seek(SeekFrom::Start(self.header.lump[bsp30::LumpType::LumpTextures as usize].offset as u64 + self.mip_texture_offsets[i] as u64))
                    .expect("Unable to seek to textures lump offset for internal texture");
                // TODO: Check header magic id, if not 30 then use Quake palette
                for _ in 0..data_size {
                    img_data.push(reader.read_u8().unwrap());
                }
                self.m_textures[i] = Wad::create_mip_texture(&img_data);
            }
        }
        self.unload_wad_files();
        info!(&crate::LOGGER, "Loaded {} textures, {} failed", self.texture_header.mip_texture_count as usize - errors, errors);
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

    pub (crate) fn load_texture_from_wads(&mut self, name: &String) -> Option<MipmapTexture> {
        for wad in self.wad_files.iter_mut() {
            if let Some(p_mipmap_tex) = wad.load_texture(name) {
                return Some(p_mipmap_tex);
            }
        }
        return None;
    }

    pub (crate) fn load_decal_texture(decal_wads: &mut Vec<Wad>, name: &String) -> Option<MipmapTexture> {
        for decal_wad in decal_wads.iter_mut() {
            if let Some(p_mipmap_tex) = decal_wad.load_texture(name) {
                return Some(p_mipmap_tex);
            }
        }
        return None;
    }

    pub (crate) fn load_decals(&mut self) {
        self.decal_wads.push(Wad::new(&Path::new(WAD_DIR.as_str()).join("valve/decals.wad").to_string_lossy().to_string()));
        self.decal_wads.push(Wad::new(&Path::new(WAD_DIR.as_str()).join("cstrike/decals.wad").to_string_lossy().to_string()));
        let info_decals: Vec<&Entity> = BSP::find_entities(&self.entities, "infodecal".to_string()).clone();
        if info_decals.is_empty() {
            info!(&crate::LOGGER, "No decals to load, skipping");
            return;
        }
        let mut loaded_tex: HashMap<String, usize> = HashMap::new();
        let mut new_m_textures: Vec<MipmapTexture> = Vec::new();
        let mut new_m_decals: Vec<Decal> = Vec::new();
        for info_decal in info_decals.iter().copied() {
            let origin_str: Option<&String> = info_decal.find_property(&"origin".to_string());
            if origin_str.is_none() {
                continue;
            }
            let split_origin: Vec<&str> = origin_str.unwrap().split(" ").collect();
            if split_origin.len() != 3 {
                error!(&crate::LOGGER, "Expected 3D origin, got {}, skipping", split_origin.len());
                continue;
            }
            let origin: glm::Vec3 = glm::vec3(
                split_origin[0].parse::<f32>().unwrap(),
                split_origin[1].parse::<f32>().unwrap(),
                split_origin[2].parse::<f32>().unwrap(),
            );
            let leaf: Option<i16> = self.find_leaf(origin, 0);
            if leaf.is_none() {
                error!(&crate::LOGGER, "Cannot find decal leaf, skipping");
                continue;
            }
            let current_leaf: Option<&bsp30::Leaf> = self.leaves.get(leaf.unwrap() as usize);
            if current_leaf.is_none() {
                error!(&crate::LOGGER, "Cannot find leaf, skipping");
                continue;
            }
            let current_leaf_value: &bsp30::Leaf = current_leaf.unwrap();
            for j in 0..current_leaf_value.mark_surface_count as usize {
                let face: &bsp30::Face = &self.faces[self.mark_surfaces[current_leaf_value.first_mark_surface as usize + j] as usize];
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
                let tex_name: Option<&String> = info_decal.find_property(&"texture".to_string());
                if tex_name.is_none() {
                    error!(&crate::LOGGER, "Unable to retrieve texture name from decal");
                    break;
                }
                let it: Option<&usize> = loaded_tex.get(tex_name.unwrap());
                let mut it_val: usize = 0;
                if it.is_none() {
                    let loaded_decal_texture: Option<MipmapTexture> = BSP::load_decal_texture(&mut self.decal_wads, &tex_name.unwrap());
                    if loaded_decal_texture.is_none() {
                        error!(&crate::LOGGER, "Unable to load mipmap texture for {}", &tex_name.unwrap());
                        break;
                    }
                    it_val = self.m_textures.len();
                    loaded_tex.insert(tex_name.unwrap().clone(), self.m_textures.len());
                    new_m_textures.push(loaded_decal_texture.unwrap());
                }
                let img_0: &Image = &self.m_textures[it_val].img[0];
                let h2: f32 = img_0.height as f32 / 2.0;
                let w2: f32 = img_0.width as f32 / 2.0;
                let s: glm::Vec3 = self.texture_infos[face.texture_info as usize].s;
                let t: glm::Vec3 = self.texture_infos[face.texture_info as usize].t;
                new_m_decals.push(Decal {
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
        self.m_textures.append(&mut new_m_textures);
        self.m_decals.append(&mut new_m_decals);
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
            let image: Image = Image {
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
        reader.seek(SeekFrom::Start(self.header.lump[bsp30::LumpType::LumpModels as usize].offset as u64)).expect("Unable to seek to models lump in BSP file");
        for _ in 0..sub_models.capacity() {
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
        for i in 0..sub_models.capacity() {
            if i != 0 {
                self.models.push(self.models.last().unwrap().clone())
            }
            let index: usize = self.models.len() - 1;
            let mut model: &mut Model = &mut self.models[index];
            model.model = sub_models[i];
        }
        todo!()
    }

    fn is_brush_entity(entity: &Entity) -> bool {
        if entity.find_property(&"model".to_string()).is_none() {
            return false;
        }
        let classname: &String = match entity.find_property(&"classname".to_string()) {
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

    pub (crate) fn parse_entities(entities_string: &String) -> Vec<Entity> {
        let mut entities: Vec<Entity> = Vec::new();
        let mut pos: usize = 0;
        loop {
            pos += match entities_string[pos..].find('{') {
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
            entities.push(Entity::new(&entities_string[(pos + 1)..(pos + end - 1)].to_string()));
            pos += end + 1;
        }
        return entities;
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

    pub (crate) fn decompress_vis(&self, leaf: usize, compresed_vis: &Vec<u8>) -> BitSet {
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
   
    #[inline(always)]
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
