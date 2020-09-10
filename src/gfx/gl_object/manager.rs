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
use wasm_bindgen::
{
    prelude::*,
    JsCast,
    JsValue,
};
use crate::gfx::
{
    Context,
    GfxError,
    gl_object::
    {
        traits::
        {
            GlObject,
            Bindable,
            Reloadable
        },
        wrapper::WebGlWrapper,
        buffer::Buffer,
        shader::shaderprogram::ShaderProgram,
        texture::Texture,
        vertex_array::VertexArray,
    },

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

/// Wraps a raw WebGl* JS type
pub struct WebGlWrapper
{
    internal: JsValue,
    type_: GlObjectType,
    bind_func: Box<dyn Fn(&Context, &JsValue)>,
    unbind_func: Box<dyn Fn()>
}

impl WebGlWrapper
{
    pub fn bind(&self, context: &Context)
    {
        (self.bind_func)(&context, &self.internal);
    }

    pub fn unbind(&self, context: &Context)
    {
    }

    pub fn update_internal(&mut self, internal: JsValue)
    {
        self.internal = internal;
    }
}

/// Handle to a GlObject
pub struct GlWrapperHandle
{
    // Handle to the type of a GlObject
    // Used for binding and unbinding GlObject's
    // based on type
    type_handle: Index,
    // Handle to the object itself
    wrapper_handle: Index
}
pub struct GlWrapperManager
{
    wrappers: ClosedGenVec<WebGlWrapper>,

    bound_allocator: IndexAllocator,
    bound_types: ExposedGenVec<Cell<bool>>,
    type_handles: HashMap<GlObjectType, Index, BuildHasherDefault<XxHash32>>,
}

impl GlWrapperManager
{
    pub fn new() -> GlWrapperManager
    {
        GlWrapperManager
        {
            wrappers: ClosedGenVec::new(),
            bound_allocator: IndexAllocator::new(),
            bound_types: ExposedGenVec::new(),
            type_handles: Default::default(),
        }
    }

    pub fn new_wrapper<B, U>(&mut self, internal: JsValue, type_: GlObjectType, bind_func: B, unbind_func: U) -> GlWrapperHandle
        where B: Fn(&Context, &JsValue) + 'static, U: Fn() + 'static
    {
        let wrapper = WebGlWrapper
        {
            internal,
            type_,
            bind_func: Box::new(bind_func),
            unbind_func: Box::new(unbind_func),
        };

        let type_handle = match self.type_handles.get(&type_)
        {
            Some(handle) => *handle,
            None =>
                {
                    let handle = self.bound_allocator.allocate();
                    self.type_handles.insert(type_, handle);
                    handle
                }
        };

        let wrapper_handle = self.wrappers.insert(wrapper);
        GlWrapperHandle { type_handle, wrapper_handle }
    }

    pub fn remove(&mut self, handle: GlWrapperHandle)
    {
        self.wrappers.remove(handle.wrapper_handle);
    }

    pub fn get(&self, handle: GlWrapperHandle) -> Option<&WebGlWrapper>
    {
        self.wrappers.get(handle.wrapper_handle)
    }

    pub fn get_mut(&mut self, handle: GlWrapperHandle) -> Option<&mut WebGlWrapper>
    {
        self.wrappers.get_mut(handle.wrapper_handle)
    }

    pub fn bind(&self, handle: GlWrapperHandle, context: &Context)
    {
        self.bound_types.get(handle.type_handle)?.set(true);
        self.wrappers.get(handle.wrapper_handle)?.bind(&context);
    }

    pub fn unbind(&self, handle: GlWrapperHandle, context: &Context)
    {
        self.bound_types.get(handle.type_handle)?.set(false);
        self.wrappers.get(handle.wrapper_handle)?.bind(&context);
    }

    pub fn update_internal(&mut self, handle: GlWrapperHandle, internal: &JsValue)
    {
        *self.wrappers.get_mut(handle.wrapper_handle)?.internal = internal;
    }
}