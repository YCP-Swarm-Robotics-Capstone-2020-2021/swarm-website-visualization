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
    HtmlCanvasElement
};
use std::{rc::Rc, cell::RefCell};

use crate::
{
    gfx::
    {
        Context,
        new_context,
        gl_object::GlObject,
        shader::
        {
            shaderprogram::ShaderProgram,
            shadersrc,
            uniform_buffer::UniformBuffer,
        },
        vertex_array::{AttribPointer, VertexArray},
        buffer::Buffer,
        transform::Transformation,
    }
};
use cgmath::{Matrix4, Vector3, vec3, Vector4};

mod gfx;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern
{
    /// Javascript `alert` function
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn log_s(s: String)
{
    log(s.as_str());
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
    let context = new_context(&canvas)?;

    let mut shaderprog =
        ShaderProgram::new(&context, Some(shadersrc::BASIC_VERT.to_string()), Some(shadersrc::BASIC_FRAG.to_string()))
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

    let mut va = VertexArray::new(&context).expect("vertex array");
    va.bind();

    let mut vb = Buffer::new(&context, Context::ARRAY_BUFFER).expect("array buffer");
    vb.bind();
    vb.buffer_data(&triangle, Context::STATIC_DRAW);
    let _vb = va.add_buffer(vb, Some(vec![AttribPointer::with_defaults::<f32>(0, 3, Context::FLOAT, 0)]));

    let mut eb = Buffer::new(&context, Context::ELEMENT_ARRAY_BUFFER).expect("element array buffer");
    eb.bind();
    eb.buffer_data(&indices, Context::STATIC_DRAW);
    let _eb = va.add_buffer(eb, None);

    va.unbind();

    let mut transformation = Transformation::new();

    let mut uniform_buffer = UniformBuffer::new(
        &context,
        std::mem::size_of::<Matrix4<f32>>() as i32,
        // Needs to be Vector4 even though its actually a Vector3
        // Using 3 element vectors with google chrome causes issues
        std::mem::size_of::<Vector4<f32>>() as i32,
        Context::STATIC_DRAW
    ).expect("uniform buffer handle");
    uniform_buffer.bind();

    uniform_buffer.add_vert_block(&mut shaderprog, "VertData").expect("VertData bound");
    uniform_buffer.add_frag_block(&mut shaderprog, "FragData").expect("FragData bound");

    transformation.global.translate(&vec3(-0.5, 0.0, 0.0));
    let buff: &[f32; 16] = transformation.matrix().as_ref();
    uniform_buffer.buffer_vert_data(buff);

    let color: Vector3<f32> = vec3(253.0/255.0, 94.0/255.0, 0.0);
    let buff: &[f32; 3] = color.as_ref();
    uniform_buffer.buffer_frag_data(buff);

    va.bind();
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(Context::COLOR_BUFFER_BIT);
    context.draw_elements_with_i32(Context::TRIANGLES, indices.len() as i32, Context::UNSIGNED_INT, 0);

    log_s(format!("{:?}", crate::gfx::gl_get_errors(&context)));

    {
        let callback = Closure::wrap(Box::new(move |event: web_sys::WebGlContextEvent|
            {
                event.prevent_default();
            }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("webglcontextlost", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }
    {
        let canvas_clone = canvas.clone();
        let callback = Closure::wrap(Box::new(move |_event: web_sys::WebGlContextEvent|
            {
                //let canvas = canvas_clone;
                let context = new_context(&canvas_clone).unwrap();
                shaderprog.reload(&context).expect("shader program reloaded");
                uniform_buffer.reload(&context).expect("uniform buffer reloaded");
                va.reload(&context).expect("vertex array reloaded");

                context.clear_color(0.0, 0.0, 0.0, 1.0);
                context.clear(Context::COLOR_BUFFER_BIT);
                context.draw_elements_with_i32(Context::TRIANGLES, 3, Context::UNSIGNED_INT, 0);
                log_s(format!("{:?}", crate::gfx::gl_get_errors(&context)));

            }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("webglcontextrestored", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    Ok(())
}