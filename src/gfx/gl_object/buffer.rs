
pub trait Buffer : crate::gfx::gl_object::traits::GlObject
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

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Hash)]
pub struct RangeBinding(pub u32, pub i32, pub i32);

/// Implements the buffer trait for either a pre-existing struct or
/// creates a new struct and implements the buffer trait for it
// TODO: `buffer_type` and `struct_name` could possibly be condensed into one parameter,
//  `buffer_type` could possibly be used as `[<$buffer_type:camel>]` for the struct name
// TODO: The Reloadable and Bindable trait implementations currently don't allow any customization
//  for pre-existing structs. This isn't required at the moment of writing, but it may
//  become an issue in the future
macro_rules! impl_buffer
{
    // Creates a new internal webgl buffer object from `context: Context`
    // This is for use within the macro only
    (@new_internal $context:ident) =>
    {{
        $context.create_buffer().ok_or_else(|| crate::gfx::GfxError::BufferCreationError(crate::gfx::gl_get_errors($context).to_string()))
    }};
    // Gets `data :T` as a u8 slice
    (@as_u8_slice $data:ident) =>
    {{
        unsafe { std::slice::from_raw_parts($data.as_ptr() as *const u8, $data.len() * std::mem::size_of::<T>()) }
    }};
    // Initialize a new `struct_name`'s required buffer fields and any additional fields
    (@init_struct $context:ident, $struct_name:ident {$($field:ident:$value:expr),*}) =>
    {{
        $struct_name
        {
            internal: impl_buffer!(@new_internal $context)?,
            context: $context.clone(),
            draw_type: 0,
            buffer: vec![],
            range_bindings: vec![],
            $(
            $field: $value,
            )*
        }
    }};
    // Defines `struct_name` with the required fields for a buffer
    // and implements a `new` function for `struct_name` along with the `Buffer` trait
    // This is for creating a buffer subtype that only differs from others because
    // of its buffer type. i.e. ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER and not UNIFORM_BUFFER
    //
    // `struct_name` should be the name of the struct to be created
    // `buffer_type` should be one of the webgl literals such as Context::VERTEX_ARRAY_BUFFER
    ($buffer_type:ident, $struct_name:ident) =>
    {paste::paste!
    {
        impl_buffer!($buffer_type, $struct_name {});
        impl $struct_name
        {
            pub fn new(context: &crate::gfx::Context) -> Result<$struct_name, crate::gfx::GfxError>
            {
                Ok(impl_buffer!(@init_struct context, $struct_name {}))
            }
        }
    }};

    // Creates a new `struct_name` with the required fields for a buffer as well as
    // any additionally specified fields and implements the `Buffer` trait
    ($buffer_type:ident, $struct_name:ident {$($field:ident:$type:ty),*}) =>
    {
        pub struct $struct_name
        {
            internal: web_sys::WebGlBuffer,
            context: crate::gfx::Context,
            draw_type: u32,
            buffer: Vec<u8>,
            range_bindings: Vec<Option<crate::gfx::gl_object::buffer::RangeBinding>>,
            $(
                $field: $type,
            )*
        }
        impl_buffer!(@trait_only $buffer_type, $struct_name);
    };

    // Implements the `Buffer` trait for a pre-existing struct `struct_name`
    //
    // `buffer_type` should be one of the webgl literals such as Context::VERTEX_ARRAY_BUFFER
    (@trait_only $buffer_type:tt, $struct_name:ident) =>
    {
        impl crate::gfx::gl_object::buffer::Buffer for $struct_name
        {
            /// Set the contents of the buffer to `data`
            /// `draw_type` is one of the webgl `*_DRAW` enum types
            fn buffer_data<T>(&mut self, data: &[T], draw_type: u32)
            {
                self.buffer_data_raw(impl_buffer!(@as_u8_slice data), draw_type);
            }

            /// Set the contents of the buffer to `data`
            /// `draw_type` is one of the webgl `*_DRAW` enum types
            fn buffer_data_raw(&mut self, data: &[u8], draw_type: u32)
            {
                self.buffer = data.to_vec();
                self.context.buffer_data_with_u8_array(crate::gfx::Context::$buffer_type, &self.buffer, draw_type);
                self.draw_type = draw_type;
            }

            /// Set the contents of the buffer to `data`, starting at `offset`
            /// `draw_type` is one of the webgl `*_DRAW` enum types
            fn buffer_sub_data<T>(&mut self, offset: i32, data: &[T])
            {
                self.buffer_sub_data_raw(offset, impl_buffer!(@as_u8_slice data));
            }

            /// Set the contents of the buffer to `data`, starting at `offset`
            /// `draw_type` is one of the webgl `*_DRAW` enum types
            fn buffer_sub_data_raw(&mut self, offset: i32, data: &[u8])
            {
                self.context.buffer_sub_data_with_i32_and_u8_array(crate::gfx::Context::$buffer_type, offset, data);
                self.buffer.splice(offset as usize..(offset as usize + data.len()), data.to_vec());
            }

            /// Bind `index` to the buffer memory range `offset`->`offset+size`
            fn bind_range(&mut self, index: u32, offset: i32, size: i32)
            {
                self.context.bind_buffer_range_with_i32_and_i32(crate::gfx::Context::$buffer_type, index, Some(&self.internal), offset, size);

                if self.range_bindings.len() <= index as usize
                {
                    self.range_bindings.resize_with(index as usize + 1, || None);
                }
                self.range_bindings[index as usize] = Some(crate::gfx::gl_object::buffer::RangeBinding(index, offset, size));
            }
        }

        impl_globject!($struct_name);

        impl crate::gfx::gl_object::traits::Bindable for $struct_name
            {
                fn bind_internal(&self)
                {
                    self.context.bind_buffer(crate::gfx::Context::$buffer_type, Some(&self.internal));
                }
                fn unbind_internal(&self)
                {
                    self.context.bind_buffer(crate::gfx::Context::$buffer_type, None);
                }
            }

            impl crate::gfx::gl_object::traits::Reloadable for $struct_name
            {
                fn reload(&mut self, context: &crate::gfx::Context, _manager: &crate::gfx::gl_object::manager::GlObjectManager) -> Result<(), crate::gfx::GfxError>
                {
                    self.context = context.clone();
                    self.internal = impl_buffer!(@new_internal context)?;
                    self.bind_internal();
                    self.context.buffer_data_with_u8_array(crate::gfx::Context::$buffer_type, &self.buffer, self.draw_type);

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

            impl Drop for $struct_name
            {
                fn drop(&mut self)
                {
                    self.context.delete_buffer(Some(&self.internal));
                }
            }
    };
}


#[cfg(test)]
mod tests
{
    inject_wasm_test_boilerplate!();

    use crate::gfx::
    {
        gl_object::
        {
            traits::Bindable,
            buffer::{Buffer, RangeBinding},
            ArrayBuffer,
        },
    };

    #[wasm_bindgen_test]
    fn test_buffer_data()
    {
        let context = get_context();
        let mut buffer = ArrayBuffer::new(&context).expect("array buffer");
        buffer.bind_internal();
        assert_eq!(GfxError::GlErrors(vec![GlError::NoError]), gl_get_errors(&context));

        let mut buff: [u8; 4] = [0, 1, 2, 3];
        buffer.buffer_data(&buff, Context::STATIC_DRAW);
        assert_eq!(&buff, buffer.buffer.as_slice());

        // TODO: This fails, is it because of the testing environment or
        //  is there actually a bug somewhere?
        assert_eq!(GfxError::GlErrors(vec![GlError::NoError]), gl_get_errors(&context));

        buff[1] = 4;
        buffer.buffer_sub_data(1, &[4u8]);
        assert_eq!(&buff, buffer.buffer.as_slice());
    }

    #[wasm_bindgen_test]
    fn test_range_bindings()
    {
        let context = get_context();
        let mut buffer = ArrayBuffer::new(&context).expect("array buffer");
        buffer.bind_internal();
        buffer.bind_range(0, 0, 1);

        assert_eq!(Some(RangeBinding(0, 0, 1)), buffer.range_bindings[0]);
    }
}