use wasm_bindgen::
{
    prelude::*,
    JsCast,
};
use web_sys::
{
    XmlHttpRequest,
    XmlHttpRequestResponseType,
    ProgressEvent,
};
use js_sys::
{
    ArrayBuffer,
    Uint8Array,
};
use std::
{
    cell::{RefCell, Cell},
    rc::Rc,
};

/// Type alias for the format of a closure for a request
type RequestClosure = Closure<dyn FnMut(ProgressEvent)>;
/// Handle for a new request added into the resource loader
pub type RequestHandle = usize;
/// Handle for XmlHttpRequest
/// This owns the closures for all of a XmlHttpRequest's callbacks so they can be cleaned
/// up when the request is complete
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
        // TODO: Remove when done testing
        crate::log("http request dropped");
    }
}

pub struct CallbackArgs(pub ProgressEvent, pub XmlHttpRequest);
pub struct OnloadCallbackArgs(pub CallbackArgs, pub Vec<u8>);

/// Resource Loader
///
/// This is for loading resources from a URL. It's intended for loading visualization assets but
/// it can be used for any sort of XmlHttpRequest
pub struct ResourceLoader
{
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
            requests_left: Cell::new(0),
            work_total: Cell::new(0.0),
            work_loaded: Cell::new(0.0),
            global_onloadend: Cell::new(None),
            global_onprogress: None,
            http_requests: vec![],
        }
    }
    /// Add an overall "when everything is done" function to the loader
    #[allow(dead_code)]
    pub fn set_onloadend<F>(&self, callback: F) where F: 'static + FnOnce()
    {
        self.global_onloadend.set(Some(Box::new(callback)));
    }
    /// Add an overall "onprogress" function to the loader
    ///
    /// First arg of `callback` is amount of work already performed
    /// Second arg of `callback` is total amount of work to be done
    #[allow(dead_code)]
    pub fn set_onprogress<F>(&mut self, callback: F) where F: 'static + FnMut(f64, f64)
    {
        self.global_onprogress = Some(Box::new(callback));
    }

    /// Add a new resource request
    ///
    /// `method` is the HTTP method to use (GET, POST, etc)
    /// `url` is the resource URL
    /// Returns a handle to the request for use in assigning callbacks
    #[allow(dead_code)]
    pub fn add_request(&mut self, method: impl Into<String>, url: impl Into<String>) -> Result<RequestHandle, JsValue>
    {
        // Set the response type to Arraybuffer so that the response can be read into a byte array
        let request = HttpRequest::new(method.into(), url.into())?;
        request.internal.set_response_type(XmlHttpRequestResponseType::Arraybuffer);
        // Add the new request
        self.http_requests.push(request);
        // Increment the number of requests that are going to be executed
        self.requests_left.set(self.requests_left.get() + 1);

        Ok(self.http_requests.len()-1)
    }

    /// Set the `onabort` event callback for a request
    #[allow(dead_code)]
    pub fn set_request_onabort<F>(&mut self, handle: RequestHandle, onabort: F)
        -> bool where F: 'static + FnOnce(CallbackArgs)
    {
        // Make sure the handle is valid
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            // Create the closure
            clone!(http_request.internal);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onabort(CallbackArgs(event, internal));
                });
            // Assign the closure to the request and it's internal JS object
            http_request.onabort = Some(closure);
            http_request.internal.set_onabort(Some(http_request.onabort.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    /// Set the `onerror` event callback for a request
    #[allow(dead_code)]
    pub fn set_request_onerror<F>(&mut self, handle: RequestHandle, onerror: F)
        -> bool where F: 'static + FnOnce(CallbackArgs)
    {
        // Make sure the handle is valid
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            // Create the closure
            clone!(http_request.internal);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onerror(CallbackArgs(event, internal));
                });
            // Assign the closure to the request and it's internal JS object
            http_request.onerror = Some(closure);
            http_request.internal.set_onerror(Some(http_request.onerror.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    /// Set the `onload` event callback for a request
    #[allow(dead_code)]
    pub fn set_request_onload<F>(&mut self, handle: RequestHandle, onload: F)
        -> bool where F: 'static + FnOnce(OnloadCallbackArgs)
    {
        // Make sure the handle is valid
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            // Create the closure
            clone!(http_request.internal);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    let mut response_vec: Vec<u8> = vec![];
                    match internal.response()
                    {
                        Ok(response) =>
                            {
                                let buffer: ArrayBuffer = response.into();
                                let byte_arr = Uint8Array::new(&buffer);
                                response_vec = byte_arr.to_vec();
                            },
                        Err(err) =>
                            {
                                crate::log_s(format!("Error getting request response: {:?}", err));
                            }
                    }
                    onload(OnloadCallbackArgs(CallbackArgs(event, internal), response_vec));
                });
            // Assign the closure to the request and it's internal JS object
            http_request.onload = Some(closure);
            http_request.internal.set_onload(Some(http_request.onload.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    /// Set the `onloadstart` event callback for a request
    #[allow(dead_code)]
    pub fn set_request_onloadstart<F>(&mut self, handle: RequestHandle, onloadstart: F)
        -> bool where F: 'static + FnOnce(CallbackArgs)
    {
        // Make sure the handle is valid// Make sure the handle is valid// Make sure the handle is valid// Make sure the handle is valid
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            // Create the closure// Create the closure// Create the closure// Create the closure
            clone!(http_request.internal, self.work_total);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    if event.length_computable()
                    {
                        work_total.set(work_total.get() + event.total());
                    }
                    onloadstart(CallbackArgs(event, internal));
                });
            // Assign the closure to the request and it's internal JS object// Assign the closure to the request and it's internal JS object// Assign the closure to the request and it's internal JS object// Assign the closure to the request and it's internal JS object
            http_request.onloadstart = Some(closure);
            http_request.internal.set_onloadstart(Some(http_request.onloadstart.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    /// Set the `onloadend` event callback for a request
    #[allow(dead_code)]
    pub fn set_request_onloadend<F>(&mut self, handle: RequestHandle, onloadend: F)
        -> bool where F: 'static + FnOnce(CallbackArgs)
    {
        // Make sure the handle is valid
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
            // Create the closure
            clone!(http_request.internal);
            let closure = Closure::once(move |event: ProgressEvent|
                {
                    onloadend(CallbackArgs(event, internal));
                });
            // Assign the closure to the request and it's internal JS object
            http_request.onloadend = Some(closure);
            http_request.internal.set_onloadend(Some(http_request.onloadend.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    /// Set the `onprogress` event callback for a request
    #[allow(dead_code)]
    pub fn set_request_onprogress<F>(&mut self, handle: RequestHandle, mut onprogress: F)
        -> bool where F: 'static + FnMut(CallbackArgs)
    {
        // Make sure the handle is valid
        if let Some(http_request) = self.http_requests.get_mut(handle)
        {
           // Create the closure
            let internal = http_request.internal.clone();
            let closure = Closure::wrap(Box::new(move |event: ProgressEvent|
                {
                    onprogress(CallbackArgs(event, internal.clone()));
                }) as Box<dyn FnMut(_)>);
            // Assign the closure to the request and it's internal JS object
            http_request.onprogress = Some(closure);
            http_request.internal.set_onprogress(Some(http_request.onprogress.as_ref().unwrap().as_ref().unchecked_ref()));

            true
        }
        else { false }
    }

    /// Add a resource to be loaded
    #[allow(dead_code)]
    pub fn submit(self)
    {
        let mut loader = self;
        // Move the http_requests out of the loader so that the requests can be mutably iterated
        //      over without conflicting with the loader being borrowed/mutably borrowed
        let mut http_requests = vec![];
        std::mem::swap(&mut loader.http_requests, &mut http_requests);
        // Wrap the loader in a Rc<RefCell<_>> so that it can be used within the closures
        let loader = Rc::new(RefCell::new(loader));

        for http_request in &mut http_requests
        {
            // New closure to wrap the `onloadend` event, call the `global_onloadend` callback,
            //      and make sure that everything stays alive until it is no longer needed
            let closure =
                {
                    // Move the request's `onloadend` callback out of the struct
                    //      so that it can be dropped once it has executed
                    let onloadend = http_request.onloadend.take();
                    clone!(loader);
                    Closure::once(move |event: ProgressEvent|
                        {
                            // If the user provided a callback, execute it
                            if let Some(onloadend) = onloadend
                            {
                                // Convert the closure into a JS function and call it
                                let onloadend: js_sys::Function = onloadend.into_js_value().into();
                                if let Err(err) = onloadend.call1(&JsValue::null(), &event)
                                {
                                    crate::log_s(format!("Error executing request onloadend func: {:?}", err))
                                }
                            }

                            let loader_borrow = loader.borrow();
                            // Decrement the amount of requests left to execute
                            let left = loader_borrow.requests_left.get();
                            loader_borrow.requests_left.set(left - 1);

                            // If this was the last request being executed
                            if left <= 1
                            {
                                // Then call the user's overall `onloadend` callback if it exists
                                //      This is what keeps the ResourceLoader alive throughout
                                //      all of the request executions, and after this expression
                                //      the ResourceLoader and all of the requests are dropped
                                if let Some(global_onloadend) = loader_borrow.global_onloadend.take()
                                {
                                    global_onloadend();
                                }
                            }
                        })
                };
            // Assign the new closure to it's request and internal JS object
            http_request.onloadend = Some(closure);
            http_request.internal.set_onloadend(Some(http_request.onloadend.as_ref().unwrap().as_ref().unchecked_ref()));

            // New closure to wrap the `onprogress` event and call the `global_onprogress` callback
            let closure =
                {
                    // Take the `onprogress` closure out of the request so that we can use it
                    let onprogress = http_request.onprogress.take();

                    // Convert the closure into a JS function if the closure exists
                    let onprogress: Option<js_sys::Function> = if let Some(closure) = onprogress
                    {
                        Some(closure.into_js_value().into())
                    }
                    else { None };
                    // Wrap the callback in Rc<RefCell<_>> since we need to call it multiple times
                    let onprogress = Rc::new(RefCell::new(onprogress));
                    // Downgrade the loader Rc to a Weak reference so that it doesn't keep
                    //      the loader alive after `global_onloadend` has been called
                    let loader = Rc::downgrade(&loader);
                    clone!(onprogress);
                    Closure::wrap(Box::new(move |event: ProgressEvent|
                        {
                            // If the user provided a callback, execute it
                            if let Some(onprogress) = onprogress.borrow_mut().as_mut()
                            {
                                if let Err(err) = onprogress.call1(&JsValue::null(), &event)
                                {
                                    crate::log_s(format!("Error executing request onprogress func: {:?}", err))
                                }
                            }

                            // Include this in the global progress tracking only if it is actually
                            //      able to be tracked
                            if event.length_computable()
                            {
                                // Attempt to access the loader. If this is false, then
                                //     `global_onloadend` has already been called. Technically
                                //      that will never actually happen, since this closure
                                //      will have been dropped right after `global_onloadend`
                                //      executes, however doing this allows the memory to properly
                                //      be cleaned up
                                if let Some(loader) = loader.upgrade()
                                {
                                    borrow_mut!(loader);
                                    // Get the total amount of work that has been completed by
                                    //      all requests
                                    let loaded = loader.work_loaded.get() + event.loaded();
                                    loader.work_loaded.set(loaded);

                                    let work_total = loader.work_total.get();
                                    // Call the user's `global_onprogress` callback if it exists
                                    if let Some(global_onprogress) = loader.global_onprogress.as_mut()
                                    {
                                        global_onprogress(loaded, work_total);
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(_)>)
                };
            // Assign the new closure to it's request and internal JS object
            http_request.onprogress = Some(closure);
            http_request.internal.set_onprogress(Some(http_request.onprogress.as_ref().unwrap().as_ref().unchecked_ref()));

            // Setup the request with the specified HTTP method and resource URL
            http_request.internal.open(&http_request.method, &http_request.url).expect("request opened");
            // Send the request
            http_request.internal.send().expect("request sent");
        }
        // Now that all of the requests have been processed, pass them back into the ResourceLoader
        //      so that they can be cleaned up when everything is finished
        loader.borrow_mut().http_requests = http_requests;
    }
}

impl Drop for ResourceLoader
{
    fn drop(&mut self)
    {
        // TODO: Remove when done testing
        crate::log("start drop");
        crate::log("end drop");
    }
}