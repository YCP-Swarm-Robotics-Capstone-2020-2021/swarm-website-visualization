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

    let context = gfx::new_context(&canvas)?;

    let shaderprog =
        ShaderProgram::new(&context, Some(shadersrc::BASIC_VERT), Some(shadersrc::BASIC_FRAG))
            .expect("shader program");


    Ok(())
}