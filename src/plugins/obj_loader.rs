/* == OBJ Loader Plugin ==
 * The .obj format represents the position of each vertex,
 * the UV position of each texture coordinate vertex, the vertex normals,
 * and the faces that make each polygon defined as a list of vertices, and texture vertices.
 *
 * Vertices are stored in a counter-clockwise order by default. OBJ coords have no units.
 * === crate.obj
 * = # List of geometric vertices, with (x, y, z, [w = 1.0]) coordinates.
 * = v = 0.123 0.234 0.345
 * = v ...
 * = # List of texture coordinates in (u, [v = 0, w = 0]) space (range = 0.0 - 1.0)
 * = vt 0.500 1 [0]
 * ...
 * ===============
*/
use bevy::log::debug;
use bevy::app::{App, Plugin};
use bevy::render::{
    mesh::{Indices, Mesh},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use bevy::asset::{
    AssetApp, AssetLoader, AsyncReadExt, LoadContext, io::Reader
};
use thiserror::Error;
use bevy::utils::ConditionalSendFuture;
// List of supported file extensions
const EXTENSIONS: &[&str; 1] = &["obj"];

// Custom Types
pub type AssetType = Mesh;

// .obj Loader Plugin
#[derive(Default)]
pub struct ObjLoaderPlugin;
impl Plugin for ObjLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.preregister_asset_loader::<ObjLoader>(EXTENSIONS);
    }
    fn finish(&self, app: &mut App) {
        app.register_asset_loader(ObjLoader);
    }
}

// Custom Error Handlers
#[derive(Error, Debug)]
pub enum ObjLoaderError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid OBJ file: {0}")]
    InvalidFile(#[from] tobj::LoadError),
}

async fn load_obj<'a, 'b>(
    bytes: &'a [u8],
    _load_context: &'a mut LoadContext<'b>,
) -> Result<Mesh, ObjLoaderError> {
    load_obj_from_bytes(bytes)
}

pub fn load_obj_from_bytes(mut bytes: &[u8]) -> Result<Mesh, ObjLoaderError> {
    // Import Phase //
    let options = tobj::LoadOptions {
        single_index: true,
        triangulate: false,
        ..Default::default()
    };
    let obj = tobj::load_obj_buf(&mut bytes, &options, |_| {
        Err(tobj::LoadError::GenericFailure)
    })?;

    let mut indices = Vec::new();
    let mut vertex_position = Vec::new();
    let mut vertex_normal = Vec::new();
    let mut vertex_texture = Vec::new();
    let mut vertex_face: Vec<u32> = Vec::new();

    for model in obj.0 {
        // Logs
        bevy::log::debug!("Indices: {}", indices.len());
        // Offset of indices
        let index_offset = vertex_position.len() as u32;
        // N: Vec::reserve(&mut self, add: usize) - reserve capacity for at least 'add' more elements.
        bevy::log::debug!("Position Vertices: {}", model.mesh.positions.len());
        vertex_position.reserve(model.mesh.positions.len() / 3);

        bevy::log::debug!("Vertex Normals: {}", model.mesh.normals.len());
        vertex_normal.reserve(model.mesh.normals.len() / 3);

        bevy::log::debug!("Texture Coordinates: {}", model.mesh.texcoords.len());
        vertex_texture.reserve(model.mesh.texcoords.len() / 2);

        // N: Face Arities: The number of vertices (arity) of each face. Empty if triangulate = true.
        bevy::log::debug!("Face Arities: {}", model.mesh.face_arities.len());
        vertex_face.reserve(model.mesh.face_arities.len());

        // N: Vec::extend extends a collection with the contents of an iterator
        // Vertex Positions
        vertex_position.extend(
            model
                .mesh
                .positions // Vec<f64> btw
                // N: returns and iterator over (chumk_size = 3) elements of the slice at a time
                .chunks_exact(3)
                // Construct a new array containing the first 3 elements of each chunk v
                .map(|v| [v[0], v[1], v[2]]),
                // N: I believe this all makes sense due to the syntax of an .obj file (read like p,p,p,n,n,n,...)
        );

        // Log vertex positions
        for vp in vertex_position.iter() {
            debug!("v {}, {}, {}", vp[0], vp[1], vp[2])
        }

        // Vertex Normals
        vertex_normal.extend(
            model
                .mesh
                .normals
                .chunks_exact(3)
                .map(|vn| [vn[0], vn[1], vn[2]]),
        );

        // Log vertex normals
        for vn in vertex_normal.iter() {
            debug!("vn {}, {}, {}", vn[0], vn[1], vn[2])
        }

        // Texture Coordinates from mesh
        vertex_texture.extend(
            model
                .mesh
                .texcoords
                .chunks_exact(2)
                .map(|vt| [vt[0], 1.0 - vt[1]]),
        );

        // Log texture coordinates
        for vt in vertex_texture.iter() {
            debug!("vt {}, {}", vt[0], vt[1])
        }

        // Extend parent meshes indices by the current meshes indices plus the index offset
        indices.extend(model.mesh.indices.iter().map(|i| i + index_offset));
    }

    // Export Phase //
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Insert imported indices into mesh
    mesh.insert_indices(Indices::U32(indices));
    // Insert imported vertex positions into mesh
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_position);
    // Insert imported texture normals into mesh (if they exist)
    if !vertex_normal.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normal);
    } else {
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
    }

    // Insert imported texture coordinates into mesh
    if !vertex_texture.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vertex_texture);
    }

    Ok(mesh)
}

// Asset Loader Implementation
struct ObjLoader;
impl AssetLoader for ObjLoader {
    type Error = ObjLoaderError;
    type Settings = ();
    type Asset = AssetType;

    // Note: 'a = explicit lifetime
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            load_obj(&bytes, load_context).await
        })
    }

    fn extensions(&self) -> &[&str] {
        EXTENSIONS
    }
}

