use wasm_bindgen::
{
    prelude::*,
    JsCast,
};
use web_sys::
{
    XmlHttpRequest,
    ProgressEvent,
};
use std::
{
    cell::{Cell},
    rc::Rc,
};

type ProgressClosure = Closure<dyn FnMut(ProgressEvent)>;
struct HttpRequest
{
    // Handle
    http_request: XmlHttpRequest,
    // Events
    onabort: Option<ProgressClosure>,
    onerror: Option<ProgressClosure>,
    onload: Option<ProgressClosure>,
    onloadstart: Option<ProgressClosure>,
    onloadend: Option<ProgressClosure>,
    onprogress: Option<ProgressClosure>,
}

pub struct ResourceLoader
{
    cb: Option<Closure<dyn FnMut(JsValue)>>,

    requests_left: Rc<Cell<i32>>,
    oncomplete: Option<Closure<dyn FnMut()>>,
    onprogress: Option<Closure<dyn FnMut(i32, i32)>>,
    http_requests: Vec<HttpRequest>,
}

impl ResourceLoader
{
    /// Create empty resource loader
    pub fn new() -> Self
    {
        ResourceLoader
        {
            cb: None,
            requests_left: Rc::new(Cell::new(0)),
            oncomplete: None,
            onprogress: None,
            http_requests: vec![],
        }
    }
    /// Add an overall "when everything is done" function to the loader
    pub fn set_oncomplete<F>(&mut self, mut callback: F) where F: 'static + FnMut()
    {
        self.oncomplete = Some(Closure::wrap(Box::new(callback) as Box<dyn FnMut()>));
    }
    /// Add an overall "onprogress" function to the loader
    ///
    /// First arg is amount of work already performed
    /// Second arg is total amount of work to be done
    pub fn set_onprogress<F>(&mut self, mut callback: F) where F: 'static + FnMut(i32, i32)
    {
        self.onprogress = Some(Closure::wrap(Box::new(callback) as Box<dyn FnMut(_, _)>));
    }
    /// Add a resource to be loaded
    pub fn add_resource<F>(&mut self,
                           resource_url: &str,
                           mut onabort: Option<F>,
                           mut onerror: Option<F>,
                           mut onload: Option<F>,
                           mut onloadstart: Option<F>,
                           mut onloadend: Option<F>,
                           mut onprogress: Option<F>
    ) -> Result<(), JsValue> where F: 'static + FnMut(ProgressEvent)
    {
        let left = self.requests_left.get();
        self.requests_left.set(left + 1);
        let mut request = HttpRequest
        {
            http_request: XmlHttpRequest::new()?,
            onabort: None, onerror: None,
            onload: None, onloadstart: None, onloadend: None,
            onprogress: None,
        };
        if let Some(onabort) = onabort
        {
            let closure = Closure::wrap(Box::new(onabort) as Box<dyn FnMut(_)>);
            request.onabort = Some(closure);
        }
        if let Some(onerror) = onerror
        {
            let closure = Closure::wrap(Box::new(onerror) as Box<dyn FnMut(_)>);
            request.onerror = Some(closure);
        }
        if let Some(onload) = onerror
        {
            let closure = Closure::wrap(Box::new(onload) as Box<dyn FnMut(_)>);
            request.onload = Some(closure);
        }
        if let Some(onloadstart) = onloadstart
        {
            let closure = Closure::wrap(Box::new(onloadstart) as Box<dyn FnMut(_)>);
            request.onloadstart = Some(closure);
        }
        if let Some(onloadend) = onloadend
        {
            let request_left = self.requests_left.clone();
            let closure = Closure::wrap(Box::new(move |event: ProgressEvent|
                {
                    let left = self.requests_left.get();
                    if left <= 1
                    {
                        // TODO: cleanup/drop all memory here
                    }
                    else
                    {
                        self.requests_left.set(left - 1);
                        onloadend(event);
                    }
                }) as Box<dyn FnMut(_)>);
        }
        if let Some(onprogress) = onprogress
        {

        }

        Ok(())
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