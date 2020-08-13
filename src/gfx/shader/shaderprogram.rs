use std::
{
    sync::Once,
    rc::Rc,
};
use web_sys::{WebGlProgram, WebGlShader};
use gen_vec::{Index, closed::ClosedGenVec};
use crate::gfx::
{
    Context,
    GfxError,
    gl_get_errors,
    gl_object::GlObject,
    buffer::Buffer,
};


#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum ShaderType
{
    VertexShader,
    FragmentShader,
    UnknownShader,
}

impl std::fmt::Display for ShaderType
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "{}", self)
    }
}
impl From<u32> for ShaderType
{
    fn from(webgl_enum: u32) -> Self
    {
        match webgl_enum
        {
            Context::VERTEX_SHADER => ShaderType::VertexShader,
            Context::FRAGMENT_SHADER => ShaderType::FragmentShader,
            _ => ShaderType::UnknownShader
        }
    }
}
impl Into<u32> for ShaderType
{
    fn into(self) -> u32
    {
        match self
        {
            ShaderType::VertexShader => Context::VERTEX_SHADER,
            ShaderType::FragmentShader => Context::FRAGMENT_SHADER,
            ShaderType::UnknownShader => 0
        }
    }
}

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
                    crate::log(format!("{}", ALIGNMENT).as_str());
                });
            ALIGNMENT
        }
}

/// Align `size` to the `UNIFORM_BUFFER_OFFSET_ALIGNMENT`
fn align(context: &Context, size: i32) -> i32
{
    (size + get_alignment(&context) - 1) & (-get_alignment(&context))
}

/// Macro to implement buffer_<block>_data_<type> functions for each block and primitive type
/// i.e. buffer_vert_data_f32
macro_rules! buffer_fn
{
    ($($block:tt),+) =>
    {paste::paste!
    {
        $(
            #[allow(dead_code)]
            pub fn [<buffer_ $block _uniform_data>]<T>(&mut self, handle: Index, data: &[T]) -> Result<(), GfxError>
            {
                let uniform_buffer = self.uniform_buffers.get_mut(handle).ok_or_else(|| GfxError::InvalidHandle(handle))?;
                uniform_buffer.buffer.buffer_sub_data(uniform_buffer.[<$block _offset>], &data);
                Ok(())
            }
        )+
    }}
}

macro_rules! add_uniform_block_fn
{
    ($($block:tt),+) =>
    {paste::paste!
    {
        $(
            pub fn [<add_ $block _uniform_block>](&mut self, handle: Index, block_name: &str) -> Result<(), GfxError>
            {
                let uniform_buffer = self.uniform_buffers.get_mut(handle).ok_or_else(|| GfxError::InvalidHandle(handle))?;
                let index = self.context.get_uniform_block_index(&self.internal, block_name);

                if index == Context::INVALID_INDEX
                {
                    Err(GfxError::InvalidUniformBlockName(String::from(block_name)))
                }
                else
                {
                    uniform_buffer.[<$block _binding>] = self.binding_counter;
                    uniform_buffer.[<$block _name>] = Some(String::from(block_name));

                    self.context.uniform_block_binding(&self.internal, index, uniform_buffer.[<$block _binding>]);

                    uniform_buffer.buffer.bind();
                    uniform_buffer.buffer.bind_range(uniform_buffer.[<$block _binding>], uniform_buffer.[<$block _offset>], uniform_buffer.[<$block _size>]);

                    self.binding_counter += 1;
                    Ok(())
                }
            }
        )+
    }};
}

pub struct UniformBuffer
{
    pub buffer: Buffer,

    vert_size: i32,
    frag_size: i32,

    vert_offset: i32,
    frag_offset: i32,

    vert_name: Option<String>,
    frag_name: Option<String>,

    vert_binding: u32,
    frag_binding: u32
}

pub struct ShaderProgram
{
    internal: WebGlProgram,
    context: Rc<Context>,
    binding_counter: u32,

    vert_src: Option<String>,
    frag_src: Option<String>,
    uniform_buffers: ClosedGenVec<UniformBuffer>
}

impl ShaderProgram
{
    fn new_program(context: &Context) -> Result<WebGlProgram, GfxError>
    {
        context.create_program().ok_or_else(|| GfxError::ShaderProgramCreationError(gl_get_errors(&context).to_string()))
    }

    pub fn new(context: &Rc<Context>, vert_src: Option<String>, frag_src: Option<String>) -> Result<ShaderProgram, GfxError>
    {
        if vert_src.is_none() && frag_src.is_none()
        {
            return Err(GfxError::NoShaderSource("At least one shader source must be present".to_string()))
        }
        let program = ShaderProgram
        {
            internal: ShaderProgram::new_program(&context)?,
            context: Rc::clone(&context),
            binding_counter: 0,
            vert_src: vert_src,
            frag_src: frag_src,
            uniform_buffers: ClosedGenVec::new()
        };
        program.compile()?;
        Ok(program)
    }

