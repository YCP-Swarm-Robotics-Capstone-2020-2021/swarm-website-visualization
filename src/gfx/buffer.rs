use std::rc::Rc;
use web_sys::{WebGlBuffer};
use paste::paste;
use gen_vec::Index;
use crate::gfx::
{
    Context,
    GlManager,
    GfxError,
    gl_get_errors,
    gl_object::GLObject,
};
use crate::gfx::GfxError::BufferCreationError;

/// A single item in the buffer
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
enum BufferContents
{
    #[allow(non_camel_case_types)]
    f32(f32),
    #[allow(non_camel_case_types)]
    i32(i32),
    #[allow(non_camel_case_types)]
    u32(u32)
}
#[derive(Debug, Copy, Clone)]
struct RangeBinding(u32, i32, i32);

pub struct Buffer
{
    internal: WebGlBuffer,
    context: Rc<Context>,
    buffer_type: u32,
    draw_type: u32,
    buffer: Vec<BufferContents>,
    range_bindings: Vec<Option<RangeBinding>>
}

/// Creates `buffer_data_$type` and `buffer_sub_data_$type` functions
/// `$type` is the primitive type associated with `$js_array`
/// i.e. f32 for Float32Array
/// @conv Convert a buffer of `$type` into a buffer of BufferContents
macro_rules! buffer_fn
{
    (@conv $type:ident, $data:ident) => {{ $data.iter().map(|i| BufferContents::$type(*i)).collect::<Vec<BufferContents>>() }};
    ($type:ty, $js_array:path) =>
    {paste!
    {
        #[allow(dead_code)]
        pub fn [<buffer_data_ $type>](&mut self, data: &[$type], draw_type: u32)
        {
            unsafe
                {
                    let buff = $js_array::view(&data);
                    self.context.buffer_data_with_array_buffer_view(self.buffer_type, &buff, draw_type);
                }
            self.draw_type = draw_type;
            self.buffer = buffer_fn!(@conv $type, data);
        }

        #[allow(dead_code)]
        pub fn [<buffer_sub_data_ $type>](&mut self, offset: i32, data: &[$type])
        {
            unsafe
                {
                    let buff = $js_array::view(&data);
                    self.context.buffer_sub_data_with_i32_and_array_buffer_view(self.buffer_type, offset, &buff);
                }
            self.buffer.splice(offset as usize..offset as usize+data.len()-1, buffer_fn!(@conv $type, data));
        }
    }}
}

impl Buffer
{
    fn new_buffer(context: &Context) -> Result<WebGlBuffer, GfxError>
    {
        context.create_buffer().ok_or_else(|| GfxError::BufferCreationError(gl_get_errors(context)))
    }

    pub fn new(manager: &GlManager, buffer_type: u32) -> Result<Index, GfxError>
    {
        let buffer = Buffer
        {
            internal: Buffer::new_buffer(&manager.context())?,
            context: Rc::clone(&manager.context()),
            buffer_type,
            draw_type: 0,
            buffer: vec![],
            range_bindings: vec![]
        };
        Ok(manager.add_gl_object(buffer))
    }

    buffer_fn!(f32, js_sys::Float32Array);
    buffer_fn!(i32, js_sys::Int32Array);
    buffer_fn!(u32, js_sys::Uint32Array);

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

/// Convert a `BufferContents` vec to a vec of bytes
macro_rules! buffer_contents_to_bytes
{
    (@conv $buff:expr, $conv_func:ident, $($variant:ident),+) =>
    {{
        let mut out_buff: Vec<u8> = Vec::with_capacity($buff.len() * std::mem::size_of::<BufferContents>());
        for i in &$buff
        {
            match i
            {
            $(
                BufferContents::$variant(contents) => out_buff.extend_from_slice(&contents.$conv_func()),
            )+
            }
        }
        out_buff
    }};
    ($buff:expr) =>
    {{
        if cfg!(target_endian = "big")
        {
            buffer_contents_to_bytes!(@conv $buff, to_be_bytes, f32, i32, u32)
        }
        else
        {
            buffer_contents_to_bytes!(@conv $buff, to_le_bytes, f32, i32, u32)
        }
    }}
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
    fn reload(&mut self, context: &Rc<Context>) -> Result<(), GfxError>
    {
        self.context = Rc::clone(&context);
        self.internal = Buffer::new_buffer(&self.context)?;
        self.bind();

        let range_bindings = self.range_bindings.to_owned();
        for range_binding in range_bindings
        {
            if let Some(range_binding) = range_binding
            {
                self.bind_range(range_binding.0, range_binding.1, range_binding.2);
            }
        }

        let bytes = buffer_contents_to_bytes!(self.buffer);
        self.context.buffer_data_with_u8_array(self.buffer_type, &bytes, self.draw_type);

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