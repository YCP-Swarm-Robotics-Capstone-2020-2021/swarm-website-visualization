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
            clone!(http_request.internal);
            let closure = Closure::once(move |event: ProgressEvent|
                {
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
            clone!(http_request.internal);
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
                    // Has the work total for this request been set
                    let mut set_total = false;
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
                                    // Add the work total into the overall work total if it hasn't been
                                    //      already
                                    if !set_total
                                    {
                                        loader.work_total.set(loader.work_total.get() + event.total());
                                        set_total = true;
                                    }

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

#[cfg(test)]
mod tests
{
    inject_wasm_test_boilerplate!();
    use wasm_bindgen_futures::JsFuture;
    use js_sys::Promise;

    use crate::
    {
        resource::
        {
            loader::*
        }
    };
    use std::{rc::Rc, cell::{RefCell, Cell}};

    // https://www.reddit.com/r/rust/comments/cpxjlw/wasmasync_discoveris_about_sleepawait_via/
    pub async fn timer(ms: i32) -> Result<(), JsValue> {
        let promise = Promise::new(&mut |yes, _| {
            let win = window().unwrap();
            win.set_timeout_with_callback_and_timeout_and_arguments_0(&yes, ms)
                .unwrap();
        });
        let js_fut = JsFuture::from(promise);
        js_fut.await?;
        Ok(())
    }

    #[wasm_bindgen_test]
    async fn test_onabort()
    {
        let mut resource_loader = ResourceLoader::new();

        let request_handle = resource_loader.add_request("GET", "/build.py").unwrap();

        let abort_executed = Rc::new(Cell::new(false));

        {
            clone!(abort_executed);
            resource_loader.set_request_onabort(request_handle, move |_|
            {
                abort_executed.set(true);
            });

            resource_loader.set_request_onloadstart(request_handle, move |CallbackArgs(_, request)|
                {
                    request.abort();
                });
        }
        resource_loader.submit();

        // Wait max of 100ms for request to be carried out
        let mut counter = 0;
        while !abort_executed.get() && counter < 100
        {
            counter += 1;
            timer(1).await.unwrap();
        }
        assert!(abort_executed.get());
    }

    #[wasm_bindgen_test]
    async fn test_onerror()
    {
        let mut resource_loader = ResourceLoader::new();

        let request_handle = resource_loader.add_request("GET", "/nonexistentfile.txt").unwrap();

        let error_executed = Rc::new(Cell::new(false));

        {
            clone!(error_executed);
            resource_loader.set_request_onerror(request_handle, move |_|
                {
                    error_executed.set(true);
                });

            // Trigger the event
            resource_loader.set_request_onload(request_handle, move |OnloadCallbackArgs(CallbackArgs(_, request), _)|
                {
                    request.dispatch_event(&Event::new("error").unwrap());
                });
        }
        resource_loader.submit();


        // Wait max of 100ms for request to be carried out
        let mut counter = 0;
        while !error_executed.get() && counter < 100
        {
            counter += 1;
            timer(1).await.unwrap();
        }
        assert!(error_executed.get());
    }

    #[wasm_bindgen_test]
    async fn test_onload()
    {
        let mut resource_loader = ResourceLoader::new();

        let expected_contents = String::from(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/build.py")));
        let request_handle = resource_loader.add_request("GET", "/build.py").unwrap();

        let done = Rc::new(Cell::new(false));
        let contents = Rc::new(Cell::new(None));

        {
            clone!(done, contents);
            resource_loader.set_request_onload(request_handle, move |OnloadCallbackArgs(_, response)|
                {
                    contents.set(Some(String::from_utf8(response).unwrap()));
                    done.set(true);
                });
        }
        resource_loader.submit();

        // Wait max of 100ms for request to be carried out
        let mut counter = 0;
        while !done.get() && counter < 100
        {
            counter += 1;
            timer(1).await.unwrap();
        }
        assert_eq!(Some(expected_contents), contents.take());
    }

    #[wasm_bindgen_test]
    async fn test_onloadstart()
    {
        let mut resource_loader = ResourceLoader::new();

        let request_handle = resource_loader.add_request("GET", "/build.py").unwrap();

        let loadstart_executed = Rc::new(Cell::new(false));

        {
            clone!(loadstart_executed);
            resource_loader.set_request_onloadstart(request_handle, move |_|
                {
                    loadstart_executed.set(true);
                });
        }
        resource_loader.submit();

        // Wait max of 100ms for request to be carried out
        let mut counter = 0;
        while !loadstart_executed.get() && counter < 100
        {
            counter += 1;
            timer(1).await.unwrap();
        }
        assert!(loadstart_executed.get());
    }

    #[wasm_bindgen_test]
    async fn test_onloadend()
    {
        let mut resource_loader = ResourceLoader::new();

        let request_handle = resource_loader.add_request("GET", "/build.py").unwrap();

        let loadend_executed = Rc::new(Cell::new(false));

        {
            clone!(loadend_executed);
            resource_loader.set_request_onloadend(request_handle, move |_|
                {
                    loadend_executed.set(true);
                });
        }
        resource_loader.submit();

        // Wait max of 100ms for request to be carried out
        let mut counter = 0;
        while !loadend_executed.get() && counter < 100
        {
            counter += 1;
            timer(1).await.unwrap();
        }
        assert!(loadend_executed.get());
    }

    #[wasm_bindgen_test]
    async fn test_onprogress()
    {
        let mut resource_loader = ResourceLoader::new();

        let request_handle = resource_loader.add_request("GET", "/build.py").unwrap();

        let onprogress_executed = Rc::new(Cell::new(false));
        let work_total = Rc::new(Cell::new(0.0));

        {
            clone!(onprogress_executed, work_total);
            resource_loader.set_request_onprogress(request_handle, move |CallbackArgs(event, _)|
                {
                    onprogress_executed.set(true);
                    work_total.set(event.total());
                });
        }
        resource_loader.submit();

        // Wait max of 100ms for request to be carried out
        let mut counter = 0;
        while !onprogress_executed.get() && counter < 100
        {
            counter += 1;
            timer(1).await.unwrap();
        }
        assert!(onprogress_executed.get());
        assert!(work_total.get() > 0.0);
    }

    #[wasm_bindgen_test]
    async fn test_global_onloadend()
    {
        let mut resource_loader = ResourceLoader::new();

        resource_loader.add_request("GET", "/build.py").unwrap();
        resource_loader.add_request("GET", "/nonexistentfile.txt").unwrap();

        let global_loadend_executed = Rc::new(Cell::new(false));

        {
            clone!(global_loadend_executed);
            resource_loader.set_onloadend(move ||
                {
                    global_loadend_executed.set(true);
                });
        }
        resource_loader.submit();

        // Wait max of 100ms for request to be carried out
        let mut counter = 0;
        while !global_loadend_executed.get() && counter < 100
        {
            counter += 1;
            timer(1).await.unwrap();
        }
        assert!(global_loadend_executed.get());
    }

    #[wasm_bindgen_test]
    async fn test_global_onprogress()
    {
        let mut resource_loader = ResourceLoader::new();

        resource_loader.add_request("GET", "/build.py").unwrap();
        resource_loader.add_request("GET", "/build.py").unwrap();

        let work_loaded = Rc::new(Cell::new(0.0));
        let work_total = Rc::new(Cell::new(0.0));
        let num_executions = Rc::new(Cell::new(0));

        {
            clone!(work_loaded, work_total, num_executions);
            resource_loader.set_onprogress(move |current, total|
                {
                    work_loaded.set(current);
                    work_total.set(total);
                    crate::log_s(format!("{} - {}", current, total));
                    num_executions.set(num_executions.get() + 1);
                });
        }
        resource_loader.submit();

        // Wait max of 100ms for request to be carried out
        let mut counter = 0;
        while !num_executions.get() < 2 && counter < 100
        {
            counter += 1;
            timer(1).await.unwrap();
        }
        assert_eq!(num_executions.get(), 2);
        assert!(work_total.get() > 0.0);
        assert!(work_loaded.get() > 0.0);
    }
}