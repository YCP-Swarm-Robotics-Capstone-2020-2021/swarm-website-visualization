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

}

impl ResourceLoader
{
    // Create empty resource loader
    fn new() -> Self
    {
        ResourceLoader {}
    }
    // Add an overall "oncomplete" function to the loader
    fn with_oncomplete(self) -> Self
    {

    }
    // Add an overall "onprogress" function to the loader
    fn with_onprogress(self) -> Self
    {

    }
    // Add a resource to be loaded
    fn with_resource(self) -> Self
    {
        self
    }
    fn submit(&mut self)
    {

    }
}