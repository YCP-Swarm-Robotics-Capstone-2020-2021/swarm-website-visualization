pub type Context = web_sys::WebGl2RenderingContext;
pub fn gl_get_error(context: &Rc<Context>) -> &'static str
{
    match context.get_error()
    {
        Context::NO_ERROR => "NO_ERROR",
        Context::INVALID_ENUM => "INVALID_ENUM",
        Context::INVALID_VALUE => "INVALID_VALUE",
        Context::INVALID_OPERATION => "INVALID_OPERATION",
        Context::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
        Context::OUT_OF_MEMORY => "OUT_OF_MEMORY",
        Context::CONTEXT_LOST_WEBGL => "CONTEXT_LOST_WEBGL",
        _ => "UNKNOWN_ERROR"
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

pub mod transform;
pub mod gl_object;
pub mod shader;
pub mod vertex_array;
pub mod buffer;