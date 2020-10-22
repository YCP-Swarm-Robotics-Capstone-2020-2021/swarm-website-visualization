use wasm_bindgen::
{
    prelude::*,
    JsCast,
};
use web_sys::
{
    FileReader,
};

pub struct ResourceLoader
{
    cb: Option<Closure<dyn FnMut(JsValue)>>
}

impl ResourceLoader
{
    // Create empty resource loader
    pub fn new() -> Self
    {
        ResourceLoader
        {
            cb: None
        }
    }
    // Add an overall "oncomplete" function to the loader
    pub fn with_oncomplete(self) -> Self
    {
        self
    }
    // Add an overall "onprogress" function to the loader
    pub fn with_onprogress(self) -> Self
    {
        self
    }
    // Add a resource to be loaded
    pub fn with_resource(self) -> Self
    {
        self
    }
    pub fn submit(self)
    {
        let rl = std::rc::Rc::new(std::cell::RefCell::new(self));
        let rl_c = rl.clone();
        let closure = Closure::wrap(Box::new(move |_event: JsValue|
            {
                {
                    std::mem::drop(rl_c.borrow_mut().cb.take());
                }
                crate::log("timeout");
            }) as Box<dyn FnMut(_)>);
        rl.borrow_mut().cb = Some(closure);
        web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(rl.borrow().cb.as_ref().unwrap().as_ref().unchecked_ref(), 2000);
        std::mem::drop(rl);
    }
}

impl Drop for ResourceLoader
{
    fn drop(&mut self)
    {
        crate::log("start drop");
        std::mem::drop(self.cb.take());
        crate::log("end drop");
    }
}