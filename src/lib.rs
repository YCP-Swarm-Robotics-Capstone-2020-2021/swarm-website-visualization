#[macro_use]
extern crate lazy_static;
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
use std::rc::Rc;

use crate::
{
    gfx::
    {
        Context,
        gl_object::GLObject,
        shader::
        {
            shaderprogram::ShaderProgram,
            shadersrc,
            //uniform_buffer::UniformBuffer,
        },
        vertex_array::VertexArray,
        buffer::Buffer,
        transform::Transformation,
    }
};
use cgmath::{Matrix4, Vector3, vec3};

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

    {
        let callback = Closure::wrap(Box::new(move |event: web_sys::WebGlContextEvent|
            {
                event.prevent_default();
                alert("Context lost");
            }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("webglcontextlost", callback.as_ref().unchecked_ref());
        callback.forget();

        let callback = Closure::wrap(Box::new(move |context: Context|
            {
                alert("Context restored");
                //run_visualization(&Rc::new(context), true);
            }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("webglcontextrestored", callback.as_ref().unchecked_ref());
        callback.forget();
    }

    let mut shaderprog =
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

    let mut va = VertexArray::new(&context).expect("vertex array");

    let mut vb = Buffer::new(&context, Context::ARRAY_BUFFER).expect("array buffer");
    let mut eb = Buffer::new(&context, Context::ELEMENT_ARRAY_BUFFER).expect("element array buffer");

    va.bind();
    vb.bind();
    vb.buffer_data_f32(&triangle, Context::STATIC_DRAW);
    vb.reload(&context);

    eb.bind();
    eb.buffer_data_u32(&indices, Context::STATIC_DRAW);

    va.attrib_ptr::<f32>(0, 3, Context::FLOAT, 0);

    let mut transformation = Transformation::new();

    let ub_handle = shaderprog.new_uniform_buffer(
        &context,
        std::mem::size_of::<Matrix4<f32>>() as i32,
        std::mem::size_of::<Vector3<f32>>() as i32,
        Context::STATIC_DRAW
    ).expect("uniform buffer handle");
    shaderprog.bind_uniform_buffer(ub_handle).expect("bound uniform buffer");

    shaderprog.add_vert_uniform_block(ub_handle, "VertData").expect("VertData uniform block");
    transformation.global.translate(&vec3(-0.5, 0.0, 0.0));
    let buff: &[f32; 16] = transformation.matrix().as_ref();
    shaderprog.buffer_vert_uniform_data_f32(ub_handle, buff).expect("transformation buffered");

    shaderprog.add_frag_uniform_block(ub_handle, "FragData").expect("FragData uniform block");
    let color: Vector3<f32> = vec3(253.0/255.0, 94.0/255.0, 0.0);
    let buff: &[f32; 3] = color.as_ref();
    shaderprog.buffer_frag_uniform_data_f32(ub_handle, buff).expect("color buffered");

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(Context::COLOR_BUFFER_BIT);
    context.draw_elements_with_i32(Context::TRIANGLES, indices.len() as i32, Context::UNSIGNED_INT, 0);

    Ok(())
}