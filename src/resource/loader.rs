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
pub type RequestHandle = usize;
struct HttpRequest
{
    // Handle
    internal: XmlHttpRequest,
    method: String,
    url: String,
    // Events
    onabort: Option<RequestClosure>,
    onerror: Option<RequestClosure>,
    onload: Option<RequestClosure>,
    onloadstart: Option<RequestClosure>,
    onloadend: Option<RequestClosure>,
    onprogress: Option<RequestClosure>,
}
impl HttpRequest
{
    fn new(method: String, url: String) -> Result<HttpRequest, JsValue>
    {
        Ok(HttpRequest
        {
            internal: XmlHttpRequest::new()?,
            method, url,
            onabort: None, onerror: None,
            onload: None, onloadstart: None, onloadend: None, onprogress: None
        })
    }
}
impl Drop for HttpRequest
{
    fn drop(&mut self)
    {
        crate::log("http request dropped");
    }
}

pub struct ResourceLoader
{
    cb: Option<Closure<dyn FnMut(JsValue)>>,

    requests_left: Cell<i32>,
    // Total work that is already done
    work_total: Cell<f64>,
    // Total work to be done across all resources
    work_loaded: Cell<f64>,
    global_onloadend: Cell<Option<Box<dyn FnOnce()>>>,
    global_onprogress: Option<Box<dyn FnMut(f64, f64)>>,
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
            requests_left: Cell::new(0),
            work_total: Cell::new(0.0),
            work_loaded: Cell::new(0.0),
            global_onloadend: Cell::new(None),
            global_onprogress: None,
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
        self.global_onprogress = Some(Box::new(callback));
    }

    pub fn add_request(&mut self, method: impl Into<String>, url: impl Into<String>) -> Result<RequestHandle, JsValue>
    {
        self.http_requests.push(HttpRequest::new(method.into(), url.into())?);
        self.requests_left.set(self.requests_left.get() + 1);
        Ok(self.http_requests.len()-1)
    }

    pub fn set_request_onabort<F>(&mut self, handle: RequestHandle, mut onabort: F)
        -> bool where F: 'static + FnOnce(XmlHttpRequest, ProgressEvent)
    {
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            clone!(http_request.internal);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onabort(internal, event);
                });
            http_request.onabort = Some(closure);
            http_request.internal.set_onabort(Some(http_request.onabort.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    pub fn set_request_onerror<F>(&mut self, handle: RequestHandle, mut onerror: F)
        -> bool where F: 'static + FnOnce(XmlHttpRequest, ProgressEvent)
    {
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            clone!(http_request.internal);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onerror(internal, event);
                });
            http_request.onerror = Some(closure);
            http_request.internal.set_onerror(Some(http_request.onerror.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    pub fn set_request_onload<F>(&mut self, handle: RequestHandle, mut onload: F)
        -> bool where F: 'static + FnOnce(XmlHttpRequest, ProgressEvent)
    {
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            clone!(http_request.internal);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onload(internal, event);
                });
            http_request.onload = Some(closure);
            http_request.internal.set_onload(Some(http_request.onload.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    pub fn set_request_onloadstart<F>(&mut self, handle: RequestHandle, mut onloadstart: F)
        -> bool where F: 'static + FnOnce(XmlHttpRequest, ProgressEvent)
    {
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            clone!(http_request.internal, self.work_total);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    if event.length_computable()
                    {
                        work_total.set(work_total.get() + event.total());
                    }
                    onloadstart(internal, event);
                });
            http_request.onloadstart = Some(closure);
            http_request.internal.set_onloadstart(Some(http_request.onloadstart.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    //  This is also where the overall "when everything is done" callback trigger is setup
    pub fn set_request_onloadend<F>(&mut self, handle: RequestHandle, mut onloadend: F)
        -> bool where F: 'static + FnOnce(XmlHttpRequest, ProgressEvent)
    {
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            clone!(http_request.internal);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onloadend(internal, event);
                });
            http_request.onloadend = Some(closure);
            http_request.internal.set_onloadend(Some(http_request.onloadend.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    pub fn set_request_onprogress<F>(&mut self, handle: RequestHandle, mut onprogress: F)
        -> bool where F: 'static + FnMut(ProgressEvent)
    {
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            let closure = Closure::wrap(Box::new(move |event: ProgressEvent|
                {
                    onprogress(event);
                }) as Box<dyn FnMut(_)>);
            http_request.onprogress = Some(closure);
            http_request.internal.set_onprogress(Some(http_request.onprogress.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    /// Add a resource to be loaded
    // TODO: Return error(s)
    pub fn submit(self)
    {
        let mut loader = self;
        let mut http_requests = vec![];
        std::mem::swap(&mut loader.http_requests, &mut http_requests);
        let mut loader = Rc::new(RefCell::new(loader));

        for http_request in &mut http_requests
        {
            let closure =
                {
                    let onloadend = http_request.onloadend.take();
                    clone!(loader);
                    Closure::once(move |event: ProgressEvent|
                        {
                            if let Some(mut onloadend) = onloadend
                            {
                                // TODO: Error checking instead of calling unwrap()
                                onloadend.into_js_value().dyn_into::<js_sys::Function>().unwrap().call1(&JsValue::null(), &event).expect("function call");
                            }

                            let loader_borrow = loader.borrow();
                            let left = loader_borrow.requests_left.get();
                            loader_borrow.requests_left.set(left - 1);

                            if left <= 1
                            {
                                if let Some(mut global_onloadend) = loader_borrow.global_onloadend.take()
                                {
                                    global_onloadend();
                                }
                            }
                        })
                };
            http_request.onloadend = Some(closure);
            http_request.internal.set_onloadend(Some(http_request.onloadend.as_ref().unwrap().as_ref().unchecked_ref()));

            let closure =
                {
                    let onprogress = http_request.onprogress.take();

                    let onprogress = if let Some(closure) = onprogress
                    {
                        Some(closure.into_js_value().dyn_into::<js_sys::Function>().unwrap())
                    }
                    else { None };
                    let onprogress = Rc::new(RefCell::new(onprogress));
                    let loader = Rc::downgrade(&loader);
                    clone!(onprogress);
                    Closure::wrap(Box::new(move |event: ProgressEvent|
                        {
                            if let Some(mut onprogress) = onprogress.borrow_mut().as_mut()
                            {
                                // TODO: Error checking instead of calling unwrap()
                                onprogress.call1(&JsValue::null(), &event).expect("function call");
                            }

                            if event.length_computable()
                            {
                                if let Some(loader) = loader.upgrade()
                                {
                                    borrow_mut!(loader);
                                    let loaded = loader.work_loaded.get() + event.loaded();
                                    loader.work_loaded.set(loaded);
                                    let work_total = loader.work_total.get();
                                    if let Some(global_onprogress) = loader.global_onprogress.as_mut()
                                    {
                                        global_onprogress(loaded, work_total);
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(_)>)
                };
            http_request.onprogress = Some(closure);
            http_request.internal.set_onprogress(Some(http_request.onprogress.as_ref().unwrap().as_ref().unchecked_ref()));

            http_request.internal.open(&http_request.method, &http_request.url).expect("request opened");
            http_request.internal.send().expect("request sent");
        }
        loader.borrow_mut().http_requests = http_requests;
    }
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