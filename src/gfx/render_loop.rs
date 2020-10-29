use wasm_bindgen::
{
    prelude::*,
    JsCast,
};
use std::
{
    rc::Rc,
    cell::{RefCell, Cell}
};
use web_sys::
{
    Window,
    HtmlCanvasElement,
};
use crate::
{
    input::listener::EventListener,
    gfx::
    {
        Context,
        new_context,
        GfxError,
        GlError,
        gl_get_errors,
        gl_object::manager::GlObjectManager,
    },
};

pub struct RenderLoop
{
    window: Window,
    canvas: HtmlCanvasElement,
    context: Rc<RefCell<Context>>,
    context_lost_ev: Option<EventListener>,
    context_restored_ev: Option<EventListener>,
    valid_context: Rc<Cell<bool>>,
    globject_manager: Rc<RefCell<GlObjectManager>>,
    // Is the render loop running
    running: Rc<Cell<bool>>,
    // request_animation_frame() callback that calls given render func
    raf_callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
    // handle from each request_animation_frame() call
    raf_handle: Rc<Cell<i32>>,
    context_config: Rc<RefCell<Box<dyn FnMut(&Context)>>>
}

impl RenderLoop
{
    /// Initialize a new `RenderLoop`
    /// `RenderLoop` will call `GlObjectManager`'s reload function in the event of a context loss
    /// `context_config` is a function to setup/configure the context before the reload occurs
    pub fn init<F: 'static + FnMut(), FC: 'static + FnMut(&Context)>(
        window: &Window,
        canvas: &HtmlCanvasElement,
        context: &Rc<RefCell<Context>>,
        globject_manager: &Rc<RefCell<GlObjectManager>>,
        render_func: F,
        context_config: FC,
    ) -> Result<RenderLoop, GfxError>
    {
        let mut render_loop = RenderLoop
        {
            window: window.clone(),
            canvas: canvas.clone(),
            context: context.clone(),
            context_lost_ev: None,
            context_restored_ev: None,
            valid_context: Rc::new(Cell::new(true)),
            globject_manager: globject_manager.clone(),
            running: Rc::new(Cell::new(false)),
            raf_callback: Rc::new(RefCell::new(None)),
            raf_handle: Rc::new(Cell::new(-1)),
            context_config: Rc::new(RefCell::new(Box::new(context_config))),
        };
        render_loop.init_on_context_lost();
        render_loop.init_on_context_restored();

        render_loop.set_render_func(render_func)?;

        Ok(render_loop)
    }

    /// Initialize the callback for a context lost even
    fn init_on_context_lost(&mut self)
    {
        clone!(self.valid_context);
        let ev = EventListener::new(&self.canvas, "webglcontextlost",
                                    move |event: web_sys::WebGlContextEvent|
                                        {
                                            event.prevent_default();
                                            valid_context.set(false);
                                            crate::log("WebGlContext lost");
                                        }
        ).expect("webglcontextlost event listener registered");
        self.context_lost_ev = Some(ev);
    }

    /// Initialize the callback for a context restored event
    fn init_on_context_restored(&mut self)
    {
        let callback =
            {
                clone!(self.canvas, self.context, self.valid_context, self.globject_manager, self.context_config);
                move |_event: web_sys::WebGlContextEvent|
                    {
                        let mut context = context.borrow_mut();
                        // Update context
                        *context = new_context(&canvas).unwrap();

                        (&mut *context_config.borrow_mut())(&context);

                        // Recreate and reload all given GlObjects with new context
                        globject_manager.borrow_mut().reload_objects(&context);

                        // Print out any webgl errors
                        if let GfxError::GlErrors(errors) = gl_get_errors(&context)
                        {
                            if errors[0] != GlError::NoError
                            {
                                crate::log_s(format!("{:?}", errors));
                            }
                        }

                        valid_context.set(true);
                        crate::log("WebGlContext restored");
                    }
            };
        let ev = EventListener::new(&self.canvas, "webglcontextrestored", callback)
            .expect("webglcontextrestored event listener registered");
        self.context_restored_ev = Some(ev);
    }

    /// Starts the render loop
    /// An error is returned if the loop is already running
    /// or `cleanup()` has already been called
    pub fn start(&mut self) -> Result<(), GfxError>
    {
        if self.running.get()
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
            self.running.set(true);
            self.raf_handle.set(self.window.request_animation_frame(raf_callback.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap());
            Ok(())
        }
    }

    /// Pauses the execution of the render loop
    /// Restart the loop by calling `start()`
    #[allow(dead_code)]
    pub fn pause(&mut self) -> Result<(), GfxError>
    {
        if !self.running.get()
        {
            Err(GfxError::RenderLoopNotRunning)
        }
        else if self.raf_callback.borrow().is_none()
        {
            Err(GfxError::RenderLoopAlreadyCleanedUp)
        }
        else
        {
            self.running.set(false);
            self.window.cancel_animation_frame(self.raf_handle.get()).expect("cancel animation frame");
            Ok(())
        }
    }

    /// Permanently stop the render loop, freeing the loop callback
    pub fn cleanup(&mut self)
    {
        self.running.set(false);
        self.window.cancel_animation_frame(self.raf_handle.get()).expect("cancel animation frame");
        self.raf_handle.set(-1);
        let _ = self.raf_callback.borrow_mut().take();
    }

    /// Set a new render func, cleaning up the previous render func
    #[allow(dead_code)]
    pub fn set_render_func<T: 'static + FnMut()>(&mut self, mut render_func: T) -> Result<(), GfxError>
    {
        if self.running.get()
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
                            if !running.get()
                            {
                                return;
                            }
                            else if valid_context.get()
                            {
                                render_func();
                            }

                            raf_handle.set(window.request_animation_frame(raf_callback.borrow().as_ref().unwrap().as_ref().unchecked_ref()).expect("raf handle"));
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