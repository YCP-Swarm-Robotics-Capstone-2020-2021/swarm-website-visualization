use crate::gfx::{Context, gl_get_error, gl_object::GLObject};
use std::rc::Rc;
use web_sys::{WebGlBuffer};
use paste::paste;

pub struct Buffer
{
    internal: WebGlBuffer,
    buffer_type: u32,
    context: Rc<Context>
}

macro_rules! buffer_fn
{
    ($type:ty, $js_array:path) =>
    {paste!
    {
        pub fn [<buffer_data_ $type>](&self, data: &[$type], draw_type: u32)
        {
            unsafe
                {
                    let buff = $js_array::view(&data);
                    self.context.buffer_data_with_array_buffer_view(self.buffer_type, &buff, draw_type);
                }
        }

        pub fn [<buffer_sub_data_ $type>](&self, offset: i32, data: &[$type])
        {
            unsafe
                {
                    let buff = $js_array::view(&data);
                    self.context.buffer_sub_data_with_i32_and_array_buffer_view(self.buffer_type, offset, &buff);
                }
        }
    }}
}

impl Buffer
{
    pub fn new(context: &Rc<Context>, buffer_type: u32) -> Result<Buffer, String>
    {
        Ok(Buffer
        {
            internal: context.create_buffer().ok_or_else(|| format!("Error creating buffer: {}", gl_get_error(context)))?,
            buffer_type,
            context: Rc::clone(context)
        })
    }

    buffer_fn!(f32, js_sys::Float32Array);
    buffer_fn!(i32, js_sys::Int32Array);
    buffer_fn!(u32, js_sys::Uint32Array);

    pub fn bind_range(&self, index: u32, offset: i32, size: i32)
    {
        self.context.bind_buffer_range_with_i32_and_i32(self.buffer_type, index, Some(&self.internal), offset, size)
    }
}

impl GLObject for Buffer
{
    fn bind(&self)
    {
        self.context.bind_buffer(self.buffer_type, Some(&self.internal));
    }

    fn unbind(&self)
    {
        self.context.bind_buffer(self.buffer_type, None);
    }
}

impl Drop for Buffer
{
    fn drop(&mut self)
    {
        self.context.delete_buffer(Some(&self.internal));
    }
}