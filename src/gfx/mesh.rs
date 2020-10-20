use std::
{
    io,
    hash::{Hash, Hasher, BuildHasherDefault},
    collections::HashMap,
};
use twox_hash::XxHash32;
use crate::gfx::GfxError;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
#[repr(C)]
pub struct Vertex
{
    pub pos: [f32; 3],
    pub tex: [f32; 2]
}

impl Hash for Vertex
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {

    }
}

pub struct Mesh
{
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Mesh
{
    pub fn load_stream<R: io::Read>(stream: R) -> io::Result<Mesh>
    {
        let mut reader = io::BufReader::new(stream);
        let (models, _materials) = tobj::load_obj_buf(&reader, true, |p|
            {
                // TODO: Load materials
                unimplemented!();
            }).expect("obj loaded");

        let mut unique_vertices: HashMap<Vertex, u32, BuildHasherDefault<XxHash32>> = Default::default();

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for model in &models
        {
            let mesh = &model.mesh;
            for index in &mesh.indices
            {
                let index = index as usize;
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
                        mesh.texcoords[3 * index],
                        mesh.texcoords[3 * index + 1],
                    ],
                };

                let index =
                    {
                        if !unique_vertices.contains_key(&vertex)
                        {
                            let len = vertices.len() as u32;
                            unique_vertices.insert(vertex, len);
                            vertices.push(vertex);
                            len as u32
                        }
                        else { unique_vertices.get(&vertex) }
                    };
                indices.push(index);
            }
        }
        Ok(Mesh { vertices, indices })
    }
}