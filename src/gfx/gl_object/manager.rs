use gen_vec::{Index, exposed::{IndexAllocator, ExposedGenVec}, closed::ClosedGenVec};
use std::
{
    collections::HashMap,
    hash::BuildHasherDefault,
    any::TypeId,
    cell::
    {
        Cell,
        RefCell,
        Ref,
        RefMut,
    },
};
use twox_hash::XxHash32;

use crate::gfx::
{
    Context,
    GfxError,
    gl_object::traits::{GlObject, Bindable, Reloadable},
};

/// Different GlObjects currently available
/// This is to be able to differentiate between
/// the different types of Buffers and Textures
/// that exist
#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum GlObjectType
{
    Buffer(u32),
    ShaderProgram,
    Texture(u32),
    UniformBuffer,
    VertexArray,
}

/// Handle to a GlObject
pub struct GlObjectHandle
{
    // Handle to the type of a GlObject
    // Used for binding and unbinding GlObject's
    // based on type
    type_handle: Index,
    // Handle to the object itself
    object_handle: Index
}
pub struct GlObjectManager
{
    objects: ClosedGenVec<RefCell<Box<dyn GlObject>>>,

    bound_allocator: IndexAllocator,
    bound_types: ExposedGenVec<Cell<bool>>,
    type_handles: HashMap<GlObjectType, Index, BuildHasherDefault<XxHash32>>,
}

impl GlObjectManager
{
    pub fn new() -> GlObjectManager
    {
        GlObjectManager
        {
            objects: ClosedGenVec::new(),
            bound_allocator: IndexAllocator::new(),
            bound_types: ExposedGenVec::new(),
            type_handles: Default::default(),
        }
    }

    pub fn insert<T>(&mut self, obj: T, type_: GlObjectType) -> GlObjectHandle where T: GlObject + 'static
    {

        let type_handle = match self.type_handles.get(&type_)
        {
            Some(handle) => *handle,
            None =>
                {
                    let handle = self.bound_allocator.allocate();
                    self.type_handles.insert(type_, handle);
                    self.bound_types.set(handle, Cell::new(false));
                    handle
                }
        };
        
        let object_handle = self.objects.insert(RefCell::new(Box::new(obj)));

        GlObjectHandle { type_handle, object_handle }
    }

    pub fn remove<T>(&mut self, handle: GlObjectHandle) where T: GlObject + 'static
    {
        self.objects.remove(handle.object_handle);
    }

    pub(in crate::gfx) fn get<T>(&self, handle: GlObjectHandle) -> Option<Ref<T>> where T: GlObject + 'static
    {
        match self.objects.get(handle.object_handle)
        {
            Some(obj) => Some(Ref::map(obj.borrow(), |obj| obj.downcast_ref::<T>().expect("GlObject downcast"))),
            _ => None
        }
    }

    pub(in crate::gfx) fn get_mut<T>(&mut self, handle: GlObjectHandle) -> Option<RefMut<T>> where T: GlObject + 'static
    {
        match self.objects.get_mut(handle.object_handle)
        {
            Some(obj) => Some(RefMut::map(obj.borrow_mut(), |obj| obj.downcast_mut::<T>().expect("GlObject downcast"))),
            _ => None
        }
    }

    pub(in crate::gfx) fn bind(&self, handle: GlObjectHandle)
    {
        if let Some(obj) = self.objects.get(handle.object_handle)
        {
            obj.borrow().bind_internal();
            if let Some(bound) = self.bound_types.get(handle.type_handle)
            {
                bound.set(true);
            }
        }
    }

    pub(in crate::gfx) fn unbind(&self, handle: GlObjectHandle)
    {
        if let Some(obj) = self.objects.get(handle.object_handle)
        {
            obj.borrow().unbind_internal();
            if let Some(bound) = self.bound_types.get(handle.type_handle)
            {
                bound.set(false);
            }
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