use std::
{
    io,
    hash::{Hash, Hasher, BuildHasherDefault},
    collections::HashMap,
};
use twox_hash::XxHash32;
use crate::gfx::GfxError;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Vertex
{
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texcoord: [f32; 2]
}

impl Hash for Vertex
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        for f in &self.position
        {
            f.to_bits().hash(state);
        }
        for f in &self.normal
        {
            f.to_bits().hash(state);
        }
        for f in &self.texcoord
        {
            f.to_bits().hash(state);
        }
    }
}

impl PartialEq for Vertex
{
    fn eq(&self, other: &Self) -> bool
    {
        approx_eq!(f32, self.position[0], other.position[0])    &&
        approx_eq!(f32, self.position[1], other.position[1])    &&
        approx_eq!(f32, self.position[2], other.position[2])    &&

        approx_eq!(f32, self.normal[0], other.normal[0])        &&
        approx_eq!(f32, self.normal[1], other.normal[1])        &&
        approx_eq!(f32, self.normal[2], other.normal[2])        &&

        approx_eq!(f32, self.texcoord[0], other.texcoord[0])    &&
        approx_eq!(f32, self.texcoord[1], other.texcoord[1])
    }
}

impl Eq for Vertex {}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Mesh
{
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh
{
    /// Loads mesh from reader for an OBJ file
    ///
    /// `obj_reader` is the reader for the OBJ file
    pub fn from_reader<R: io::Read>(obj_reader: R) -> Result<Mesh, tobj::LoadError>
    {

        // Load OBJ and associate with materials
        let mut bufreader = io::BufReader::new(obj_reader);
        let (models, _materials) = tobj::load_obj_buf(&mut bufreader, true, |p|
            {
                // Placeholder to ignore any material files
                tobj::load_mtl_buf(&mut io::BufReader::new("".to_string().as_bytes()))
            })?;


        // Keep track of the index associated with each unique vertex
        let mut unique_vertices: HashMap<Vertex, u32, BuildHasherDefault<XxHash32>> = Default::default();

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for model in &models
        {
            let mesh = &model.mesh;
            for index in &mesh.indices
            {
                let index = *index as usize;
                let vertex = Vertex
                {
                    position:
                    [
                        mesh.positions[3 * index],
                        mesh.positions[3 * index + 1],
                        mesh.positions[3 * index + 2],
                    ],
                    normal:
                    [
                        mesh.normals[3 * index],
                        mesh.normals[3 * index + 1],
                        mesh.normals[3 * index + 2],
                    ],
                    texcoord:
                    [
                        mesh.texcoords[2 * index],
                        1.0 - mesh.texcoords[2 * index + 1],
                    ],
                };

                // Get the index for the vertex
                let index =
                    {
                        // See if the vertex is already within the hashmap of unique vertices
                        if !unique_vertices.contains_key(&vertex)
                        {
                            // If it isn't, add it with the current current total # of vertices found
                            //  as its index
                            let len = vertices.len() as u32;
                            unique_vertices.insert(vertex, len);
                            vertices.push(vertex);
                            len
                        }
                        // If it already exists, return the assigned index
                        else { *unique_vertices.get(&vertex).unwrap() }
                    };
                indices.push(index);
            }
        }
        Ok(Mesh { vertices, indices })
    }
}