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
};

use crate::
{
    gfx::
    {
        Context,
        gl_object::GLObject,
        shader::
        {
            shaderprogram::ShaderProgram,
            shadersrc
        },
        vertex_array::VertexArray,
        buffer::Buffer
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
    shaderprog.bind();


    // Triangle point data
    let triangle: [f32; 9] =
        [
            -0.5, -0.5,  0.0,
            0.5, -0.5,  0.0,
            0.0,  0.5,  0.0
        ];
    // Triangle point order
    let indices: [u32; 3] = [0, 1, 2];

    let va = VertexArray::new(&context).expect("vertex array");

    let vb = Buffer::new(&context, Context::ARRAY_BUFFER).expect("array buffer");
    let eb = Buffer::new(&context, Context::ELEMENT_ARRAY_BUFFER).expect("element array buffer");

    va.bind();
    vb.bind();
    vb.buffer_data_f32(&triangle, Context::STATIC_DRAW);

    eb.bind();
    eb.buffer_data_u32(&indices, Context::STATIC_DRAW);

    va.attrib_ptr::<f32>(0, 3, Context::FLOAT, 0);

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(Context::COLOR_BUFFER_BIT);
    context.draw_elements_with_i32(Context::TRIANGLES, indices.len() as i32, Context::UNSIGNED_INT, 0);

    Ok(())
}