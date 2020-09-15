/// WebGlVertexArrayObject wrapper

use crate::gfx::
{
    Context,
    GfxError,
    gl_get_errors,
    gl_object::
    {
        manager::{GlObjectHandle, GlObjectManager},
        traits::{GlObject, Bindable, Reloadable},
        buffer::Buffer,
    },
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
            attrib_ptrs: ExposedGenVec::new(),
        })
    }

    /// Registers `buffer` to this `VertexArray` with the given `AttribPointer`s, if any
    /// The target buffer MUST be bound directly before calling this function
    pub fn register_buffer(&mut self, handle: GlObjectHandle, attrib_ptrs: Option<Vec<AttribPointer>>)
    {
        if let Some(attrib_ptrs) = &attrib_ptrs
        {
            self.set_attrib_ptrs(&attrib_ptrs);
        }
        self.attrib_ptrs.set(handle, attrib_ptrs);
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
    pub fn unregister_buffer(&mut self, handle: GlObjectHandle)
    {
        self.attrib_ptrs.remove(handle);
    }
}

impl_globject!(vertex_array, VertexArray);

impl Bindable for VertexArray
{
    fn bind_internal(&self)
    {
        self.context.bind_vertex_array(Some(&self.internal));
    }
    fn unbind_internal(&self)
    {
        self.context.bind_vertex_array(None);
    }
}

impl Reloadable for VertexArray
{
    fn reload(&mut self, context: &Context, manager: &mut GlObjectManager) -> Result<(), GfxError>
    {
        self.context = context.clone();
        self.internal = VertexArray::new_vertex_array(&self.context)?;
        self.bind_internal();

        for (handle, attrib_ptrs) in &self.attrib_ptrs
        {
            manager.get(handle).ok_or_else(|| GfxError::InvalidHandle(handle))?.borrow().bind_internal();
            self.set_attrib_ptrs(&attrib_ptrs)
        }

        for index in self.allocator.iter()
        {
            self.buffers.get_mut(index).ok_or_else(|| GfxError::InvalidHandle(index))?.reload(&self.context)?;
            if let Some(attrib_ptrs) = self.attrib_ptrs.get(index).ok_or_else(|| GfxError::InvalidHandle(index))?
            {
                self.set_attrib_ptrs(&attrib_ptrs);
            }
        }

        // If this isn't unbound and a different VERTEX_ARRAY buffer gets bound,
        // then it will overwrite the VERTEX_ARRAY buffer currently attached to
        // this vertex array object
        self.unbind_internal();
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