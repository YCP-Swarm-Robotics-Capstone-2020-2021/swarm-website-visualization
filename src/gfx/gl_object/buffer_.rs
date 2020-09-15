
trait Buffer
{
    /// Set the contents of the buffer to `data`
    /// `draw_type` is one of the webgl `*_DRAW` enum types
    fn buffer_data<T>(&mut self, data: &[T], draw_type: u32);

    /// Set the contents of the buffer to `data`
    /// `draw_type` is one of the webgl `*_DRAW` enum types
    fn buffer_data_raw(&mut self, data: &[u8], draw_type: u32);

    /// Set the contents of the buffer to `data`, starting at `offset`
    /// `draw_type` is one of the webgl `*_DRAW` enum types
    fn buffer_sub_data<T>(&mut self, offset: i32, data: &[T]);

    /// Set the contents of the buffer to `data`, starting at `offset`
    /// `draw_type` is one of the webgl `*_DRAW` enum types
    fn buffer_sub_data_raw(&mut self, offset: i32, data: &[u8]);

    /// Bind `index` to the buffer memory range `offset`->`offset+size`
    fn bind_range(&mut self, index: u32, offset: i32, size: i32);
}

#[derive(Debug, Copy, Clone)]
pub(in crate::gfx::gl_object) struct RangeBinding(u32, i32, i32);

macro_rules! inject_buffer_fields
{
    () =>
    {
        internal: WebGlBuffer,
        context: Context,
        draw_type: u32,
        buffer: Vec<u8>,
        range_bindings: Vec<Option<RangeBinding>>
    }
}

macro_rules! impl_buffer
{
    (@new_internal $context:ident) =>
    {{
        $context.create_buffer().ok_or_else(|| GfxError::BufferCreationError(gl_get_errors($context).to_string()))
    }}
    (@as_u8_slice $data:ident) =>
    {{
        unsafe { std::slice::from_raw_parts($data.as_ptr() as *const u8, $data.len() * std::mem::size_of::<T>()) }
    }}
    ($name:ident, $_type:tt, $buffer_type:tt) =>
    {paste::paste!
    {
        pub struct $_type { inject_buffer_fields!(); }
        impl $_type
        {
            pub fn new(context: &Context, manager: &mut GlObjectManager, buffer_type: u32) -> Result<GlObjectHandle, GfxError>
            {
                let buffer = Buffer
                {
                    internal: Buffer::new_buffer(&context)?,
                    context: context.clone(),
                    buffer_type,
                    draw_type: 0,
                    buffer: vec![],
                    range_bindings: vec![]
                };
                Ok(manager.[<insert_ $name>](buffer))
            }
        }
        impl_buffer!(@trait_only $_type, $buffer_type);
    }}
    (@trait_only $name:ident, $_type:tt, $buffer_type:tt) =>
    {
        impl Buffer for $_type
        {
            /// Set the contents of the buffer to `data`
            /// `draw_type` is one of the webgl `*_DRAW` enum types
            pub fn buffer_data<T>(&mut self, data: &[T], draw_type: u32)
            {
                self.buffer_data_raw(impl_buffer!(@as_u8_slice data), draw_type);
            }

            /// Set the contents of the buffer to `data`
            /// `draw_type` is one of the webgl `*_DRAW` enum types
            pub fn buffer_data_raw(&mut self, data: &[u8], draw_type: u32)
            {
                self.buffer = data.to_vec();
                self.context.buffer_data_with_u8_array($buffer_type, &self.buffer, draw_type);
                self.draw_type = draw_type;
            }

            /// Set the contents of the buffer to `data`, starting at `offset`
            /// `draw_type` is one of the webgl `*_DRAW` enum types
            pub fn buffer_sub_data<T>(&mut self, offset: i32, data: &[T])
            {
                self.buffer_sub_data_raw(offset, impl_buffer!(@as_u8_slice data));
            }

            /// Set the contents of the buffer to `data`, starting at `offset`
            /// `draw_type` is one of the webgl `*_DRAW` enum types
            pub fn buffer_sub_data_raw(&mut self, offset: i32, data: &[u8])
            {
                self.context.buffer_sub_data_with_i32_and_u8_array($buffer_type, offset, data);
                self.buffer.splice(offset as usize..(offset as usize + data.len()), data.to_vec());
            }

            /// Bind `index` to the buffer memory range `offset`->`offset+size`
            pub fn bind_range(&mut self, index: u32, offset: i32, size: i32)
            {
                self.context.bind_buffer_range_with_i32_and_i32($buffer_type, index, Some(&self.internal), offset, size);

                if self.range_bindings.len() <= index as usize
                {
                    self.range_bindings.resize_with(index as usize + 1, || None);
                }
                self.range_bindings[index as usize] = Some(RangeBinding(index, offset, size));
            }
        }

        impl Bindable for Buffer
            {
                fn bind_internal(&self)
                {
                    self.context.bind_buffer($buffer_type, Some(&self.internal));
                }
                fn unbind_internal(&self)
                {
                    self.context.bind_buffer($buffer_type, None);
                }
            }

            impl Reloadable for Buffer
            {
                fn reload(&mut self, context: &Context) -> Result<(), GfxError>
                {
                    self.context = context.clone();
                    self.internal = Buffer::new_buffer(&self.context)?;
                    self.bind_internal();
                    self.context.buffer_data_with_u8_array($buffer_type, &self.buffer, self.draw_type);

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
    }
}