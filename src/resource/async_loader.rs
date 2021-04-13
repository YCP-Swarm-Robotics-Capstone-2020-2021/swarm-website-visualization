use wasm_bindgen::
{
    prelude::*,
    JsCast,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::
{
    Request,
    RequestInit,
    RequestMode,
    Response,
};
use js_sys::
{
    ArrayBuffer,
    Uint8Array,
};

#[derive(Debug)]
pub enum AsyncResourceError
{
    JsValueErr(JsValue),
    StrErr(&'static str),
}
impl From<JsValue> for AsyncResourceError
{
    fn from(js_val: JsValue) -> Self { AsyncResourceError::JsValueErr(js_val) }
}
impl From<&'static str> for AsyncResourceError
{
    fn from(s: &'static str) -> Self { AsyncResourceError::StrErr(s) }
}

/// Download the contents from `resource_url` and load it into memory as a byte vector
pub async fn load<S: AsRef<str>>(resource_url: S) -> Result<Vec<u8>, AsyncResourceError>
{
    // Set the request as a CORS GET request
    let mut req_opts = RequestInit::new();
    req_opts.method("GET");
    // TODO: Should the request be CORS?
    req_opts.mode(RequestMode::Cors);

    // Create the request
    let request = Request::new_with_str_and_init(resource_url.as_ref(), &req_opts)
        .expect("New fetch API request object");
    let window = web_sys::window().expect("Window context");
    // Create a Future out of the fetch request
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await
        .expect("Promise resolved");

    let resp: Response = resp_value.dyn_into()
        .or(Err("Response is not 'Request' type"))?;

    if resp.ok()
    {
        // Get the response body as a byte buffer
        let body_buffer: ArrayBuffer = JsFuture::from(resp.array_buffer()?).await?.into();
        let bytes_buffer = Uint8Array::new(&body_buffer);

        Ok(bytes_buffer.to_vec())
    }
    else
    {
        Err(AsyncResourceError::StrErr("Failed to fetch resource"))
    }
}

/// Asynchronously download the resource located at each of the `resource_urls`
/// The returned `Vec` contains either the resource as a byte buffer, or an error that
/// occurred while downloading the resource. This `Vec` is in the same order as the
/// `resource_urls`. i.e. index 0 of the return `Vec` is the result for index 0 of
/// `resource_urls`
pub async fn load_multiple<S: ToString>(resource_urls: &Vec<S>)
                                        -> Vec<Result<Vec<u8>, AsyncResourceError>>
{
    let mut tasks = Vec::with_capacity(resource_urls.len());

    for url in resource_urls
    {
        let url = url.to_string();
        tasks.push(async move {
            self::load(&url).await
        });
    }

    futures::future::join_all(tasks).await
}