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

/*
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
}*/

macro_rules! define_manager
{
    // TODO: Some of these should probably be tt instead of ident
    ($manager_name:ident, $handle_name:ident => $($managee_name:ident: $managee_type:ident),+) =>
    {paste::paste!
    {
        pub type $handle_name = Index;
        pub struct $manager_name
        {
            $(
            [<$managee_name s>]: ClosedGenVec<RefCell<crate::gfx::gl_object::$managee_name::$managee_type>>,
            [<bound_ $managee_name>]: Option<Index>
            ),+
        }

        impl $manager_name
        {
            pub fn new() -> $manager_name
            {
                $manager_name
                {
                    $(
                    [<$managee_name s>]: ClosedGenVec::new(),
                    [<bound_ $managee_name>]: None
                    ),+
                }
            }

            $(
            pub fn [<insert_ $managee_name>](&mut self, $managee_name: crate::gfx::gl_object::$managee_name::$managee_type) -> Index
            {
                self.[<$managee_name s>].insert(RefCell::new($managee_name))
            }
            pub(in crate::gfx::gl_object) fn [<get_ $managee_name>](&self, handle: Index) -> Option<Ref<crate::gfx::gl_object::$managee_name::$managee_type>>
            {
                Some(self.[<$managee_name s>].get(handle)?.borrow())
            }
            pub(in crate::gfx::gl_object) fn [<get_mut_ $managee_name>](&self, handle: Index) -> Option<RefMut<crate::gfx::gl_object::$managee_name::$managee_type>>
            {
                Some(self.[<$managee_name s>].get(handle)?.borrow_mut())
            }
            pub(in crate::gfx::gl_object) fn [<remove_ $managee_name>](&mut self, handle: Index)
            {
                self.[<$managee_name s>].remove(handle);
            }
            pub(in crate::gfx::gl_object) fn [<bind_ $managee_name>](&mut self, handle: Option<Index>) -> Result<(), GfxError>
            {
                if let Some(handle) = handle
                {
                    if let Some(obj) = self.[<$managee_name s>].get(handle)
                    {
                        self.[<bound_ $managee_name>] = Some(handle);
                        obj.bind();
                    }
                    else
                    {
                        return Err(GfxError::InvalidHandle(handle))
                    }
                }
                else
                {
                    self.[<bound_ $managee_name>] = handle;
                }

                Ok(())
            }
            )+

            pub fn reload_objects(&mut self, context: &Context)
            {
                $(
                for (_, obj) in &mut self.[<$managee_name s>] { obj.borrow_mut().reload(&context); }
                )+

                $(
                if let Some(handle) = self.[<bound_ $managee_name>]
                {
                    if let Some(obj) = self.[<$managee_name s>].get(handle)
                    {
                        obj.borrow().bind_internal();
                    }
                }
                )+
            }
        }
    }}
}
define_manager!(GlObjectManager, GlObjectHandle => buffer: Buffer, shader_program: ShaderProgram, texture: Texture, uniform_buffer: UniformBuffer, vertex_array: VertexArray);