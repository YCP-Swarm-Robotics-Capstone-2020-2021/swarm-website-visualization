use std::
{
    rc::Rc,
    cell::{RefCell, RefMut},
};

use cgmath::Matrix4;
use crate::gfx::
{
    Context,
    GfxError,
    gl_object::
    {
        manager::{GlObjectManager, GlObjectHandle},
        traits::GlObject,
        shader_program::{ShaderProgram, shader_source},
        uniform_buffer::UniformBuffer,
        vertex_array::VertexArray,
        texture::Texture2d
    },
};

pub struct RenderDto
{
    pub model_mat: [f32; 16],
    pub tex_handle: GlObjectHandle,
    pub vert_arr_handle: GlObjectHandle,
    pub num_indices: i32,
}
pub struct Node<'a>(RenderDto, &'a [Node<'a>]);

pub struct Renderer2D
{
    context: Rc<RefCell<Context>>,
    shader_program_handle: GlObjectHandle,
    uniform_buff_handle: GlObjectHandle,
}

impl Renderer2D
{
    pub fn new(context: &Rc<RefCell<Context>>, manager: &mut GlObjectManager) -> Result<Renderer2D, GfxError>
    {
        let renderer = Renderer2D
        {
            context: context.clone(),
            shader_program_handle: manager.insert_shader_program(
                ShaderProgram::new(&context.borrow(), Some(shader_source::TEXTURE_VERT.to_string()), Some(shader_source::TEXTURE_FRAG.to_string()))?,
            ),
            uniform_buff_handle: manager.insert_uniform_buffer(
                UniformBuffer::new(&context.borrow(), std::mem::size_of::<Matrix4<f32>>() as i32, 0, Context::DYNAMIC_DRAW)?
            ),
        };
        ShaderProgram::bind(manager, renderer.shader_program_handle);
        ShaderProgram::bind(manager, renderer.uniform_buff_handle);
        let mut shader_program = manager.get_mut_shader_program(renderer.shader_program_handle).expect("renderer2d shader program");
        let mut uniform_buffer = manager.get_mut_uniform_buffer(renderer.uniform_buff_handle).expect("renderer2d uniform buffer");
        shader_program.set_uniform_i32("tex", &[0])?;
        uniform_buffer.add_vert_block(&mut shader_program, "VertData")?;

        Ok(renderer)
    }

    pub fn render<'a>(&self, manager: &GlObjectManager, nodes: &[Node<'a>])
    {
        ShaderProgram::bind(manager, self.shader_program_handle);
        UniformBuffer::bind(manager, self.uniform_buff_handle);

        let mut uniform_buffer: RefMut<UniformBuffer> = manager.get_mut_uniform_buffer(self.uniform_buff_handle).expect("renderer2d uniform buffer");
        borrow!(self.context);

        for node in nodes
        {
            uniform_buffer.buffer_vert_data(&node.0.model_mat);

            context.active_texture(Context::TEXTURE0);
            Texture2d::bind(manager, node.0.tex_handle);

            VertexArray::bind(manager, node.0.vert_arr_handle);

            context.draw_elements_with_i32(Context::TRIANGLES, node.0.num_indices, Context::UNSIGNED_INT, 0);
        }
    }
}