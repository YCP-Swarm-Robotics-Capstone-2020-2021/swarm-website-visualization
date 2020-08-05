use std::
{
    sync::
    {
        Once,
        atomic::{AtomicU32, Ordering},
    },
    rc::Rc
};
use crate::gfx::
{
    Context,
    gl_object::GLObject,
    buffer::Buffer,
    shader::shaderprogram::ShaderProgram
};

// THIS VARIABLE MUST NEVER BE CHANGED OUTSIDE OF `get_alignment`
// Using it in this way to cache the uniform buffer alignment is thread-safe as long
// as it is only changed once with INIT.call_once();
static mut ALIGNMENT: i32 = 0;
static INIT: Once = Once::new();

/// Get the uniform buffer offset alignment of the GPU
fn get_alignment(context: &Rc<Context>) -> i32
{
    unsafe
        {
            INIT.call_once(||
                {
                    ALIGNMENT = context.get_parameter(Context::UNIFORM_BUFFER_OFFSET_ALIGNMENT)
                        .expect("Uniform Buffer Alignment").as_f64().expect("Uniform Buffer Alignment as f64") as i32;
                });
            ALIGNMENT
        }
}

/// Align `size` to the `UNIFORM_BUFFER_OFFSET_ALIGNMENT`
fn align(context: &Rc<Context>, size: i32) -> i32
{
    (size + get_alignment(&context) - 1) & (-get_alignment(&context))
}

// Atomic counter for creating new block bindings
static BLOCK_BINDING: AtomicU32 = AtomicU32::new(1);

/// Macro to implement buffer_<block>_data_<type> functions for each block and primitive type
/// i.e. buffer_vert_data_f32
macro_rules! buffer_fn
{
    ($type:ty, $($block:tt),+) =>
    {paste::paste!
    {
        $(
            #[allow(dead_code)]
            pub fn [<buffer_ $block _data_ $type>](&self, data: &[$type])
            {
                self.buffer.[<buffer_sub_data_ $type>](self.[<$block _offset>], &data);
            }
        )+
    }}
}

pub struct UniformBuffer
{
    buffer: Buffer,
    //context: Rc<Context>,

    vert_size: i32,
    frag_size: i32,

    vert_offset: i32,
    frag_offset: i32,

    vert_binding: u32,
    frag_binding: u32
}

impl UniformBuffer
{
    pub fn new(context: &Rc<Context>, vert_size: i32, frag_size: i32, draw_type: u32) -> Result<UniformBuffer, String>
    {
        let vert_size = align(&context, vert_size);
        let frag_size = align(&context, frag_size);

        Ok(UniformBuffer
        {
            buffer:
                {
                    // Create a new buffer and fill it with empty data to size it
                    let buffer = Buffer::new(&context, Context::UNIFORM_BUFFER)?;
                    buffer.bind();
                    buffer.buffer_data_f32(&vec![0f32; (vert_size + frag_size) as usize], draw_type);
                    buffer
                },
            //context: Rc::clone(context),
            vert_size,
            frag_size,

            vert_offset: 0,
            frag_offset: vert_size,

            vert_binding: 0,
            frag_binding: 0
        })
    }

    /// Internal buffer
    pub fn buffer(&self) -> &Buffer
    {
        &self.buffer
    }

    /// Add a uniform block from the shader to this uniform buffer
    fn add_uniform_block(buffer: &Buffer, shader_program: &ShaderProgram, block_name: &str, offset: i32, size: i32) -> u32
    {
        let binding = BLOCK_BINDING.fetch_add(1, Ordering::Relaxed);
        shader_program.add_uniform_block_binding(block_name, binding);
        buffer.bind_range(binding, offset, size);
        binding
    }

    /// Add a uniform block from the vertex shader to this uniform buffer
    pub fn add_vert_block(&mut self, shader_program: &ShaderProgram, block_name: &str)
    {
        self.vert_binding = UniformBuffer::add_uniform_block(&self.buffer(), &shader_program, &block_name, self.vert_offset, self.vert_size);
    }

    /// Add a uniform block from the fragment shader to this uniform buffer
    pub fn add_frag_block(&mut self, shader_program: &ShaderProgram, block_name: &str)
    {
        self.frag_binding = UniformBuffer::add_uniform_block(&self.buffer, &shader_program, &block_name, self.frag_offset, self.frag_size);
    }

    buffer_fn!(f32, vert, frag);
    buffer_fn!(i32, vert, frag);
    buffer_fn!(u32, vert, frag);

}

impl GLObject for UniformBuffer
{
    fn bind(&self)
    {
        self.buffer.bind();
    }

    fn unbind(&self)
    {
        self.buffer.unbind();
    }
}

impl Drop for UniformBuffer
{
    fn drop(&mut self)
    {

    }
}