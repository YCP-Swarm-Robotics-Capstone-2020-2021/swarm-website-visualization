pub type Context = web_sys::WebGl2RenderingContext;

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

#[derive(Debug, Clone)]
pub struct GlErrors(std::vec::Vec<GlError>);
impl std::fmt::Display for GlErrors
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        for error in &self.0
        {
            write!(f, "{}, ", error)?
        }
        Ok(())
    }
}

pub fn gl_get_errors(context: &Context) -> GlErrors
{
    let mut error: GlError = context.get_error().into();
    if error != GlError::NoError
    {
        let mut errors = GlErrors(vec![]);

        while error != GlError::NoError
        {
            errors.0.push(error);
            error = context.get_error().into();
        }
        errors
    }
    else
    {
        GlErrors(vec![GlError::NoError])
    }
}
use std::rc::Rc;

pub fn new_context(canvas: &web_sys::HtmlCanvasElement) -> Result<Rc<Context>, &'static str>
{
    match canvas.get_context("webgl2")
    {
        Ok(Some(context)) =>
            {
                use wasm_bindgen::JsCast;
                Ok(Rc::new(context.dyn_into::<Context>().or(Err("failed to cast webgl2 context into WebGl2RenderingContext"))?))
            },
        _ => Err("failed to get webgl2 context from canvas")
    }
}

use std::sync::{Arc, RwLock};
use crate::gfx::gl_object::GLObject;

lazy_static!
{
    static ref GL_OBJECT_RELOADER: RwLock<Vec<Arc<RwLock<dyn GLObject<ReloadError=String> + Send + Sync>>>> = RwLock::new(vec![]);
}
pub fn reload_gl_objects(context: &Rc<Context>) -> Result<(), String>
{
    for obj in GL_OBJECT_RELOADER.write().or_else(|e| Err(e.to_string()))?.iter_mut()
    {
        obj.write().or_else(|e| Err(e.to_string()))?.reload(&context);
    }
    Ok(())
}

pub use gen_vec::{Index, closed::ClosedGenVec};
pub struct GlManager<'a>
{
    context: Rc<Context>,
    gl_objects: ClosedGenVec<Box<dyn GLObject<ReloadError=String> + 'a>>
}

impl GlManager<'_>
{
    pub fn new<'a>(context: &Rc<Context>) -> Arc<RwLock<GlManager<'a>>>
    {
        Arc::new(RwLock::new(GlManager
        {
            context: Rc::clone(&context),
            gl_objects: ClosedGenVec::new()
        }))
    }
}

type SafeGlManager<'a> = Arc<RwLock<GlManager<'a>>>;
fn add_to_gl_manager<'a, T>(manager: &SafeGlManager, gl_object: T) -> Result<Index, String> where T: GLObject<ReloadError=String> + 'a
{
    Ok(manager.write().or_else(|e| Err(e.to_string()))?.gl_objects.insert(Box::new(gl_object)))
}

pub mod transform;
pub mod gl_object;
pub mod shader;
pub mod vertex_array;
pub mod buffer;