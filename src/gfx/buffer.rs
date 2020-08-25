use web_sys::{WebGlBuffer};
use crate::gfx::
{
    Context,
    GfxError,
    gl_get_errors,
    gl_object::GlObject,
};

#[derive(Debug, Copy, Clone)]
struct RangeBinding(u32, i32, i32);

pub struct Buffer
{
    internal: WebGlBuffer,
    context: Context,
    buffer_type: u32,
    draw_type: u32,
    buffer: Vec<u8>,
    range_bindings: Vec<Option<RangeBinding>>
}

fn as_u8_slice<T>(data: &[T]) -> &[u8]
{
    unsafe
        {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<T>()
            )
        }
}

impl Buffer
{
    fn new_buffer(context: &Context) -> Result<WebGlBuffer, GfxError>
    {
        context.create_buffer().ok_or_else(|| GfxError::BufferCreationError(gl_get_errors(context).to_string()))
    }

    pub fn new(context: &Context, buffer_type: u32) -> Result<Buffer, GfxError>
    {
        Ok(Buffer
        {
            internal: Buffer::new_buffer(&context)?,
            context: context.clone(),
            buffer_type,
            draw_type: 0,
            buffer: vec![],
            range_bindings: vec![]
        })
    }

    /// Set the contents of the buffer to `data`
    /// `draw_type` is one of the webgl `*_DRAW` enum types
    pub fn buffer_data<T>(&mut self, data: &[T], draw_type: u32)
    {
        self.buffer_data_raw(as_u8_slice(data), draw_type);
    }

    /// Set the contents of the buffer to `data`
    /// `draw_type` is one of the webgl `*_DRAW` enum types
    pub fn buffer_data_raw(&mut self, data: &[u8], draw_type: u32)
    {
        self.buffer = data.to_vec();
        self.context.buffer_data_with_u8_array(self.buffer_type, &self.buffer, draw_type);
        self.draw_type = draw_type;
    }

    /// Set the contents of the buffer to `data`, starting at `offset`
    /// `draw_type` is one of the webgl `*_DRAW` enum types
    pub fn buffer_sub_data<T>(&mut self, offset: i32, data: &[T])
    {
        self.buffer_sub_data_raw(offset, as_u8_slice(data));
    }

    /// Set the contents of the buffer to `data`, starting at `offset`
    /// `draw_type` is one of the webgl `*_DRAW` enum types
    pub fn buffer_sub_data_raw(&mut self, offset: i32, data: &[u8])
    {
        self.context.buffer_sub_data_with_i32_and_u8_array(self.buffer_type, offset, data);
        self.buffer.splice(offset as usize..(offset as usize + data.len()), data.to_vec());
    }

    /// Bind `index` to the buffer memory range `offset`->`offset+size`
    pub fn bind_range(&mut self, index: u32, offset: i32, size: i32)
    {
        self.context.bind_buffer_range_with_i32_and_i32(self.buffer_type, index, Some(&self.internal), offset, size);

        if self.range_bindings.len() <= index as usize
        {
            self.range_bindings.resize_with(index as usize + 1, || None);
        }
        self.range_bindings[index as usize] = Some(RangeBinding(index, offset, size));
    }
}

impl GlObject for Buffer
{
    fn bind(&self)
    {
        self.context.bind_buffer(self.buffer_type, Some(&self.internal));
    }
    fn unbind(&self)
    {
        self.context.bind_buffer(self.buffer_type, None);
    }
    fn recreate(&mut self, context: &Context) -> Result<(), GfxError>
    {
        self.context = context.clone();
        self.internal = Buffer::new_buffer(&self.context)?;
        Ok(())
    }
    fn reload(&mut self) -> Result<(), GfxError>
    {
        self.bind();
        self.context.buffer_data_with_u8_array(self.buffer_type, &self.buffer, self.draw_type);

        let range_bindings = self.range_bindings.to_owned();
        for range_binding in range_bindings
        {
            if let Some(range_binding) = range_binding
            {
                self.bind_range(range_binding.0, range_binding.1, range_binding.2);
            }
        }

        Ok(())
    }
}

impl Drop for Buffer
{
    fn drop(&mut self)
    {
        self.context.delete_buffer(Some(&self.internal));
    }
}