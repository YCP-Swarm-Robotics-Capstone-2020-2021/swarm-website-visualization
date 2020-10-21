use std::
{
    io,
    hash::{Hash, Hasher, BuildHasherDefault},
    collections::HashMap,
};
use twox_hash::XxHash32;
use serde::{Serialize, Deserialize};
use crate::gfx::GfxError;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Vertex
{
    pub pos: [f32; 3],
    pub tex: [f32; 2]
}

#[derive(Debug, Clone)]
pub struct Mesh
{
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh
{
    pub fn from_reader<R: io::Read>(reader: R) -> io::Result<Mesh>
    {
        let mut bufreader = io::BufReader::new(reader);
        let (models, _materials) = tobj::load_obj_buf(&mut bufreader, true, |p|
            {
                // TODO: Load materials
                unimplemented!();
            }).expect("obj loaded");

        let mut serializer = flexbuffers::FlexbufferSerializer::new();

        // Keep track of the index associated with each unique vertex
        let mut unique_vertices: HashMap<Vec<u8>, u32, BuildHasherDefault<XxHash32>> = Default::default();

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
                    pos:
                    [
                        mesh.positions[3 * index],
                        mesh.positions[3 * index + 1],
                        mesh.positions[3 * index + 2],
                    ],
                    tex:
                    [
                        mesh.texcoords[2 * index],
                        1.0 - mesh.texcoords[2 * index + 1],
                    ],
                };

                // Find the index for the vertex
                let index =
                    {
                        // TODO: Find out a better way to do this
                        //       A collection of unique vertices needs to be maintained, where the
                        //       vertex is a key for its index. Floating point values
                        //       can't/shouldn't be hashed, but there is probably a better way to
                        //       do this than serializing the structure then hashing it

                        // Serialize the vertex and see if its already within the hashmap of unique vertices
                        vertex.serialize(&mut serializer).expect("vertex serialized");
                        let view_vec = serializer.view().to_vec();
                        if !unique_vertices.contains_key(&view_vec)
                        {
                            // If it isn't, add it with the current current total # of vertices found
                            //  as it's index
                            let len = vertices.len() as u32;
                            unique_vertices.insert(view_vec, len);
                            vertices.push(vertex);
                            len as u32
                        }
                        // If it already exists, return the assigned index
                        else { *unique_vertices.get(&view_vec)
                            .expect("key should be presented since contains_key was true to reach here") }
                    };
                indices.push(index);
            }
        }
        Ok(Mesh { vertices, indices })
    }
}