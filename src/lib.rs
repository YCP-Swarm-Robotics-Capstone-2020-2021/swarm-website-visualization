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
    time::Duration,
    hash::BuildHasherDefault,
    collections::HashMap,
};
use twox_hash::XxHash32;

#[macro_use]
mod redeclare;
#[macro_use]
mod test_util;
#[macro_use]
extern crate memoffset;
#[macro_use]
extern crate float_cmp;

mod gfx;
mod input;
mod math;
mod resource;
mod script;

use crate::
{
    gfx::
    {
        Context,
        new_context,
        mesh::{Vertex, Mesh},
        render_loop::RenderLoop,
        renderer::
        {
            renderer::
            {
                RenderDto,
                Node,
                Renderer,
            },
        },
        gl_object::
        {
            traits::GlObject,
            buffer::Buffer,
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
    resource::
    {
        async_loader::{self, /*AsyncResourceError*/},
        manager::ResourceManager,
    },
};
use cgmath::{vec3, Deg};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern
{
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn log_s(s: String)
{
    log(s.as_str());
}

#[wasm_bindgen]
/// Initialize and start the visualization software
/// `canvas_id` is the ID of the HTML canvas element to render the visualization to
/// `resource_dir` is the directory containing resources for the visualization (textures, models, etc)
pub async fn init_visualization(canvas_id: String, resource_dir: String) -> Result<(), JsValue>
{
    let canvas_id = 
        {
            if canvas_id.is_empty()
            {
                crate::log("Warning: No canvas id provided, defaulting to \"canvas\"");
                String::from("canvas")
            }
            else
            {
                canvas_id.to_owned()
            }
        };
    let resource_dir = 
        {
            if resource_dir.is_empty()
            {
                crate::log("Warning: No resource directory provided, defaulting to '/'");
                String::from("/")
            }
            else if resource_dir.ends_with("/")
            {
                resource_dir.to_owned()
            }
            else
            {
                resource_dir.to_owned() + "/"
            }
        };

    let mut resource_manager = ResourceManager::new();

    let mut resource_urls = vec![
        "models/dolphin.obj".to_string(),
        "models/room.obj".to_string(),
        "images/tex_atlas.pbm".to_string(),
        "images/robot_tex.pbm".to_string(),
        "shaders/texture_vert.glsl".to_string(),
        "shaders/texture_frag.glsl".to_string(),
    ];

    for url in &mut resource_urls
    {
        *url = resource_dir.to_owned() + url;
    }

    //let urls = resource_urls.iter().map(AsRef::<str>::as_ref).collect();
    let mut resources = async_loader::load_multiple(&resource_urls).await;

    resource_manager.insert_with_name(
        "robot.obj",
        resources.remove(0).expect("robot.obj"));
    resource_manager.insert_with_name(
        "room.obj",
        resources.remove(0).expect("room.obj"));
    resource_manager.insert_with_name(
        "tex_atlas.pbm",
        resources.remove(0).expect("tex_atlas.pbm"));
    resource_manager.insert_with_name(
        "robot_tex.pbm",
        resources.remove(0).expect("robot_tex.pbm"));
    resource_manager.insert_with_name(
        "texture_vert.glsl",
        resources.remove(0).expect("texture_vert.glsl"));
    resource_manager.insert_with_name(
        "texture_frag.glsl",
        resources.remove(0).expect("texture_frag.glsl"));

    wrap!(resource_manager);

    // Get HTML element references
    let window: Window = window().expect("window context");
    let document: Document = window.document().expect("document context");
    let canvas =
        {
            let elem = document.get_element_by_id(&canvas_id).expect("canvas element exists");
            elem.dyn_into::<HtmlCanvasElement>()?
        };

    {
        fn update_canvas_size(canvas: &HtmlCanvasElement)
        {
            let rect = canvas.get_bounding_client_rect();
            canvas.set_width(rect.width() as u32);
            canvas.set_height(rect.height() as u32);
        }
        update_canvas_size(&canvas);

        let canvas = canvas.clone();
        let callback = move |_event: web_sys::UiEvent|
            {
                update_canvas_size(&canvas);

                let mut width = canvas.width() as f32;
                let mut height = canvas.height() as f32;

                while !approx_eq!(f32, width / height, 16.0/9.0, epsilon=0.005)
                {
                    let x = ((width / 16.0).max(height / 9.0)).floor();
                    width = x * 16.0;
                    height = x * 9.0;
                }
                let context = new_context(&canvas).expect("canvas webgl2 context");
                context.viewport(0, 0, width as i32, height as i32);
            };
        let ev = EventListener::new(&window, "resize", callback).expect("event listener registered");
        ev.forget();
    }
    let canvas_size: (u32, u32) = (canvas.width(), canvas.height());

    let context = new_context(&canvas)?;
    let context_config_func =
        {
            let canvas = canvas.clone();
            move |context: &Context|
                {
                    context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
                    context.pixel_storei(Context::UNPACK_ALIGNMENT, 1);
                    context.enable(Context::CULL_FACE);
                    context.enable(Context::DEPTH_TEST);
                }
        };
    context_config_func(&context);

    let robot_obj = resource_manager.borrow().get_by_name("robot.obj").expect("robot obj resource").clone();
    let robot_mesh = Mesh::from_reader(&*robot_obj).expect("robot mesh");
    let skybox_obj = resource_manager.borrow().get_by_name("room.obj").expect("skybox obj resource").clone();
    let skybox_mesh = Mesh::from_reader(&*skybox_obj).expect("skybox mesh");

    // Setup object manager
    let manager = Rc::new(RefCell::new(GlObjectManager::new()));
    let mut manager_ref = manager.borrow_mut();


    // Texture atlas
    let skybox_tex_pbm = resource_manager.borrow().get_by_name("tex_atlas.pbm").expect("texture atlas").clone();

    let skybox_texture_handle = manager_ref.insert_texture2d(
        Texture2d::new(&context, Texture2dParams
        {
            target: Context::TEXTURE_2D,
            internal_format: Context::RGB8,
            format: Context::RGB,
            size: (800, 400),
            wrap_type: Context::REPEAT,
            filter_type: Context::LINEAR,
            data: skybox_tex_pbm
        }).expect("texture atlas")
    );
    {
        Texture2d::bind(&manager_ref, skybox_texture_handle);
        manager_ref.get_texture2d(skybox_texture_handle).expect("skybox texture2d").setup_texture().expect("texture2d setup");
    }

    let robot_tex_pbm = resource_manager.borrow().get_by_name("robot_tex.pbm").expect("robot texture").clone();
    let robot_texture_handle = manager_ref.insert_texture2d(
        Texture2d::new(&context, Texture2dParams
        {
            target: Context::TEXTURE_2D,
            internal_format: Context::RGB8,
            format: Context::RGB,
            size: (640, 427),
            wrap_type: Context::REPEAT,
            filter_type: Context::LINEAR,
            data: robot_tex_pbm
        }).expect("robot texture")
    );
    {
        Texture2d::bind(&manager_ref, robot_texture_handle);
        manager_ref.get_texture2d(robot_texture_handle).expect("robot texture2d").setup_texture().expect("texture2d setup");
    }

    let robot_vao_handle = manager_ref.insert_vertex_array(
        VertexArray::new(&context).expect("robot vertex array")
    );

    {
        let arr_buff_handle = manager_ref.insert_array_buffer(
            ArrayBuffer::new(&context).expect("robot array buffer")
        );

        let elem_buff_handle = manager_ref.insert_element_array_buffer(
            ElementArrayBuffer::new(&context).expect("robot element array buffer")
        );
        VertexArray::bind(&manager_ref, robot_vao_handle);
        // Setup the vertex array buffer with the robot vertices
        ArrayBuffer::bind(&manager_ref, arr_buff_handle);
        {
            let mut arr_buff = manager_ref.get_mut_array_buffer(arr_buff_handle).expect("robot array buffer");
            arr_buff.buffer_data(&robot_mesh.vertices, Context::STATIC_DRAW);
        }
        // Setup the element array buffer with the robot indices
        ElementArrayBuffer::bind(&manager_ref, elem_buff_handle);
        {
            let mut elem_arr_buff = manager_ref.get_mut_element_array_buffer(elem_buff_handle).expect("robot element array buffer");
            elem_arr_buff.buffer_data(&robot_mesh.indices, Context::STATIC_DRAW);
        }
        // Register the vertex and element array buffers with the VAO
        {
            let mut vert_arr = manager_ref.get_mut_vertex_array(robot_vao_handle).expect("vertex array");

            let attribs = vec![
                AttribPointer::without_defaults(0, 3, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, position) as i32),
                AttribPointer::without_defaults(1, 3, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, normal) as i32),
                AttribPointer::without_defaults(2, 2, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, texcoord) as i32),
            ];
            vert_arr.register_array_buffer(arr_buff_handle, Some(attribs));
            vert_arr.register_element_array_buffer(elem_buff_handle, None);
        }
        VertexArray::unbind(&manager_ref, robot_vao_handle);
        ArrayBuffer::unbind(&manager_ref, arr_buff_handle);
        ElementArrayBuffer::unbind(&manager_ref, elem_buff_handle);
    }

    let skybox_vao_handle = manager_ref.insert_vertex_array(
        VertexArray::new(&context).expect("skybox vertex array")
    );

    {
        let arr_buff_handle = manager_ref.insert_array_buffer(
            ArrayBuffer::new(&context).expect("skybox array buffer")
        );

        let elem_buff_handle = manager_ref.insert_element_array_buffer(
            ElementArrayBuffer::new(&context).expect("skybox element array buffer")
        );
        VertexArray::bind(&manager_ref, skybox_vao_handle);
        // Setup the vertex array buffer with the skybox vertices
        ArrayBuffer::bind(&manager_ref, arr_buff_handle);
        {
            let mut arr_buff = manager_ref.get_mut_array_buffer(arr_buff_handle).expect("skybox array buffer");
            arr_buff.buffer_data(&skybox_mesh.vertices, Context::STATIC_DRAW);
        }
        // Setup the element array buffer with the skybox indices
        ElementArrayBuffer::bind(&manager_ref, elem_buff_handle);
        {
            let mut elem_arr_buff = manager_ref.get_mut_element_array_buffer(elem_buff_handle).expect("skybox element array buffer");
            elem_arr_buff.buffer_data(&skybox_mesh.indices, Context::STATIC_DRAW);
        }
        // Register the vertex and element array buffers with the VAO
        {
            let mut vert_arr = manager_ref.get_mut_vertex_array(skybox_vao_handle).expect("skybox vertex array");

            let attribs = vec![
                AttribPointer::without_defaults(0, 3, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, position) as i32),
                AttribPointer::without_defaults(1, 3, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, normal) as i32),
                AttribPointer::without_defaults(2, 2, Context::FLOAT, false, std::mem::size_of::<Vertex>() as i32, offset_of!(Vertex, texcoord) as i32),
            ];
            vert_arr.register_array_buffer(arr_buff_handle, Some(attribs));
            vert_arr.register_element_array_buffer(elem_buff_handle, None);
        }
        VertexArray::unbind(&manager_ref, skybox_vao_handle);
        ArrayBuffer::unbind(&manager_ref, arr_buff_handle);
        ElementArrayBuffer::unbind(&manager_ref, elem_buff_handle);
    }

    // Release the borrow on the manager
    drop(manager_ref);

    // Log any errors that may have occurred during setup
    crate::log_s(format!("{:?}", crate::gfx::gl_get_errors(&context)));

    // Wrap the context in an Rc<RefCell<>>
    wrap!(context);

    // Get javascript performance ref for getting frame time
    let performance = window.performance().expect("performance");
    let mut last_time: Duration = Duration::new(0, 0);

    let delta_time: f32 = 0.01;
    let mut accumulator: f32 = 0.0;

    let renderer = Renderer::new(&context.borrow(), &mut manager.borrow_mut(), &resource_manager.borrow()).expect("renderer");

    // Setup render information
    let robot_renderable = RenderDto
    {
        tex_handle: robot_texture_handle,
        vert_arr_handle: robot_vao_handle,
        num_indices: robot_mesh.indices.len() as i32,
    };

    let mut skybox_transform = Transformation::new();
    let skybox_renderable = RenderDto
    {
        tex_handle: skybox_texture_handle,
        vert_arr_handle: skybox_vao_handle,
        num_indices: skybox_mesh.indices.len() as i32,
    };

    let perspective = cgmath::perspective(Deg(45.0f32), canvas_size.0 as f32 / canvas_size.1 as f32, 0.1f32, 50.0f32);
    let camera = Rc::new(RefCell::new(
        Camera::from_eye(
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, -0.5, -1.0),
            vec3(0.0, 1.0, 0.0)
        )));
    camera.borrow_mut().move_cam_locked(vec3(0.0, 10.0, 20.0));
    {
        clone!(camera);
        let callback = move |event: web_sys::WheelEvent|
            {
                if event.delta_y() > 0.0
                {
                    camera.borrow_mut().move_cam_long_locked(0.1);
                }
                else if event.delta_y() < 0.0
                {
                    camera.borrow_mut().move_cam_long_locked(-0.1);
                }
            };
        let ev = EventListener::new(&canvas, "wheel", callback).expect("zoom event listener");
        ev.forget();
    }

    {
        clone!(camera);
        let canvas_clone = canvas.clone();
        let callback = move |event: web_sys::MouseEvent|
            {
                if event.buttons() == 1
                {
                    canvas_clone.request_pointer_lock();
                    borrow_mut!(camera);
                    if event.movement_x() != 0
                    {
                        //camera.move_cam_lat(event.movement_x() as f32 / 800.0);
                        let delta = if event.movement_x() < 0 { -1.0 } else { 1.0 };
                        camera.rotate_world_yaw(delta);
                    }
                    if event.movement_y() != 0
                    {
                        //camera.move_cam_vert(event.movement_y() as f32 / 800.0);
                        let delta = if event.movement_y() < 0 { -1.0 } else { 1.0 };
                        camera.rotate_cam_pitch(delta);
                    }
                }
            };
        let ev = EventListener::new(&canvas, "mousemove", callback).expect("mouse move event listener");
        ev.forget();
    }

    let input_listener = Rc::new(InputStateListener::new(&canvas).expect("input state listener"));

    let script_str =
        {
            let name = "scripts/test_script.script".to_owned();
            let dir = resource_dir.to_owned() + &name;
            let resource = async_loader::load(dir).await.expect("downloaded test script");
            let string = String::from_utf8(resource.to_owned()).expect("script as string");
            borrow_mut!(resource_manager);
            resource_manager.insert_with_name(name, resource);

            string
        };
    let mut script = script::Script::new(&script_str).expect("script parsed");

    let render_func =
        {
            clone!(context, manager, camera);
            // Transformation objects for each robot being rendered. Key is robot ID
            let mut transformations: HashMap<String, Transformation, BuildHasherDefault<XxHash32>> = Default::default();

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
                        script.advance();
                        if script.is_done()
                        {
                            script.reset();
                        }
                        accumulator -= delta_time;
                    }

                    {
                        borrow!(context);
                        // Reset the render area
                        context.clear_color(0.0, 0.0, 0.0, 1.0);
                        context.clear(Context::COLOR_BUFFER_BIT | Context::DEPTH_BUFFER_BIT);


                        // Setup scene graph
                        let mut nodes =
                            vec![
                                Node(
                                    &skybox_renderable,
                                    skybox_transform.matrix(),
                                    Some(vec![

                                    ]),
                                )
                            ];

                        let script_data = script.current_data().expect("data available");

                        // Transformation matrices. Matrices from the transformations are copied
                        // into here to avoid borrowing errors since t.matrix() requires
                        // a mutable transformation
                        let mut matrices = Vec::with_capacity(script_data.len());

                        for data in script_data
                        {
                            // Either get the existing transformation for the robot or
                            // create a new one
                            let t = match transformations.get_mut(&data.id)
                            {
                                Some(t) => t,
                                None => transformations.entry(data.id.to_owned()).or_default()
                            };

                            // Apply the robot's position and orientation
                            t.global.set_translation(vec3(data.x_pos, 0.25, data.y_pos));
                            t.local.set_orientation_angle_axis(Deg(data.rotation), vec3(0.0, 1.0, 0.0));
                            // Add the transformation matrix
                            matrices.push(t.matrix().clone());
                        }

                        // Add the render nodes for the robots
                        for matrix in &matrices
                        {
                            nodes.push(Node(
                                &robot_renderable,
                                matrix,
                                None
                            ));
                        }

                        renderer.render(&context, &manager.borrow(), perspective * camera.borrow().view_matrix(), &nodes);
                    }

                    // Input state tests
                    borrow_mut!(camera);
                    let key_state = input_listener.key_state(Key_w);
                    if key_state == InputState::Down || key_state == InputState::Repeating
                    {
                        camera.move_cam_long_locked(-0.1);
                    }
                    let key_state = input_listener.key_state(Key_s);
                    if key_state == InputState::Down || key_state == InputState::Repeating
                    {
                        camera.move_cam_long_locked(0.1);
                    }
                    let key_state = input_listener.key_state(Key_a);
                    if key_state == InputState::Down || key_state == InputState::Repeating
                    {
                        camera.move_cam_lat(0.1);
                    }
                    let key_state = input_listener.key_state(Key_d);
                    if key_state == InputState::Down || key_state == InputState::Repeating
                    {
                        camera.move_cam_lat(-0.1);
                    }
                    let key_state = input_listener.key_state(Key_Space);
                    if key_state == InputState::Down || key_state == InputState::Repeating
                    {
                        camera.move_cam_vert_locked(0.1);
                    }
                    let key_state = input_listener.key_state(Key_Control);
                    if key_state == InputState::Down || key_state == InputState::Repeating
                    {
                        camera.move_cam_vert_locked(-0.1);
                    }

                }
        };

    // Setup and start render loop
    let render_loop = Rc::new(RefCell::new(RenderLoop::init(&window, &canvas, &context, &manager, render_func, context_config_func).expect("render_loop")));
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

#[wasm_bindgen(start)]
pub fn main_function() -> Result<(), JsValue>
{
    #[cfg(feature="debug")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

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