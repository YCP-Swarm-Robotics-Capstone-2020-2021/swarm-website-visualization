use wasm_bindgen::
{
    prelude::*,
    JsCast,
};
use std::
{
    rc::Rc,
    cell::RefCell
};
use web_sys::
{
    Window,
    HtmlCanvasElement,
};
use crate::gfx::
{
    Context,
    new_context,
    GfxError,
    GlError,
    gl_get_errors,
    gl_object::GlObject,
};

pub struct RenderLoop
{
    window: Rc<Window>,
    canvas: Rc<HtmlCanvasElement>,
    context: Rc<RefCell<Rc<Context>>>,
    // GlObjects to be stored during context loss recovery
    gl_objects: Rc<RefCell<Vec<Rc<RefCell<dyn GlObject>>>>>,
    valid_context: Rc<RefCell<bool>>,
    // Is the render loop running
    running: Rc<RefCell<bool>>,
    // request_animation_frame() callback that calls given render func
    raf_callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
    // handle from each request_animation_frame() call
    raf_handle: Rc<RefCell<i32>>
}

impl RenderLoop
{
    /// Initialize a new `RenderLoop`
    /// `RenderLoop` will call `GlObject::recreate_and_reload()` for each item in
    /// `globjects` in the even of a context loss
    pub fn init<T: 'static + FnMut()>(
        window: &Rc<Window>,
        canvas: &Rc<HtmlCanvasElement>,
        context: &Rc<RefCell<Rc<Context>>>,
        gl_objects: &Rc<RefCell<Vec<Rc<RefCell<dyn GlObject>>>>>,
        render_func: T
    ) -> Result<RenderLoop, GfxError>
    {
        let mut render_loop = RenderLoop
        {
            window: window.clone(),
            canvas: canvas.clone(),
            context: context.clone(),
            gl_objects: gl_objects.clone(),
            valid_context: Rc::new(RefCell::new(true)),
            running: Rc::new(RefCell::new(false)),
            raf_callback: Rc::new(RefCell::new(None)),
            raf_handle: Rc::new(RefCell::new(-1)),
        };
        render_loop.init_on_context_lost();
        render_loop.init_on_context_restored();

        render_loop.set_render_func(render_func)?;

        Ok(render_loop)
    }

    /// Initialize the callback for a context lost even
    fn init_on_context_lost(&self)
    {
        clone!(self.valid_context);
        let callback = Closure::wrap(Box::new(move |event: web_sys::WebGlContextEvent|
            {
                event.prevent_default();
                *valid_context.borrow_mut() = false;
            }
        ) as Box<dyn FnMut(_)>);
        self.canvas.add_event_listener_with_callback("webglcontextlost", callback.as_ref().unchecked_ref()).expect("webglcontextlost event listener added");
        callback.forget();
    }

    /// Initialize the callback for a context restored event
    fn init_on_context_restored(&self)
    {
        let callback =
            {
                clone!(self.canvas, self.context, self.gl_objects, self.valid_context);
                Closure::wrap(Box::new(move |_event: web_sys::WebGlContextEvent|
                    {
                        let mut context = context.borrow_mut();
                        // Update context
                        *context = new_context(&canvas).unwrap();

                        // Recreate and reload all given GlObjects with new context
                        for obj in gl_objects.borrow().iter()
                        {
                            obj.borrow_mut().recreate_and_reload(&context).expect("GlObject recreated and reloaded");
                        }

                        // Print out any webgl errors
                        if let GfxError::GlErrors(errors) = gl_get_errors(&context)
                        {
                            if errors[0] != GlError::NoError
                            {
                                crate::log_s(format!("{:?}", errors));
                            }
                        }

                        *valid_context.borrow_mut() = true;
                    }
                ) as Box<dyn FnMut(_)>)
            };
        self.canvas.add_event_listener_with_callback("webglcontextrestored", callback.as_ref().unchecked_ref()).expect("webglcontextrestored event listener added");
        callback.forget();
    }

    /// Starts the render loop
    /// An error is returned if the loop is already running
    /// or `cleanup()` has already been called
    pub fn start(&mut self) -> Result<(), GfxError>
    {
        if *self.running.borrow()
        {
            Err(GfxError::RenderLoopAlreadyRunning)
        }
        else if self.raf_callback.borrow().is_none()
        {
            Err(GfxError::RenderLoopAlreadyCleanedUp)
        }
        else
        {
            clone!(self.raf_callback);
            *self.running.borrow_mut() = true;
            *self.raf_handle.borrow_mut() = self.window.request_animation_frame(raf_callback.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
            Ok(())
        }
    }

    /// Pauses the execution of the render loop
    /// Restart the loop by calling `start()`
    #[allow(dead_code)]
    pub fn pause(&mut self) -> Result<(), GfxError>
    {
        if !*self.running.borrow()
        {
            Err(GfxError::RenderLoopNotRunning)
        }
        else if self.raf_callback.borrow().is_none()
        {
            Err(GfxError::RenderLoopAlreadyCleanedUp)
        }
        else
        {
            *self.running.borrow_mut() = false;
            self.window.cancel_animation_frame(*self.raf_handle.borrow()).expect("cancel animation frame");
            Ok(())
        }
    }

    /// Permanently stop the render loop, freeing the loop callback
    pub fn cleanup(&mut self)
    {
        *self.running.borrow_mut() = false;
        self.window.cancel_animation_frame(*self.raf_handle.borrow()).expect("cancel animation frame");
        *self.raf_handle.borrow_mut() = -1;
        let _ = self.raf_callback.borrow_mut().take();
    }

    /// Set a new render func, cleaning up the previous render func
    #[allow(dead_code)]
    pub fn set_render_func<T: 'static + FnMut()>(&mut self, mut render_func: T) -> Result<(), GfxError>
    {
        if *self.running.borrow()
        {
            Err(GfxError::RenderLoopAlreadyRunning)
        }
        else
        {
            self.cleanup();
            *self.raf_callback.borrow_mut() =
                {
                    clone!(self.window, self.valid_context, self.running, self.raf_callback, self.raf_handle);
                    Some(Closure::wrap(Box::new(move ||
                        {
                            if !*running.borrow()
                            {
                                return;
                            }
                            else if *valid_context.borrow()
                            {
                                render_func();
                            }

                            *raf_handle.borrow_mut() = window.request_animation_frame(raf_callback.borrow().as_ref().unwrap().as_ref().unchecked_ref()).expect("raf handle");
                        }
                    ) as Box<dyn FnMut()>))
                };

            Ok(())
        }
    }
}

impl Drop for RenderLoop
{
    fn drop(&mut self)
    {
        self.cleanup();
    }
}