//! All things graphics related

use gen_vec::Index;
use crate::gfx::shader::shaderprogram::ShaderType;

pub type Context = web_sys::WebGl2RenderingContext;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum GfxError
{
    /// Errors from glGetErrors()
    GlErrors(Vec<GlError>),

    /// Invalid `Index` handle
    InvalidHandle(Index),

    /// All shader `src` parameters are `None`
    NoShaderSource(String),
    /// Error creating a new shader program
    ShaderProgramCreationError(String),
    /// Error creating a new shader fragment
    ShaderCreationError(ShaderType, String),
    /// Error compiling shader fragment
    ShaderCompilationError(ShaderType, String),
    /// Error linking shader fragments to shader program
    ShaderProgramLinkingError(String),
    /// Invalid block name for uniform buffer binding
    InvalidUniformBlockName(String),
    /// Invalid name for a regular uniform variable
    InvalidUniformName(String),

    /// Error creating a new buffer
    BufferCreationError(String),

    /// Error creating a new vertex array
    VertexArrayCreationError(String),

    /// Error creating a new texture
    TextureCreationError(String),

    RenderLoopAlreadyRunning,
    RenderLoopNotRunning,
    RenderLoopAlreadyCleanedUp,

    #[allow(dead_code)]
    /// Anything else
    Other(String)
}
impl std::fmt::Display for GfxError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum GlError
{
    NoError,
    InvalidEnum,
    InvalidValue,
    InvalidOperation,
    InvalidFramebufferOperation,
    OutOfMemory,
    ContextLostWebGl,
    Unknown
}
impl From<u32> for GlError
{
    fn from(error_enum: u32) -> Self
    {
        match error_enum
        {
            Context::NO_ERROR => GlError::NoError,
            Context::INVALID_ENUM => GlError::InvalidEnum,
            Context::INVALID_VALUE => GlError::InvalidValue,
            Context::INVALID_OPERATION => GlError::InvalidOperation,
            Context::INVALID_FRAMEBUFFER_OPERATION => GlError::InvalidFramebufferOperation,
            Context::OUT_OF_MEMORY => GlError::OutOfMemory,
            Context::CONTEXT_LOST_WEBGL => GlError::ContextLostWebGl,
            _ => GlError::Unknown
        }
    }
}
impl Into<u32> for GlError
{
    fn into(self) -> u32
    {
        match self
        {
            GlError::NoError => Context::NO_ERROR,
            GlError::InvalidEnum => Context::INVALID_ENUM,
            GlError::InvalidValue => Context::INVALID_VALUE,
            GlError::InvalidOperation => Context::INVALID_OPERATION,
            GlError::InvalidFramebufferOperation => Context::INVALID_FRAMEBUFFER_OPERATION,
            GlError::OutOfMemory => Context::OUT_OF_MEMORY,
            GlError::ContextLostWebGl => Context::CONTEXT_LOST_WEBGL,
            _ => 0
        }
    }
}
impl std::fmt::Display for GlError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "{:?}", self)
    }
}
pub fn gl_get_errors(context: &Context) -> GfxError
{
    let mut error: GlError = context.get_error().into();
    if error != GlError::NoError
    {
        let mut errors = vec![];

        while error != GlError::NoError
        {
            errors.push(error);
            error = context.get_error().into();
        }
        GfxError::GlErrors(errors)
    }
    else
    {
        GfxError::GlErrors(vec![GlError::NoError])
    }
}

/// Gets a new context from the canvas
/// Returns the Context within an Rc to allow the context to be stored and referenced
/// by GlObjects
pub fn new_context(canvas: &web_sys::HtmlCanvasElement) -> Result<Context, &'static str>
{
    match canvas.get_context("webgl2")
    {
        Ok(Some(context)) =>
            {
                use wasm_bindgen::JsCast;
                Ok(context.dyn_into::<Context>().or(Err("failed to cast webgl2 context into WebGl2RenderingContext"))?)
            },
        _ => Err("failed to get webgl2 context from canvas")
    }
}

pub mod gl_object;
pub mod render_loop;
pub mod renderer;
