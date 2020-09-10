use crate::gfx::
{
    Context,
    GfxError,
    gl_object::
    {
        manager::{GlWrapperHandle, GlWrapperManager},
        traits::{GlObject, Bindable, Reloadable},
        buffer::Buffer,
        shader::shaderprogram::ShaderProgram,
    },
};
use std::
{
    sync::
    {
        Once,
        atomic::
        {
            AtomicU32,
            Ordering,
        }
    },
};
// THIS VARIABLE MUST NEVER BE CHANGED OUTSIDE OF `get_alignment`
// Using it in this way to cache the uniform buffer alignment is thread-safe as long
// as it is only changed once with INIT.call_once();
static mut ALIGNMENT: i32 = 0;
static INIT: Once = Once::new();

/// Get the uniform buffer offset alignment of the GPU
fn get_alignment(context: &Context) -> i32
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
fn align(context: &Context, size: i32) -> i32
{
    (size + get_alignment(&context) - 1) & (-get_alignment(&context))
}

/// Atomic counter for generating new uniform block bindings
static BLOCK_BINDINGS: AtomicU32 = AtomicU32::new(0);

pub struct UniformBuffer
{
    buffer: Buffer,

    vert_size: i32,
    frag_size: i32,

    vert_offset: i32,
    frag_offset: i32,

    vert_binding: Option<u32>,
    frag_binding: Option<u32>
}

impl UniformBuffer
{
    pub fn new(context: &Context, vert_size: i32, frag_size: i32, draw_type: u32) -> Result<UniformBuffer, GfxError>
    {
        let vert_size = align(&context, vert_size);
        // not necessary to align frag size since it isn't used as an offset
        //let frag_size = align(&context, frag_size);

        Ok(UniformBuffer
        {
            buffer:
            {
                let mut buffer = Buffer::new(&context, Context::UNIFORM_BUFFER)?;
                buffer.bind();
                buffer.buffer_data_raw(&vec![0u8; (vert_size + frag_size) as usize], draw_type);
                buffer
            },
            vert_size,
            frag_size,

            vert_offset: 0,
            frag_offset: vert_size,

            vert_binding: None,
            frag_binding: None
        })
    }

    /// Set `data` as the contents of the vertex shader uniform block
    pub fn buffer_vert_data<T>(&mut self, data: &[T])
    {
        self.buffer.buffer_sub_data(self.vert_offset, &data);
    }

    /// Set `data` as the contents of the vertex shader uniform block, starting at `offset`
    #[allow(dead_code)]
    pub fn buffer_vert_data_with_offset<T>(&mut self, offset: i32, data: &[T])
    {
        self.buffer.buffer_sub_data(self.vert_offset+offset, &data)
    }

    /// Set `data` as the contents of the fragment shader uniform block
    pub fn buffer_frag_data<T>(&mut self, data: &[T])
    {
        self.buffer.buffer_sub_data(self.frag_offset, &data);
    }

    /// Set `data` as the contents of the fragment shader uniform block, starting at `offset`
    #[allow(dead_code)]
    pub fn buffer_frag_data_with_offset<T>(&mut self, offset: i32, data: &[T])
    {
        self.buffer.buffer_sub_data(self.frag_offset+offset, &data);
    }

    /// Register a vertex shader uniform block of `block_name` from within `shader_program` to this uniform buffer
    pub fn add_vert_block(&mut self, shader_program: &mut ShaderProgram, block_name: &str) -> Result<(), GfxError>
    {
        if self.vert_binding == None
        {
            self.vert_binding = Some(BLOCK_BINDINGS.fetch_add(1, Ordering::Relaxed));
        }
        shader_program.add_uniform_block_binding(block_name, self.vert_binding.unwrap())?;
        self.buffer.bind();
        self.buffer.bind_range(self.vert_binding.unwrap(), self.vert_offset, self.vert_size);
        Ok(())
    }

    /// Register a fragment shader uniform block of `block_name` from within `shader_program` to this uniform buffer
    pub fn add_frag_block(&mut self, shader_program: &mut ShaderProgram, block_name: &str) -> Result<(), GfxError>
    {
        if self.frag_binding == None
        {
            self.frag_binding = Some(BLOCK_BINDINGS.fetch_add(1, Ordering::Relaxed));
        }
        shader_program.add_uniform_block_binding(block_name, self.frag_binding.unwrap())?;
        self.buffer.bind();
        self.buffer.bind_range(self.frag_binding.unwrap(), self.frag_offset, self.frag_size);
        Ok(())
    }
}

impl GlObject for UniformBuffer
{
}

impl Bindable for UniformBuffer
{
    fn bind(&self) { self.buffer.bind(); }
    fn unbind(&self) { self.buffer.unbind(); }
}

impl Reloadable for UniformBuffer
{
    fn reload(&mut self, context: &Context) -> Result<(), GfxError>
    {
        self.buffer.reload(&context)
    }
}

impl Drop for UniformBuffer
{
    fn drop(&mut self) {}
}