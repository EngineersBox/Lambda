const MAX_MAP_HULLS: usize = 4;

const MAX_MAP_MODELS: usize = 400;
const MAX_MAP_BRUSHES: usize = 4096;
const MAX_MAP_ENTITIES: usize = 1024;
const MAX_MAP_ENTSTRING: usize = (128 * 1024);

const MAX_MAP_PLANES: usize = 32767;
const MAX_MAP_NODES: usize = 32767; // Negative shorts are leaves
const MAX_MAP_CLIPNODES: usize = 32767;
const MAX_MAP_LEAFS: usize = 8192;
const MAX_MAP_VERTS: usize = 65535;
const MAX_MAP_FACES: usize = 65535;
const MAX_MAP_MARKSURFACES: usize = 65535;
const MAX_MAP_TEXINFO: usize = 8192;
const MAX_MAP_EDGES: usize = 256000;
const MAX_MAP_SURFEDGES: usize = 512000;
const MAX_MAP_TEXTURES: usize = 512;
const MAX_MAP_MIPTEX: usize = 0x200000;
const MAX_MAP_LIGHTING: usize = 0x200000;
const MAX_MAP_VISIBILITY: usize = 0x200000;

const MAX_MAP_PORTALS: usize = 65536;

const MAX_KEY: usize = 32;
const MAX_VALUE: usize = 1024;

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

enum ContentType {
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


