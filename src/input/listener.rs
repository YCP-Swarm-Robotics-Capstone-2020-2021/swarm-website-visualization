use wasm_bindgen::
{
    prelude::*,
    JsCast,
};
use std::cell::Cell;

pub struct EventListener
{
    target: web_sys::EventTarget,
    name: String,
    callback: Cell<Option<Closure<dyn FnMut(JsValue)>>>,
}

impl EventListener
{
    /// Creates a new managed JS event listener
    /// Essentially equivalent to `target.addEventListener(name, (event) => { /* ... */ })` in JS,
    /// but the event listener is removed when this `EventListener` is dropped or `forget()` is called
    pub fn new<T, F>(target: &web_sys::EventTarget, name: impl Into<String>, mut callback: F) -> Result<EventListener, JsValue>
        where T: 'static + From<JsValue>, F: 'static + FnMut(T)
    {
        let closure = Closure::wrap(Box::new(move |event: JsValue|
            {
                callback(event.into());
            }) as Box<dyn FnMut(_)>);
        let name = name.into();
        target.add_event_listener_with_callback(name.as_str(), closure.as_ref().unchecked_ref())?;
        Ok(EventListener { target: target.clone(), name, callback: Cell::new(Some(closure)) })
    }

    /// Releases Rust's handle to this JS callback,
    /// meaning that it is essentially active until page reload/session end
    /// even if this event listener object gets dropped
    pub fn forget(&self)
    {
        let callback = self.callback.take();
        if let Some(callback) = callback
        {
            callback.forget();
        }
    }
}

impl Drop for EventListener
{
    fn drop(&mut self)
    {
        if let Some(callback) = self.callback.take()
        {
            self.target.remove_event_listener_with_callback(self.name.as_str(), callback.as_ref().unchecked_ref())
                .expect(&format!("{} callback removed", self.name));
        }
    }
}