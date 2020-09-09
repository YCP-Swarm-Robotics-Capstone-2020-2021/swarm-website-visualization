use gen_vec::{Index, exposed::{IndexAllocator, ExposedGenVec}};
use std::
{
    collections::HashMap,
    hash::BuildHasherDefault,
    any::TypeId,
    cell::RefCell,
};
use twox_hash::XxHash32;

use crate::gfx::
{
    Context,
    GfxError,
    gl_object::traits::{GlObject, Bindable, Reloadable},
    buffer::Buffer,
    shader::shaderprogram::ShaderProgram,
    texture::Texture,
    vertex_array::VertexArray,
};

pub type GlObjectHandle = Index;
pub struct GlObjectManager
{
    allocator: IndexAllocator,
    objects: ExposedGenVec<RefCell<dyn GlObject>>,
    bound: ExposedGenVec<Option<GlObjectHandle>>,
}

impl GlObjectManager
{
    pub fn new() -> GlObjectManager
    {
        GlObjectManager
        {
            allocator: ExposedGenVec::new(),
            objects: ExposedGenVec::new(),
            bound: ExposedGenVec::new(),
        }
    }

    pub fn insert<T>(&mut self, obj: T) -> GlObjectHandle where T: GlObject + 'static
    {
        self.objects.insert(Box::new(obj))
    }

    pub fn remove<T>(&mut self, handle: GlObjectHandle) where T: GlObject + 'static
    {
        self.objects.remove(handle);
    }

    pub(in crate::gfx) fn get<T>(&self, handle: GlObjectHandle) -> Option<&T> where T: GlObject + 'static
    {
        match self.objects.get(handle)
        {
            Some(obj) => obj.downcast_ref::<T>(),
            _ => None
        }
    }

    pub(in crate::gfx) fn get_mut<T>(&mut self, handle: GlObjectHandle) -> Option<&mut T> where T: GlObject + 'static
    {
        match self.objects.get_mut(handle)
        {
            Some(obj) => obj.downcast_mut::<T>(),
            _ => None
        }
    }

/*    pub(in crate::gfx) fn bind(&mut self, handle: GlObjectHandle)
    {
        if let Some(obj) = self.objects.get(handle)
        {
            obj.bind();
            self.bound.insert(TypeId::of::<T>(), handle);
        }
    }

    pub(in crate::gfx) fn unbind(&mut self, handle: GlObjectHandle)
    {
        if let Some(obj) = self.objects.get(handle)
        {
            obj.unbind();
            self.bound.insert(TypeId::of::<T>(), None);
        }
    }

    pub fn reload_objects(&mut self, context: &Context) -> Result<(), GfxError>
    {
        for (_, obj) in &mut self.objects
        {
            obj.reload(&context);
        }

        for (_, handle) in &self.bound
        {
            if let Some(handle) = *handle
            {
                if let Some(obj) = self.objects.get(handle)
                {
                    obj.bind();
                }
            }
        }
        Ok(())
    }*/
}