    /// Compiles the shader fragments and attaches them to the shader program
    fn compile(&self) -> Result<(), GfxError>
    {

        let vert = if let Some(src) = &self.vert_src
        {
            Some(self.compile_shader(src, ShaderType::VertexShader)?)
        } else { None };

        let frag = if let Some(src) = &self.frag_src
        {
            Some(self.compile_shader(src, ShaderType::FragmentShader)?)
        } else { None };

        self.context.link_program(&self.internal);

        if self.context.get_program_parameter(&self.internal, Context::LINK_STATUS).as_bool().unwrap_or(false)
        {
            self.context.delete_shader(vert.as_ref());
            self.context.delete_shader(frag.as_ref());
            Ok(())
        }
        else
        {
            let info_log = self.context.get_program_info_log(&self.internal)
                .unwrap_or_else(|| format!("Error getting shader program info logs after linking. GlErrors: {}", gl_get_errors(&self.context)).to_string());
            Err(GfxError::ShaderProgramLinkingError(info_log))
        }
    }

    /// Compiles a shader fragment
    fn compile_shader(&self, src: &String, shader_type: ShaderType) -> Result<WebGlShader, GfxError>
    {
        let shader = self.context.create_shader(shader_type.into())
            .ok_or(GfxError::ShaderCreationError(shader_type, gl_get_errors(&self.context).to_string()))?;
        self.context.shader_source(&shader, &src.as_str());
        self.context.compile_shader(&shader);
        self.context.attach_shader(&self.internal, &shader);

        if self.context.get_shader_parameter(&shader, Context::COMPILE_STATUS).as_bool().unwrap_or(false)
        {
            Ok(shader)
        }
        else
        {
            let info_log = self.context.get_shader_info_log(&shader)
                .unwrap_or_else(|| format!("Error getting shader compilation info log. GlErrors: {}", gl_get_errors(&self.context)).to_string());
            Err(GfxError::ShaderCompilationError(shader_type, info_log))
        }
    }

    pub fn new_uniform_buffer(&mut self, context: &Rc<Context>, vert_size: i32, frag_size: i32, draw_type: u32) -> Result<Index, GfxError>
    {
        let vert_size = align(&context, vert_size);
        // not necessary to align frag size since it isn't used as an offset
        //let frag_size = align(&context, frag_size);

        let index = self.uniform_buffers.insert(
            UniformBuffer
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

                    vert_name: None,
                    frag_name: None,

                    vert_binding: 0,
                    frag_binding: 0
                }
        );

        Ok(index)
    }

    #[allow(dead_code)]
    pub fn get_uniform_buffer(&self, handle: Index) -> Option<&UniformBuffer>
    {
        self.uniform_buffers.get(handle)
    }

    #[allow(dead_code)]
    pub fn get_uniform_buffer_mut(&mut self, handle: Index) -> Option<&mut UniformBuffer>
    {
        self.uniform_buffers.get_mut(handle)
    }

    #[allow(dead_code)]
    pub fn delete_uniform_buffer(&mut self, handle: Index) -> Result<(), GfxError>
    {
        self.uniform_buffers.remove(handle).ok_or_else(|| GfxError::InvalidHandle(handle))?;
        Ok(())
    }

    pub fn bind_uniform_buffer(&self, handle: Index) -> Result<(), GfxError>
    {
        self.get_uniform_buffer(handle).ok_or_else(|| GfxError::InvalidHandle(handle))?.buffer.bind();
        Ok(())
    }

    #[allow(dead_code)]
    pub fn unbind_uniform_buffer(&self, handle: Index) -> Result<(), GfxError>
    {
        self.get_uniform_buffer(handle).ok_or_else(|| GfxError::InvalidHandle(handle))?.buffer.unbind();
        Ok(())
    }

    add_uniform_block_fn!(vert, frag);
    buffer_fn!(vert, frag);
}

impl GlObject for ShaderProgram
{
    fn bind(&self) { self.context.use_program(Some(&self.internal)); }
    fn unbind(&self) { self.context.use_program(None); }
    fn reload(&mut self, context: &Rc<Context>) -> Result<(), GfxError>
    {
        self.context = Rc::clone(context);
        self.internal = ShaderProgram::new_program(&self.context)?;
        self.compile()?;
        self.bind();

        for (_, uniform_buffer) in &mut self.uniform_buffers
        {
            uniform_buffer.buffer.reload(&self.context)?;
            if let Some(block_name) = &uniform_buffer.vert_name
            {
                let index = self.context.get_uniform_block_index(&self.internal, block_name.as_str());
                self.context.uniform_block_binding(&self.internal, index, uniform_buffer.vert_binding);
                //uniform_buffer.buffer.bind();
                //uniform_buffer.buffer.bind_range(uniform_buffer.vert_binding, uniform_buffer.vert_offset, uniform_buffer.vert_size);
            }
            if let Some(block_name) = &uniform_buffer.frag_name
            {
                let index = self.context.get_uniform_block_index(&self.internal, block_name.as_str());
                self.context.uniform_block_binding(&self.internal, index, uniform_buffer.frag_binding);
                //uniform_buffer.buffer.bind();
                //uniform_buffer.buffer.bind_range(uniform_buffer.frag_binding, uniform_buffer.frag_offset, uniform_buffer.frag_size);
            }
        }
        Ok(())
    }
}

impl Drop for ShaderProgram
{
    fn drop(&mut self)
    {
        self.context.delete_program(Some(&self.internal));
    }
}