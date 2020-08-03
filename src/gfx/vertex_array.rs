use crate::gfx::
{
    Context,
    gl_get_error,
    gl_object::GLObject
};
use web_sys::WebGlVertexArrayObject;
use std::rc::Rc;

pub struct VertexArray
{
    internal: WebGlVertexArrayObject,
    context: Rc<Context>
}

impl VertexArray
{
    pub fn new(context: &Rc<Context>) -> Result<VertexArray, String>
    {
        Ok(VertexArray
        {
            internal: context.create_vertex_array().ok_or_else(|| format!("Error creating vertex array: {}", gl_get_error(context)))?,
            context: Rc::clone(context)
        })
    }

    /// glVertexAttribPtr with default values of `false` for `normalized` and `size_of<T>()` for `stride`
    pub fn attrib_ptr<T>(&self, index: u32, size: i32, data_type: u32, offset: i32)
    {
        self.attrib_ptr_raw(index, size, data_type, false, size * std::mem::size_of::<T>() as i32, offset);
    }

    pub fn attrib_ptr_raw(&self, index: u32, size: i32, data_type: u32, normalized: bool, stride: i32, offset: i32)
    {
        self.context.vertex_attrib_pointer_with_i32(index, size, data_type, normalized, stride, offset);
    }
}

impl GLObject for VertexArray
{
    fn bind(&self)
    {
        self.context.bind_vertex_array(Some(&self.internal));
    }

    fn unbind(&self)
    {
        self.context.bind_vertex_array(None);
    }
}

impl Drop for VertexArray
{
    fn drop(&mut self)
    {
        self.context.delete_vertex_array(Some(&self.internal));
    }
}