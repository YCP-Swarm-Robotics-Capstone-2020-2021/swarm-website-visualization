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
    time::Duration,
};

#[macro_use]
mod redeclare;
#[macro_use]
extern crate memoffset;

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
        renderer::{vertex::Vertex},
        gl_object::
        {
            traits::{GlObject, Bindable},
            shader_program::{ShaderProgram, shader_source::*},
            buffer::Buffer,
            uniform_buffer::UniformBuffer,
            ArrayBuffer,
            ElementArrayBuffer,
            vertex_array::{AttribPointer, VertexArray},
            texture::{Texture2d, Texture2dParams},
            manager::{GlObjectManager},
        }
    },
    input::
    {
        input_consts::*,
        listener::EventListener,
        states::{InputState, InputStateListener},
    },
    math::transform::{Transformation},
};
use cgmath::
{
    Matrix4,
    vec3,
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

    let manager = Rc::new(RefCell::new(GlObjectManager::new()));
    let mut manager_ref = manager.borrow_mut();

    let shader_prog =
        ShaderProgram::new(&context, Some(TEXTURE_VERT.to_string()), Some(TEXTURE_FRAG.to_string()))
            .expect("shader program");
    let shader_progam = manager_ref.insert_shader_program(shader_prog);
    ShaderProgram::bind(&mut manager_ref, shader_progam);
    manager_ref.get_mut_shader_program(shader_progam).unwrap().set_uniform_i32("tex", &[0]).expect("tex sampler2d set to 0");

    let tex = Texture2d::new(&context, Texture2dParams
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

    let tex = manager_ref.insert_texture2d(tex);
    Texture2d::bind(&mut manager_ref, tex);

    // Triangle point data
    let triangle =
        [
            Vertex { pos: [-0.5, -0.5, 0.0], tex: [0.0, 0.0] },
            Vertex { pos: [ 0.5, -0.5, 0.0], tex: [1.0, 0.0] },
            Vertex { pos: [ 0.0,  0.5, 0.0], tex: [0.5, 1.0] }
        ];
    // Triangle point order
    let indices: [u32; 3] = [0, 1, 2];

    // VAO setup
    let mut vao = VertexArray::new(&context).expect("vertex array");
    vao.bind_internal();

    // Array buffer setup
    let mut arr_buff = ArrayBuffer::new(&context).expect("array buffer");
    arr_buff.bind_internal();

    arr_buff.buffer_data(&triangle, Context::STATIC_DRAW);
    let attribs = vec![
        AttribPointer::without_defaults(0, 3, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, pos) as i32),
        AttribPointer::without_defaults(1, 2, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, tex) as i32),
    ];
    let arr_buff = manager_ref.insert_array_buffer(arr_buff);
    vao.register_array_buffer(arr_buff, Some(attribs));

    // Element array buffer setup
    let mut elem_arr_buff = ElementArrayBuffer::new(&context).expect("element array buffer");
    elem_arr_buff.bind_internal();
    elem_arr_buff.buffer_data(&indices, Context::STATIC_DRAW);
    let elem_arr_buff = manager_ref.insert_element_array_buffer(elem_arr_buff);
    vao.register_element_array_buffer(elem_arr_buff, None);

    vao.unbind_internal();
    manager_ref.get_array_buffer(arr_buff).unwrap().unbind_internal();
    manager_ref.get_element_array_buffer(elem_arr_buff).unwrap().unbind_internal();
    let vao = manager_ref.insert_vertex_array(vao);

    let mut transformation = Transformation::new();

    let mut uniform_buffer = UniformBuffer::new(
        &context,
        std::mem::size_of::<Matrix4<f32>>() as i32,
        // Needs to be Vector4 even though its actually a Vector3
        // Using 3 element vectors with google chrome causes issues
        0,
        Context::STATIC_DRAW
    ).expect("uniform buffer handle");
    uniform_buffer.bind_internal();
    uniform_buffer.add_vert_block(&mut manager_ref.get_mut_shader_program(shader_progam).unwrap(), "VertData").expect("VertData bound");

    transformation.global.translate(&vec3(-0.5, 0.0, 0.0));
    let buff: &[f32; 16] = transformation.matrix().as_ref();
    uniform_buffer.buffer_vert_data(buff);

    uniform_buffer.unbind_internal();
    let uniform_buffer = manager_ref.insert_uniform_buffer(uniform_buffer);

    // Release the borrow on the manager
    drop(manager_ref);

    crate::log_s(format!("{:?}", crate::gfx::gl_get_errors(&context)));

    wrap!(context);

    let input_listener = Rc::new(InputStateListener::new(&canvas).expect("input state listener"));
    let render_func =
        {
            clone!(context, manager);
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
                    {
                        borrow_mut!(manager);
                        UniformBuffer::bind(&mut manager, uniform_buffer);
                        {
                            let mut uniform_buffer = manager.get_mut_uniform_buffer(uniform_buffer).expect("uniform buffer");
                            uniform_buffer.buffer_vert_data(buff);
                        }
                        VertexArray::bind(&mut manager, vao);
                    }

                    {
                        borrow!(context);
                        context.clear_color(0.0, 0.0, 0.0, 1.0);
                        context.clear(Context::COLOR_BUFFER_BIT);
                        context.draw_elements_with_i32(Context::TRIANGLES, 3, Context::UNSIGNED_INT, 0);
                    }

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


    let render_loop = Rc::new(RefCell::new(RenderLoop::init(&window, &canvas, &context, &manager, render_func).expect("render_loop")));
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
    }

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