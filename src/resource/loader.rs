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
    cell::{RefCell, Cell},
    rc::Rc,
};

type RequestClosure = Closure<dyn FnMut(ProgressEvent)>;
struct HttpRequest
{
    // Handle
    http_request: XmlHttpRequest,
    // Events
    onabort: Option<RequestClosure>,
    onerror: Option<RequestClosure>,
    onload: Option<RequestClosure>,
    onloadstart: Option<RequestClosure>,
    onloadend: Option<RequestClosure>,
    onprogress: Option<RequestClosure>,
}

pub struct ResourceLoader
{
    cb: Option<Closure<dyn FnMut(JsValue)>>,

    requests_left: Rc<Cell<i32>>,
    // Total work that is already done
    work_total: Rc<Cell<f64>>,
    // Total work to be done across all resources
    work_loaded: Rc<Cell<f64>>,
    global_onloadend: Rc<Cell<Option<Box<dyn FnOnce()>>>>,
    global_onprogress: Rc<RefCell<Option<Box<dyn FnMut(f64, f64)>>>>,
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
            work_total: Rc::new(Cell::new(0.0)),
            work_loaded: Rc::new(Cell::new(0.0)),
            global_onloadend: Rc::new(Cell::new(None)),
            global_onprogress: Rc::new(RefCell::new(None)),
            http_requests: vec![],
        }
    }
    /// Add an overall "when everything is done" function to the loader
    pub fn set_onloadend<F>(&self, mut callback: F) where F: 'static + FnOnce()
    {
        //self.oncomplete = Some(Closure::wrap(Box::new(callback) as Box<dyn FnMut()>));
        self.global_onloadend.set(Some(Box::new(callback)));
    }
    /// Add an overall "onprogress" function to the loader
    ///
    /// First arg is amount of work already performed
    /// Second arg is total amount of work to be done
    pub fn set_onprogress<F>(&mut self, mut callback: F) where F: 'static + FnMut(f64, f64)
    {
        //self.onprogress = Some(Closure::wrap(Box::new(callback) as Box<dyn FnMut(_, _)>));
        *self.global_onprogress.borrow_mut() = Some(Box::new(callback));
    }
    /// Add a resource to be loaded
    pub fn add_resource<FO, FM>(&mut self,
                            resource_url: &str,
                            mut onabort: Option<FO>,
                            mut onerror: Option<FO>,
                            mut onload: Option<FO>,
                            mut onloadstart: Option<FO>,
                            mut onloadend: Option<FO>,
                            mut onprogress: Option<FM>
    ) -> Result<(), JsValue> where FO: 'static + FnOnce(XmlHttpRequest, ProgressEvent), FM: 'static + FnMut(ProgressEvent)
    {
        // Increment the counter of the number of requests being made
        let left = self.requests_left.get();
        self.requests_left.set(left + 1);

        // Initialize all callbacks to None
        let mut request = HttpRequest
        {
            http_request: XmlHttpRequest::new()?,
            onabort: None, onerror: None,
            onload: None, onloadstart: None, onloadend: None,
            onprogress: None,
        };

        // Setup onabort callback
        if let Some(mut onabort) = onabort
        {
            clone!(request.http_request);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onabort(http_request, event);
                });
            request.onabort = Some(closure);
        }

        // Setup onerror callback
        if let Some(mut onerror) = onerror
        {
            clone!(request.http_request);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onerror(http_request, event);
                });
            request.onerror = Some(closure);
        }

        // Setup onload callback
        if let Some(mut onload) = onload
        {
            clone!(request.http_request);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onload(http_request, event);
                });
            request.onload = Some(closure);
        }

        // Setup onloadstart callback
        if let Some(mut onloadstart) = onloadstart
        {
            clone!(request.http_request, self.work_total);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    if event.length_computable()
                    {
                        work_total.set(work_total.get() + event.total());
                    }
                    onloadstart(http_request, event);
                });
            request.onloadstart = Some(closure);
        }

        // Setup onloadend callback
        //  This is also where the overall "when everything is done" callback trigger is setup
        if let Some(mut onloadend) = onloadend
        {
            clone!(request.http_request, self.global_onloadend);
            let requests_left = self.requests_left.clone();
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    let left = requests_left.get();
                    if left <= 1
                    {
                        // TODO: cleanup/drop all memory here
                        if let Some(mut global_onloadend) = global_onloadend.take()
                        {
                            global_onloadend();
                        }
                    }
                    else
                    {
                        requests_left.set(left - 1);
                        onloadend(http_request, event);
                    }
                });
            request.onloadend = Some(closure);
        }

        if let Some(mut onprogress) = onprogress
        {
            clone!(self.global_onprogress, self.work_loaded, self.work_total);
            let closure = Closure::wrap(Box::new(move |event: ProgressEvent|
                {
                    if let Some(global_onprogress) = global_onprogress.borrow_mut().as_mut()
                    {
                        if event.length_computable()
                        {
                            let loaded = work_loaded.get() + event.loaded();
                            work_loaded.set(loaded);
                            global_onprogress(loaded, work_total.get());
                        }
                    }

                    onprogress(event);
                }) as Box<dyn FnMut(_)>);
            request.onprogress = Some(closure);
        }

        self.http_requests.push(request);
        Ok(())
    }

    pub fn submit(self)
    {

    }

/*    pub fn submit(self)
    {
        let rl = std::rc::Rc::new(std::cell::RefCell::new(self));
        let rl_c = rl.clone();
        let closure = Closure::wrap(Box::new(move |_event: JsValue|
            {
                {
                    rl_c.borrow_mut().cb.take();
                    // std::mem::drop(rl_c.borrow_mut().cb.take());
                }
                crate::log("timeout");
            }) as Box<dyn FnMut(_)>);
        rl.borrow_mut().cb = Some(closure);
        web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(rl.borrow().cb.as_ref().unwrap().as_ref().unchecked_ref(), 2000);
        std::mem::drop(rl);
    }*/
}

impl Drop for ResourceLoader
{
    fn drop(&mut self)
    {
        crate::log("start drop");
        // std::mem::drop(self.cb.take());
        crate::log("end drop");
    }
}