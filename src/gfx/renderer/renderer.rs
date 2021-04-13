use std::
{
    cell::RefMut,
};

use cgmath::Matrix4;
use crate::
{
    gfx::
    {
        Context,
        GfxError,
        gl_object::
        {
            manager::{GlObjectManager, GlObjectHandle},
            traits::GlObject,
            shader_program::ShaderProgram,
            uniform_buffer::UniformBuffer,
            vertex_array::VertexArray,
            texture::Texture2d
        },
    },
    resource::manager::ResourceManager,
};

/// Render info data transfer object
/// This contains information needed to render an object
pub struct RenderDto
{
    pub tex_handle: GlObjectHandle,
    pub vert_arr_handle: GlObjectHandle,
    pub num_indices: i32,
}
/// Node in the scene to be rendered
/// The second parameter is a model matrix and the third parameter is any child nodes
pub struct Node<'a>(pub &'a RenderDto, pub &'a Matrix4<f32>, pub Option<Vec<Node<'a>>>);

/// Scene Renderer
pub struct Renderer
{
    shader_program_handle: GlObjectHandle,
    uniform_buff_handle: GlObjectHandle,
}

impl Renderer
{
    /// Create a new Renderer instance
    pub fn new(context: &Context, gl_manager: &mut GlObjectManager, resource_manager: &ResourceManager) -> Result<Renderer, GfxError>
    {
        // Get shader source code and read into string
        let vert_shader = String::from_utf8(resource_manager.get_by_name("texture_vert.glsl")
            .ok_or_else(|| GfxError::Other("texture_vert.glsl not available by name in resource manager".to_string()))?.clone())
            .or_else(|err| Err(GfxError::Other(format!("Error reading texture_vert.glsl into string: {}", err.to_string()))))?;
        let frag_shader = String::from_utf8(resource_manager.get_by_name("texture_frag.glsl").
            ok_or_else(|| GfxError::Other("texture_frag.glsl not available by name in resource manager".to_string()))?.clone())
            .or_else(|err| Err(GfxError::Other(format!("Error reading texture_frag.glsl into string: {}", err.to_string()))))?;

        let renderer = Renderer
        {
            shader_program_handle: gl_manager.insert_shader_program(
                ShaderProgram::new(&context, Some(vert_shader), Some(frag_shader))?,
            ),
            uniform_buff_handle: gl_manager.insert_uniform_buffer(
                UniformBuffer::new(&context, std::mem::size_of::<Matrix4<f32>>() as i32, 0, Context::DYNAMIC_DRAW)?
            ),
        };
        // Setup the renderer's uniform buffer
        ShaderProgram::bind(gl_manager, renderer.shader_program_handle);
        UniformBuffer::bind(gl_manager, renderer.uniform_buff_handle);
        let mut shader_program = gl_manager.get_mut_shader_program(renderer.shader_program_handle).expect("renderer shader program");
        let mut uniform_buffer = gl_manager.get_mut_uniform_buffer(renderer.uniform_buff_handle).expect("renderer uniform buffer");
        // Set the shader sampler2d to TEXTURE0
        shader_program.set_uniform_i32("tex", &[0])?;
        uniform_buffer.add_vert_block(&mut shader_program, "VertData")?;

        Ok(renderer)
    }

    /// Render's a scene
    /// `context` is the current rendering context
    /// `manager` is the object manager for the `RenderDto`s in `nodes`
    /// `proj_view_mat` is the projection-view matrix
    /// `nodes` is the scene graph to render
    pub fn render<'a>(&self, context: &Context, manager: &GlObjectManager, proj_view_mat: Matrix4<f32>, nodes: &Vec<Node<'a>>)
    {
        ShaderProgram::bind(manager, self.shader_program_handle);
        UniformBuffer::bind(manager, self.uniform_buff_handle);

        let mut uniform_buffer: RefMut<UniformBuffer> = manager.get_mut_uniform_buffer(self.uniform_buff_handle).expect("renderer uniform buffer");
        // context.active_texture(Context::TEXTURE0);
        manager.set_active_texture(&context, Context::TEXTURE0);

        // Iterate over scene graph
        for parent in nodes
        {
            // Render children
            if let Some(children) = &parent.2
            {
                for child in children
                {
                    // Multiply the parent matrix into the child's model
                    // matrix and buffer it
                    {
                        let mvp = proj_view_mat * parent.1 * child.1;
                        let buff: &[f32; 16] = mvp.as_ref();
                        uniform_buffer.buffer_vert_data(buff);
                    }
                    // Bind the appropriate resources and draw the object
                    Texture2d::bind(manager, child.0.tex_handle);
                    VertexArray::bind(manager, child.0.vert_arr_handle);
                    context.draw_elements_with_i32(Context::TRIANGLES, child.0.num_indices, Context::UNSIGNED_INT, 0);
                }
            }

            // Buffer the model matrix
            {
                let mvp = proj_view_mat * parent.1;
                let buff: &[f32; 16] = mvp.as_ref();
                uniform_buffer.buffer_vert_data(buff);
            }
            // Bind the appropriate resources and draw the object
            Texture2d::bind(manager, parent.0.tex_handle);
            VertexArray::bind(manager, parent.0.vert_arr_handle);
            context.draw_elements_with_i32(Context::TRIANGLES, parent.0.num_indices, Context::UNSIGNED_INT, 0);
        }
    }
}