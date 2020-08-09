use std::
{
    sync::Once,
    rc::Rc,
    collections::HashMap
};
use web_sys::{WebGlProgram, WebGlShader};
use twox_hash::XxHash32;
use gen_vec::{Index, closed::ClosedGenVec};
use crate::gfx::
{
    Context,
    GfxError,
    GlManager,
    ManagedGlItem,
    gl_get_errors,
    gl_object::GLObject,
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
    ($type:ty, $($block:tt),+) =>
    {paste::paste!
    {
        $(
            #[allow(dead_code)]
            pub fn [<buffer_ $block _uniform_data_ $type>](&mut self, handle: Index, data: &[$type]) -> Result<(), GfxError>
            {
                let mut uniform_buffer = self.uniform_buffers.get_mut(handle).ok_or_else(|| GfxError::InvalidHandle(handle))?;
                uniform_buffer.buffer.[<buffer_sub_data_ $type>](uniform_buffer.[<$block _offset>], &data);
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
                self.binding_counter += 1;
                let binding = self.binding_counter;

                let index = self.context.get_uniform_block_index(&self.internal, block_name);
                if(index == Context::INVALID_INDEX)
                {
                    Err(GfxError::InvalidUniformBlockName(String::from(block_name)))
                }
                else
                {
                    self.context.uniform_block_binding(&self.internal, index, binding);
                    let uniform_buffer = self.uniform_buffers.get_mut(handle).ok_or_else(|| GfxError::InvalidHandle(handle))?;
                    uniform_buffer.buffer.bind_range(binding, uniform_buffer.[<$block _offset>], uniform_buffer.[<$block _size>]);
                    uniform_buffer.[<$block _binding>] = binding;
                    Ok(())
                }
            }
        )+
    }}
}

pub struct UniformBuffer
{
    buffer: Buffer,
    context: Rc<Context>,

    vert_size: i32,
    frag_size: i32,

    vert_offset: i32,
    frag_offset: i32,

    vert_binding: u32,
    frag_binding: u32
}

impl GLObject for UniformBuffer
{
    fn bind(&self) { self.buffer.bind(); }
    fn unbind(&self) { self.buffer.unbind(); }
    fn reload(&mut self, context: &Rc<Context>) -> Result<(), GfxError>
    {
        self.context = Rc::clone(&context);
        self.buffer.reload(&context)
    }
}

impl Drop for UniformBuffer
{
    fn drop(&mut self) {}
}

pub struct ShaderProgram
{
    internal: WebGlProgram,
    context: Rc<Context>,
    binding_counter: u32,

    vert_src: Option<String>,
    frag_src: Option<String>,
    uniform_block_bindings: HashMap<u32, String, std::hash::BuildHasherDefault<XxHash32>>,
    uniform_buffers: ClosedGenVec<UniformBuffer>
}

impl ShaderProgram
{
    fn new_program(context: &Context) -> Result<WebGlProgram, GfxError>
    {
        context.create_program().ok_or_else(|| GfxError::ShaderProgramCreationError(gl_get_errors(&context).to_string()))
    }

    pub fn new(manager: &mut GlManager, vert_src: Option<String>, frag_src: Option<String>) -> Result<(Index, ManagedGlItem), GfxError>
    {
        if vert_src.is_none() && frag_src.is_none()
        {
            return Err(GfxError::NoShaderSource("At least one shader source must be present".to_string()))
        }
        let program = ShaderProgram
        {
            internal: ShaderProgram::new_program(&manager.context())?,
            context: Rc::clone(&manager.context()),
            binding_counter: 0,
            vert_src: vert_src,
            frag_src: frag_src,
            uniform_block_bindings: Default::default(),
            uniform_buffers: ClosedGenVec::new()
        };
        program.compile(&program.vert_src, &program.frag_src)?;
        Ok(manager.add_gl_object(program))
    }

    /// Compiles the shader fragments and attaches them to the shader program
    fn compile(&self, vert_src: &Option<String>, frag_src: &Option<String>) -> Result<(), GfxError>
    {

        let vert = if let Some(src) = vert_src
        {
            Some(self.compile_shader(src, ShaderType::VertexShader)?)
        } else { None };

        let frag = if let Some(src) = frag_src
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

    pub fn new_uniform_buffer(&mut self, manager: &mut GlManager, vert_size: i32, frag_size: i32, draw_type: u32) -> Result<(Index, ManagedGlItem), GfxError>
    {
        let vert_size = align(&manager.context(), vert_size);
        let frag_size = align(&manager.context(), frag_size);

        let index = self.uniform_buffers.insert(
            UniformBuffer
                {
                    buffer:
                    {
                        // Create a new buffer and fill it with empty data to size it
                        let (handle, buffer) = manager.add_gl_object(Buffer::new(&context, Context::UNIFORM_BUFFER)?);
                        {
                            let mut buffer = buffer.write().unwrap();
                            buffer.bind();
                            buffer.buffer_data_f32(&vec![0f32; (vert_size + frag_size) as usize], draw_type);
                        }
                        item
                    },
                    context: Rc::clone(context),
                    vert_size,
                    frag_size,

                    vert_offset: 0,
                    frag_offset: vert_size,

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

    pub fn delete_uniform_buffer(&mut self, handle: Index) -> Result<(), GfxError>
    {
        self.uniform_buffers.remove(handle).ok_or(GfxError::InvalidHandle)?;
        Ok(())
    }

    pub fn bind_uniform_buffer(&self, handle: Index) -> Result<(), GfxError>
    {
        self.get_uniform_buffer(handle).ok_or(GfxError::InvalidHandle)?.bind();
        Ok(())
    }

    #[allow(dead_code)]
    pub fn unbind_uniform_buffer(&self, handle: Index) -> Result<(), GfxError>
    {
        self.get_uniform_buffer(handle).ok_or(GfxError::InvalidHandle)?.unbind();
        Ok(())
    }

    add_uniform_block_fn!(vert, frag);

    buffer_fn!(f32, vert, frag);
    buffer_fn!(i32, vert, frag);
    buffer_fn!(u32, vert, frag);
}

impl GLObject for ShaderProgram
{
    fn bind(&self) { self.context.use_program(Some(&self.internal)); }
    fn unbind(&self) { self.context.use_program(None); }
    fn reload(&mut self, context: &Rc<Context>) -> Result<(), GfxError>
    {
        self.context = Rc::clone(context);
        self.internal = ShaderProgram::new_program(&self.context)?;
        self.compile(self.vert_src, self.frag_src)?;
        for (binding, block_name) in self.uniform_block_bindings.drain()
        {
            //self.add
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