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
use std::
{
    rc::Rc,
    cell::RefCell,
    boxed::Box,
    time::{Duration, SystemTime, UNIX_EPOCH}
};

#[macro_use]
mod redeclare;
#[macro_use]
extern crate memoffset;
#[macro_use]
extern crate downcast_rs;

mod gfx;
mod input;
mod math;

use crate::
{
    gfx::
    {
        Context,
        new_context,
        render_loop::RenderLoop,
        gl_object::
        {
            shader::
            {
                shaderprogram::ShaderProgram,
                shadersrc,
                uniform_buffer::UniformBuffer,
            },
            vertex_array::{AttribPointer, VertexArray},
            buffer::Buffer,
            texture::{TextureParams, Texture},
        },
        renderer::{vertex::Vertex},
    },
    input::
    {
        input_consts::*,
        listener::EventListener,
        states::{InputState, InputStateListener},
    },
    math::transform::{Transformation, SubTransformation},
};
use cgmath::
{
    Matrix4,
    Vector3,
    vec3,
    Vector4
};

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

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue>
{
    /*#[cfg(feature="debug")]
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
        ShaderProgram::new(&context, Some(shadersrc::TEXTURE_VERT.to_string()), Some(shadersrc::TEXTURE_FRAG.to_string()))
            .expect("shader program");
    shaderprog.bind();
    shaderprog.set_uniform_i32("tex", &[0]);
    let tex = Texture::new(&context, TextureParams
    {
        target: Context::TEXTURE_2D,
        format: Context::RGBA,
        size: (1, 1),
        wrap_type: Context::REPEAT,
        filter_type: Context::NEAREST,
        //data: vec![255, 0, 0, 255]
        //data: vec![(253.0f32/255.0f32) as u8, (94.0f32/255.0f32) as u8, 0, 255]
        data: vec![253.0 as u8, 94.0 as u8, 0, 255]
    }).expect("texture");
    context.active_texture(Context::TEXTURE0);
    tex.bind();
    // Triangle point data
    let triangle =
        [
            Vertex { pos: [-0.5, -0.5, 0.0], tex: [0.0, 0.0] },
            Vertex { pos: [ 0.5, -0.5, 0.0], tex: [1.0, 0.0] },
            Vertex { pos: [ 0.0,  0.5, 0.0], tex: [0.5, 1.0] }
        ];
    // Triangle point order
    let indices: [u32; 3] = [0, 1, 2];

    let mut va = VertexArray::new(&context).expect("vertex array");
    va.bind();

    let mut vb = Buffer::new(&context, Context::ARRAY_BUFFER).expect("array buffer");
    vb.bind();

    vb.buffer_data(&triangle, Context::STATIC_DRAW);
    let attribs = vec![
        AttribPointer::without_defaults(0, 3, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, pos) as i32),
        AttribPointer::without_defaults(1, 2, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, tex) as i32),
    ];
    let _vb = va.add_buffer(vb, Some(attribs));

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
        0,
        Context::STATIC_DRAW
    ).expect("uniform buffer handle");
    uniform_buffer.bind();

    uniform_buffer.add_vert_block(&mut shaderprog, "VertData").expect("VertData bound");


    transformation.global.translate(&vec3(-0.5, 0.0, 0.0));
    let buff: &[f32; 16] = transformation.matrix().as_ref();
    uniform_buffer.buffer_vert_data(buff);

    crate::log_s(format!("{:?}", crate::gfx::gl_get_errors(&context)));

    wrap!(transformation, shaderprog, uniform_buffer, va, tex);

    // Context container so the context can be updated from within the restored callback
    // First Rc is to allow the container to be cloned and then moved into the callback
    // RefCell is for interior mutability
    let context: Rc<RefCell<Context>> = Rc::new(RefCell::new(context));

    // I feel like this is overly complicated, but I'm not sure of a better way to maintain
    // a vector of things that need to be reloaded while allowing them to be used
    // without accessing them through the vector
    let globjects: Rc<RefCell<Vec<Rc<RefCell<dyn GlObject>>>>> = Rc::new(RefCell::new(
        vec![shaderprog.clone(), uniform_buffer.clone(), va.clone(), tex.clone()]
    ));

    let input_listener = Rc::new(InputStateListener::new(&canvas).expect("input state listener"));
    let render_func =
        {
            clone!(context, transformation, uniform_buffer, input_listener);
            let mut dir: f32 = 1.0;
            let speed: f32 = 1.0;
            let performance = window.performance().expect("performance");
            let mut last_time: Duration = Duration::new(0, 0);

            move ||
                {
                    if last_time.as_secs() == 0
                    {
                        last_time = time(&performance);
                    }
                    let now_time = time(&performance);
                    let elapsed_time = now_time - last_time;
                    last_time = now_time;
                    let elapsed_time = elapsed_time.as_secs_f32();

                    borrow_mut!(transformation, uniform_buffer);

                    transformation.global.translate(&vec3(speed * elapsed_time * dir, 0.0, 0.0));

                    let translation =
                        {
                            let mut t = transformation.global.get_translation().clone();
                            if dir == -1.0
                            {
                                t.x = f32::max(-0.5, t.x);
                            }
                            else if dir == 1.0
                            {
                                t.x = f32::min(0.5, t.x);
                            }
                            t
                        };

                    transformation.global.set_translation(translation);
                    if transformation.translation().x <= -0.5 || transformation.translation().x >= 0.5
                    {
                        dir *= -1.0;
                    }
                    let buff: &[f32; 16] = transformation.matrix().as_ref();
                    uniform_buffer.buffer_vert_data(buff);

                    let context = context.borrow();
                    context.clear_color(0.0, 0.0, 0.0, 1.0);
                    context.clear(Context::COLOR_BUFFER_BIT);

                    va.borrow().bind();
                    context.draw_elements_with_i32(Context::TRIANGLES, 3, Context::UNSIGNED_INT, 0);

                    let state = input_listener.key_state(Key_a);
                    if state == InputState::Down
                    {
                        crate::log("Key 'a' is down");
                    }
                    else if state == InputState::Repeating
                    {
                        crate::log("Key 'a' is repeating");
                    }
                }
        };


    let render_loop = Rc::new(RefCell::new(RenderLoop::init(&window, &canvas, &context, &globjects, render_func).expect("render_loop")));
    //render_loop.borrow_mut().start().unwrap();

    {
        clone!(context, render_loop);
        let callback = move |event: web_sys::KeyboardEvent|
            {
                if event.key() == Key_1
                {
                    render_loop.borrow_mut().start().expect("render loop started");
                }
                else if event.key() == Key_2
                {
                    render_loop.borrow_mut().pause().expect("render loop paused");
                    borrow!(context);
                    context.clear_color(0.0, 0.0, 0.0, 1.0);
                    context.clear(Context::COLOR_BUFFER_BIT);
                }
                else if event.key() == Key_3
                {
                    render_loop.borrow_mut().cleanup();
                }
            };
        let ev = EventListener::new(&canvas, "keydown", callback).expect("event listener registered");
        ev.forget();
    }*/

    Ok(())
}

/// From https://rustwasm.github.io/docs/wasm-bindgen/examples/performance.html
fn time(performance: &web_sys::Performance) -> Duration
{
    let perf = performance.now();
    let secs = (perf as u64) / 1_000;
    let nanos = ((perf as u32) % 1_000) * 1_000_000;
    Duration::new(secs, nanos)
}