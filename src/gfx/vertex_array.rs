/// WebGlVertexArrayObject wrapper

use crate::gfx::
{
    Context,
    GfxError,
    gl_get_errors,
    gl_object::GlObject,
    buffer::Buffer,
};
use web_sys::WebGlVertexArrayObject;
use gen_vec::{Index, exposed::{IndexAllocator, ExposedGenVec}};

#[derive(Debug, Copy, Clone)]
pub struct AttribPointer
{
    index: u32,
    size: i32,
    data_type: u32,
    normalized: bool,
    stride: i32,
    offset: i32
}

impl AttribPointer
{
    #[allow(dead_code)]
    /// Default values of `false` for `normalized` and `size_of<T>()` for `stride`
    pub fn with_defaults<T>(index: u32, size: i32, data_type: u32, offset: i32) -> AttribPointer
    {
        AttribPointer
        {
            index,
            size,
            data_type,
            normalized: false,
            stride: size * std::mem::size_of::<T>() as i32,
            offset
        }
    }

    #[allow(dead_code)]
    /// No default values
    pub fn without_defaults(index: u32, size: i32, data_type: u32, normalized: bool, stride: i32, offset: i32) -> AttribPointer
    {
        AttribPointer
        {
            index, size, data_type, normalized, stride, offset
        }
    }
}

pub struct VertexArray
{
    internal: WebGlVertexArrayObject,
    context: Context,
    allocator: IndexAllocator,
    buffers: ExposedGenVec<Buffer>,
    attrib_ptrs: ExposedGenVec<Option<Vec<AttribPointer>>>
}

impl VertexArray
{
    fn new_vertex_array(context: &Context) -> Result<WebGlVertexArrayObject, GfxError>
    {
        context.create_vertex_array().ok_or_else(|| GfxError::VertexArrayCreationError(gl_get_errors(context).to_string()))
    }

    /// Creates a new vertex array
    pub fn new(context: &Context) -> Result<VertexArray, GfxError>
    {
        Ok(VertexArray
        {
            internal: VertexArray::new_vertex_array(&context)?,
            context: context.clone(),
            allocator: IndexAllocator::new(),
            buffers: ExposedGenVec::new(),
            attrib_ptrs: ExposedGenVec::new(),
        })
    }

    /// Add a `Buffer` to this `VertexArray` with the given `AttribPointer`s, if any
    pub fn add_buffer(&mut self, buffer: Buffer, attrib_ptrs: Option<Vec<AttribPointer>>) -> Index
    {
        buffer.bind();
        if let Some(attrib_ptrs) = &attrib_ptrs
        {
            self.set_attrib_ptrs(&attrib_ptrs);
        }
        let handle = self.allocator.allocate();
        self.buffers.set(handle, buffer);
        self.attrib_ptrs.set(handle, attrib_ptrs);
        handle
    }

    fn set_attrib_ptrs(&self, attrib_ptrs: &Vec<AttribPointer>)
    {
        for ptr in attrib_ptrs
        {
            self.context.vertex_attrib_pointer_with_i32(ptr.index, ptr.size, ptr.data_type, ptr.normalized, ptr.stride, ptr.offset);
            self.context.enable_vertex_attrib_array(ptr.index);
        }
    }

    #[allow(dead_code)]
    pub fn get_buffer(&self, handle: Index) -> Result<&Buffer, GfxError>
    {
        self.buffers.get(handle).ok_or_else(|| GfxError::InvalidHandle(handle))
    }

    #[allow(dead_code)]
    pub fn get_buffer_mut(&mut self, handle: Index) -> Result<&mut Buffer, GfxError>
    {
        self.buffers.get_mut(handle).ok_or_else(|| GfxError::InvalidHandle(handle))
    }

    #[allow(dead_code)]
    pub fn remove_buffer(&mut self, handle: Index) -> Result<Buffer, GfxError>
    {
        self.attrib_ptrs.remove(handle);
        self.buffers.remove(handle).ok_or_else(|| GfxError::InvalidHandle(handle))
    }
}

impl GlObject for VertexArray
{
    fn bind(&self)
    {
        self.context.bind_vertex_array(Some(&self.internal));
    }
    fn unbind(&self)
    {
        self.context.bind_vertex_array(None);
    }
    fn recreate(&mut self, context: &Context) -> Result<(), GfxError>
    {
        self.context = context.clone();
        self.internal = VertexArray::new_vertex_array(&self.context)?;
        Ok(())
    }
    fn reload(&mut self) -> Result<(), GfxError>
    {
        self.bind();

        for index in self.allocator.iter()
        {
            self.buffers.get_mut(index).ok_or_else(|| GfxError::InvalidHandle(index))?.recreate_and_reload(&self.context)?;
            if let Some(attrib_ptrs) = self.attrib_ptrs.get(index).ok_or_else(|| GfxError::InvalidHandle(index))?
            {
                self.set_attrib_ptrs(&attrib_ptrs);
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