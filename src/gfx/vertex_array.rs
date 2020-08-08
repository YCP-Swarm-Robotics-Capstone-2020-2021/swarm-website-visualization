/// WebGlVertexArrayObject wrapper

use crate::gfx::
{
    Context,
    gl_get_errors,
    gl_object::GLObject
};
use web_sys::WebGlVertexArrayObject;
use std::rc::Rc;

#[derive(Debug, Copy, Clone)]
struct AttribPointer(u32, i32, u32, bool, i32, i32);

pub struct VertexArray
{
    internal: WebGlVertexArrayObject,
    context: Rc<Context>,
    attrib_ptrs: Vec<Option<AttribPointer>>
}

impl VertexArray
{
    fn new_vertex_array(context: &Context) -> Result<WebGlVertexArrayObject, String>
    {
        context.create_vertex_array().ok_or_else(|| format!("Error creating vertex array: {}", gl_get_errors(context)))
    }

    pub fn new(context: &Rc<Context>) -> Result<VertexArray, String>
    {
        Ok(VertexArray
        {
            internal: VertexArray::new_vertex_array(&context)?,
            context: Rc::clone(context),
            attrib_ptrs: vec![]
        })
    }

    /// glVertexAttribPtr with default values of `false` for `normalized` and `size_of<T>()` for `stride`
    /// This function also enables the vertex attrib array at `index`
    pub fn attrib_ptr<T>(&mut self, index: u32, size: i32, data_type: u32, offset: i32)
    {
        self.attrib_ptr_raw(index, size, data_type, false, size * std::mem::size_of::<T>() as i32, offset);
    }

    /// glVertexAttribPtr with no default values
    /// This function also enables the vertex attrib array at `index`
    pub fn attrib_ptr_raw(&mut self, index: u32, size: i32, data_type: u32, normalized: bool, stride: i32, offset: i32)
    {
        self.context.vertex_attrib_pointer_with_i32(index, size, data_type, normalized, stride, offset);
        self.context.enable_vertex_attrib_array(index);

        if self.attrib_ptrs.len() <= index as usize
        {
            self.attrib_ptrs.resize_with(index as usize + 1, || None);
        }
        self.attrib_ptrs[index as usize] = Some(AttribPointer(index, size, data_type, normalized, stride, offset));
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
    type ReloadError = String;
    fn reload(&mut self, context: &Rc<Context>) -> Result<(), Self::ReloadError>
    {
        self.context = Rc::clone(&context);
        self.internal = VertexArray::new_vertex_array(&self.context)?;

        let attrib_ptrs = self.attrib_ptrs.to_owned();
        for attrib_ptr in attrib_ptrs
        {
            if let Some(attrib_ptr) = attrib_ptr
            {
                self.attrib_ptr_raw(
                    attrib_ptr.0,
                    attrib_ptr.1,
                    attrib_ptr.2,
                    attrib_ptr.3,
                    attrib_ptr.4,
                    attrib_ptr.5
                );
            }
        }
        Ok(())
    }
}

impl Drop for VertexArray
{
    fn drop(&mut self)
    {
        self.context.delete_vertex_array(Some(&self.internal));
    }
}