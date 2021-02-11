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

pub struct AsyncResourceLoader
{

}

impl AsyncResourceLoader
{
    pub async fn load(resource_url: &str) -> Result<Vec<u8>, AsyncResourceError>
    {
        let mut req_opts = RequestInit::new();
        req_opts.method("GET");
        req_opts.mode(RequestMode::Cors);

/*        let request = Request::new_with_str_and_init(&resource_url, &req_opts)
            .expect("New fetch API request object");*/
        let request = Request::new_with_str_and_init(&resource_url, req_opts)
            .expect("New Request object");

        let window = web_sys::window().expect("Window context");
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await
            .expect("Promise resolved");

        let resp: Response = resp_value.dyn_into()
            .or(Err("Response is not 'Request' type"))?;
        // TODO: Check for error

        let body_buffer: ArrayBuffer = JsFuture::from(resp.array_buffer()?).await?.into();
        let bytes_buffer = Uint8Array::new(&body_buffer);

        Ok(bytes_buffer.to_vec())
    }
}
