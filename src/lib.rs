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
        renderer::
        {
            renderer2d::
            {
                RenderDto,
                Node,
                Renderer2D,
            },
            vertex::Vertex,
        },
        gl_object::
        {
            traits::{GlObject, Bindable},
            buffer::Buffer,
            uniform_buffer::UniformBuffer,
            ArrayBuffer,
            ElementArrayBuffer,
            vertex_array::{AttribPointer, VertexArray},
            texture::{Texture2d, Texture2dParams},
            manager::{GlObjectManager},
        },
        camera::Camera,
    },
    input::
    {
        input_consts::*,
        listener::EventListener,
        states::{InputState, InputStateListener},
    },
    math::transform::{Transformation},
};
use cgmath::{Matrix4, vec3, Deg, InnerSpace};

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

const TRIANGLE_VERTICESS: [Vertex; 3] =
    [
        Vertex { pos: [-0.5, -0.5, 1.0], tex: [0.0, 0.0] },
        Vertex { pos: [ 0.5, -0.5, 1.0], tex: [1.0, 0.0] },
        Vertex { pos: [ 0.0,  0.5, 1.0], tex: [0.5, 1.0] }
    ];
const TRIANGLE_INDICES: [u32; 3] = [0, 1, 2];

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue>
{
    // Allow panics to print to javascript console if debug build
    #[cfg(feature="debug")]
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Get HTML element references
    let window: Window = window().expect("window context");
    let document: Document = window.document().expect("document context");
    let canvas =
        {
            let elem = document.get_element_by_id("canvas").expect("canvas handle");
            elem.dyn_into::<HtmlCanvasElement>()?
        };
    let context = new_context(&canvas)?;

    // Setup object manager
    let manager = Rc::new(RefCell::new(GlObjectManager::new()));
    let mut manager_ref = manager.borrow_mut();

    // Texture for triangle 1
    let texture_handle_t1 = manager_ref.insert_texture2d(
        Texture2d::new(&context, Texture2dParams
        {
            target: Context::TEXTURE_2D,
            format: Context::RGBA,
            size: (1, 1),
            wrap_type: Context::REPEAT,
            filter_type: Context::NEAREST,
            data: vec![253.0 as u8, 94.0 as u8, 0, 255]
        }).expect("texture")
    );

    // Texture for triangle 2
    let texture_handle_t2 = manager_ref.insert_texture2d(
        Texture2d::new(&context, Texture2dParams
        {
            target: Context::TEXTURE_2D,
            format: Context::RGBA,
            size: (1, 1),
            wrap_type: Context::REPEAT,
            filter_type: Context::NEAREST,
            data: vec![30.0 as u8, 144.0 as u8, 255.0 as u8, 255]
        }).expect("texture")
    );

    // VAO setup
    let vert_arr_handle = manager_ref.insert_vertex_array(
        VertexArray::new(&context).expect("vertex array")
    );

    let arr_buff_handle = manager_ref.insert_array_buffer(
        ArrayBuffer::new(&context).expect("array buffer")
    );

    let elem_buff_handle = manager_ref.insert_element_array_buffer(
        ElementArrayBuffer::new(&context).expect("element array buffer")
    );

    {
        VertexArray::bind(&manager_ref, vert_arr_handle);
        // Setup the vertex array buffer with the triangle vertices
        ArrayBuffer::bind(&manager_ref, arr_buff_handle);
        {
            let mut arr_buff = manager_ref.get_mut_array_buffer(arr_buff_handle).expect("array buffer");
            arr_buff.buffer_data(&TRIANGLE_VERTICESS, Context::STATIC_DRAW);
        }
        // Setup the element array buffer with the triangle indices
        ElementArrayBuffer::bind(&manager_ref, elem_buff_handle);
        {
            let mut elem_arr_buff = manager_ref.get_mut_element_array_buffer(elem_buff_handle).expect("element array buffer");
            elem_arr_buff.buffer_data(&TRIANGLE_INDICES, Context::STATIC_DRAW);
        }
        // Register the vertex and element array buffers with the VAO
        {
            let mut vert_arr = manager_ref.get_mut_vertex_array(vert_arr_handle).expect("vertex array");

            let attribs = vec![
                AttribPointer::without_defaults(0, 3, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, pos) as i32),
                AttribPointer::without_defaults(1, 2, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, tex) as i32),
            ];
            vert_arr.register_array_buffer(arr_buff_handle, Some(attribs));
            vert_arr.register_element_array_buffer(elem_buff_handle, None);
        }
        VertexArray::unbind(&manager_ref, vert_arr_handle);
        ArrayBuffer::unbind(&manager_ref, arr_buff_handle);
        ElementArrayBuffer::unbind(&manager_ref, elem_buff_handle);
    }

    // Release the borrow on the manager
    drop(manager_ref);

    // Log any errors that may have occurred during setup
    crate::log_s(format!("{:?}", crate::gfx::gl_get_errors(&context)));

    // Wrap the context in an Rc<RefCell<>>
    wrap!(context);

    // Direction of triangle movement
    let mut dir: f32 = 1.0;
    // Speed factor of triangle movement
    let speed: f32 = 0.5;
    // Get javascript performance ref for getting frame time
    let performance = window.performance().expect("performance");
    let mut last_time: Duration = Duration::new(0, 0);

    let delta_time: f32 = 0.01;
    let mut accumulator: f32 = 0.0;

    let renderer = Renderer2D::new(&context.borrow(), &mut manager.borrow_mut()).expect("2d renderer");

    // Setup render information for triangle
    let mut transformation_t1 = Transformation::new();
    let mut transformation_t2 = Transformation::new();
    let t1 = RenderDto
    {
        tex_handle: texture_handle_t1,
        vert_arr_handle: vert_arr_handle,
        num_indices: 3
    };
    let t2 = RenderDto
    {
        tex_handle: texture_handle_t2,
        vert_arr_handle: vert_arr_handle,
        num_indices: 3
    };

    let perspective = cgmath::perspective(Deg(45.0f32), 1.0f32, 0.1f32, 10.0f32);
    let mut camera = Rc::new(RefCell::new(
        Camera::from_eye(
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 1.0, 0.0)
        )));
    {
        clone!(camera);
        let callback = move |event: web_sys::WheelEvent|
            {
                if event.delta_y() > 0.0
                {
                    camera.borrow_mut().move_cam_long(0.1);
                }
                else if event.delta_y() < 0.0
                {
                    camera.borrow_mut().move_cam_long(-0.1);
                }
            };
        let ev = EventListener::new(&canvas, "wheel", callback).expect("zoom event listener");
        ev.forget();
    }

    {
        clone!(camera);
        let callback = move |event: web_sys::MouseEvent|
            {
                if event.buttons() == 1
                {
                    borrow_mut!(camera);
                    if event.movement_x() != 0
                    {
                        //camera.move_cam_lat(event.movement_x() as f32 / 800.0);
                        let delta = if event.movement_x() < 0 { -1.0 } else { 1.0 };
                        camera.rotate_cam_yaw(delta);
                    }
                    if event.movement_y() != 0
                    {
                        //camera.move_cam_vert(event.movement_y() as f32 / 800.0);
                    }
                }
            };
        let ev = EventListener::new(&canvas, "mousemove", callback).expect("mouse move event listener");
        ev.forget();
    }

    let input_listener = Rc::new(InputStateListener::new(&canvas).expect("input state listener"));

    let render_func =
        {
            clone!(context, manager, camera);

            move ||
                {
                    // If this is the first frame, initialize last_time to now
                    if last_time.as_secs() == 0
                    {
                        last_time = time(&performance);
                    }
                    // Calculate the time elapsed between last frame and now
                    let now_time = time(&performance);
                    let elapsed_time = now_time - last_time;
                    last_time = now_time;
                    let elapsed_time =
                        {
                            let time = elapsed_time.as_secs_f32();
                            if time > 0.25 { 0.25 } else { time }
                        };

                    accumulator += elapsed_time;

                    // Perform any updates skipped due to missed frames
                    while accumulator >= delta_time
                    {
                        //transformation_t1.global.translate(&vec3(speed * dir * delta_time, 0.0, 0.0));
                        transformation_t1.local.rotate_angle_axis(Deg(10.0 * delta_time), &vec3(0.0, 0.0, 1.0));

                        transformation_t2.local.rotate_angle_axis(Deg(10.0 * delta_time), &vec3(0.0, 0.0, 1.0));

                        // Triangle movement bounds
                        /*                        let translation =
                                                    {
                                                        let mut t = transformation_t1.global.get_translation().clone();
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

                                                transformation_t1.global.set_translation(translation);
                                                // Reverse triangle direction if out of the bounds
                                                if transformation_t1.translation().x <= -0.5 || transformation_t1.translation().x >= 0.5
                                                {
                                                    dir *= -1.0;
                                                }*/

                        accumulator -= delta_time;
                    }

                    {
                        borrow!(context);
                        // Reset the render area
                        context.clear_color(0.0, 0.0, 0.0, 1.0);
                        context.clear(Context::COLOR_BUFFER_BIT);

                        // Setup scene graph
                        let nodes =
                            vec![
                                Node(
                                    &t1,
                                    transformation_t1.matrix(),
                                    Some(vec![
                                        Node(
                                            &t2,
                                            transformation_t2.matrix(),
                                            None
                                        ),
                                    ])
                                ),
                            ];

                        renderer.render(&context, &manager.borrow(), perspective * camera.borrow().view_matrix(), &nodes);
                    }

                    // Input state tests
                    borrow_mut!(camera);
                    if input_listener.key_state(Key_ArrowLeft) == InputState::Down
                    {
                        camera.move_cam_lat(0.1);
                        crate::log("aa");
                    }
                    if input_listener.key_state(Key_ArrowRight) == InputState::Down
                    {
                        camera.move_cam_lat(-0.1);
                    }
                    if input_listener.key_state(Key_ArrowUp) == InputState::Down
                    {
                        camera.move_cam_vert(0.1);
                    }
                    if input_listener.key_state(Key_ArrowDown) == InputState::Down
                    {
                        camera.move_cam_vert(-0.1);
                    }

                }
        };

    // Setup and start render loop
    let render_loop = Rc::new(RefCell::new(RenderLoop::init(&window, &canvas, &context, &manager, render_func).expect("render_loop")));
    render_loop.borrow_mut().start().unwrap();

    {
        let callback = move |event: web_sys::KeyboardEvent|
            {
                if event.key() == Key_ArrowUp || event.key() == Key_ArrowDown
                    || event.key() == Key_ArrowLeft || event.key() == Key_ArrowRight
                {
                    event.prevent_default();
                }
                if false
                {
                    render_loop.borrow();
                }
            };
        let ev = EventListener::new(&canvas, "keydown", callback).expect("event listener registered");
        ev.forget();
        let callback = move |event: web_sys::WheelEvent|
            {
                event.prevent_default();
            };
        let ev = EventListener::new(&canvas, "wheel", callback).expect("event listener registered");
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