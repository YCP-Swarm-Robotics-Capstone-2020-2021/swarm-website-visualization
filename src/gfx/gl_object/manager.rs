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

/// Defines and implements a new struct instance manager.
///
/// `manager_name` is the name of the manager struct to be created and implemented
/// `handle_name` is the name of the type alias to `gen_vec::Index` specific to this manager
/// `module_path`, `managed_struct` should look something like
///     `$module_path::texture, Texture`
macro_rules! define_manager
{
    ($manager_name:ident, $handle_name:ident; $($module_path:path => $managed_struct:ident),+) =>
    {paste::paste!
    {
        pub type [<$handle_name:camel>] = Index;
        pub struct $manager_name
        {
            $(
            [<$managed_struct:snake s>]: ClosedGenVec<RefCell<$module_path::$managed_struct>>,
            [<bound_ $managed_struct:snake>]: Option<[<$handle_name:camel>]>
            ),+
        }

        impl $manager_name
        {
            pub fn new() -> $manager_name
            {
                $manager_name
                {
                    $(
                    [<$managed_struct:snake s>]: ClosedGenVec::new(),
                    [<bound_ $managed_struct:snake>]: None
                    ),+
                }
            }

            $(
            pub fn [<insert_ $managed_struct:snake>](&mut self, [<$managed_struct:snake>]: $module_path::$managed_struct) -> [<$handle_name:camel>]
            {
                self.[<$managed_struct:snake s>].insert(RefCell::new([<$managed_struct:snake>]))
            }
            pub(in crate::gfx::gl_object) fn [<get_ $managed_struct:snake>](&self, handle: [<$handle_name:camel>]) -> Option<Ref<$module_path::$managed_struct>>
            {
                Some(self.[<$managed_struct:snake s>].get(handle)?.borrow())
            }
            pub(in crate::gfx::gl_object) fn [<get_mut_ $managed_struct:snake>](&self, handle: [<$handle_name:camel>]) -> Option<RefMut<$module_path::$managed_struct>>
            {
                Some(self.[<$managed_struct:snake s>].get(handle)?.borrow_mut())
            }
            pub(in crate::gfx::gl_object) fn [<remove_ $managed_struct:snake>](&mut self, handle: [<$handle_name:camel>])
            {
                self.[<$managed_struct:snake s>].remove(handle);
            }
            pub(in crate::gfx::gl_object) fn [<bind_ $managed_struct:snake>](&mut self, handle: Option<[<$handle_name:camel>]>) -> Result<(), GfxError>
            {
                if let Some(handle) = handle
                {
                    if let Some(obj) = self.[<$managed_struct:snake s>].get(handle)
                    {
                        self.[<bound_ $managed_struct:snake>] = Some(handle);
                        obj.borrow().bind_internal();
                    }
                    else
                    {
                        return Err(GfxError::InvalidHandle(handle))
                    }
                }
                else
                {
                    self.[<bound_ $managed_struct:snake>] = handle;
                }

                Ok(())
            }
            )+

            pub fn reload_objects(&self, context: &Context)
            {
                $(
                for (_, obj) in &self.[<$managed_struct:snake s>] { obj.borrow_mut().reload(&context, &self); }
                )+

                $(
                if let Some(handle) = self.[<bound_ $managed_struct:snake>]
                {
                    if let Some(obj) = self.[<$managed_struct:snake s>].get(handle)
                    {
                        obj.borrow().bind_internal();
                    }
                }
                )+
            }
        }
    }};
}
define_manager!(GlObjectManager, GlObjectHandle;
    crate::gfx::gl_object => ArrayBuffer,
    crate::gfx::gl_object => ElementArrayBuffer,
    crate::gfx::gl_object::shader_program => ShaderProgram,
    crate::gfx::gl_object::texture => Texture,
    crate::gfx::gl_object::uniform_buffer => UniformBuffer,
    crate::gfx::gl_object::vertex_array => VertexArray
);