use gen_vec::{Index, closed::ClosedGenVec};
use std::
{
    collections::HashMap,
    hash::BuildHasherDefault,
    any::
    {
        Any,
        TypeId,
    }
};
use twox_hash::XxHash32;

use crate::gfx::
{
    Context,
    GfxError,
    gl_object::GlObject,
};

pub type GlObjectHandle = Index;
pub struct GlObjectManager
{
    objects: ClosedGenVec<Box<dyn GlObject>>,
    bound: HashMap<TypeId, Option<GlObjectHandle>, BuildHasherDefault<XxHash32>>,
}

impl GlObjectManager
{
    pub fn new() -> GlObjectManager
    {
        GlObjectManager
        {
            objects: ClosedGenVec::new(),
            bound: Default::default()
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

    pub(in crate::gfx) fn bind<T>(&mut self, handle: Option<GlObjectHandle>) where T: GlObject + 'static
    {
        self.bound.insert(TypeId::of::<T>(),handle);
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
    }
}