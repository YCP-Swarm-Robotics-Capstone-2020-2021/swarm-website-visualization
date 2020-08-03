use wasm_bindgen::
{
    prelude::*,
    JsCast,
};
#[cfg(feature="debug")]
use console_error_panic_hook;
use web_sys::
{
    window,
    Window,
    Document,
    HtmlCanvasElement,

    WebGl2RenderingContext,

};
use std::rc::Rc;
use crate::
{
    gfx::
    {
        shader::
        {
            shaderprogram::ShaderProgram,
            shadersrc
        }
    }
};

type Context = WebGl2RenderingContext;
fn gl_get_error(context: &Rc<Context>) -> &'static str
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

mod gfx;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern
{
    /// Javascript `alert` function
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn pointless_binding(msg: &str)
{
    alert(msg);
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue>
{
    #[cfg(feature="debug")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window: Window = window().expect("window context");
    let document: Document = window.document().expect("document context");
    let canvas =
        {
            let elem = document.get_element_by_id("canvas").expect("canvas handle");
            elem.dyn_into::<HtmlCanvasElement>()?
        };
    let context = canvas.get_context("webgl2")?.expect("webgl context").dyn_into::<Context>()?;
    let context = Rc::new(context);

    let shaderprog =
        ShaderProgram::new(&context, Some(shadersrc::basic_vert::SRC), Some(shadersrc::basic_frag::SRC))
            .expect("shader program");


    Ok(())
}