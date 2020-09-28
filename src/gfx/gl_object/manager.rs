use gen_vec::{Index, closed::ClosedGenVec};
use std::
{
    cell::
    {
        Cell,
        RefCell,
        Ref,
        RefMut,
    },
};

use crate::gfx::
{
    Context,
    GfxError,
    gl_object::traits::{Bindable, Reloadable},
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
            [<bound_ $managed_struct:snake>]: Cell<Option<[<$handle_name:camel>]>>
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
                    [<bound_ $managed_struct:snake>]: Cell::new(None)
                    ),+
                }
            }

            $(
            /// Insert (and move) the given struct into the manager's ownership
            /// Returns a handle to the object
            #[allow(dead_code)]
            pub fn [<insert_ $managed_struct:snake>](&mut self, [<$managed_struct:snake>]: $module_path::$managed_struct) -> [<$handle_name:camel>]
            {
                self.[<$managed_struct:snake s>].insert(RefCell::new([<$managed_struct:snake>]))
            }
            /// Get an immutable reference to the struct associated with `handle` if `handle` is valid
            #[allow(dead_code)]
            pub fn [<get_ $managed_struct:snake>](&self, handle: [<$handle_name:camel>]) -> Option<Ref<$module_path::$managed_struct>>
            {
                Some(self.[<$managed_struct:snake s>].get(handle)?.borrow())
            }
            /// Get a mutable reference to the struct associated with `handle` if `handle` is valid
            #[allow(dead_code)]
            pub fn [<get_mut_ $managed_struct:snake>](&self, handle: [<$handle_name:camel>]) -> Option<RefMut<$module_path::$managed_struct>>
            {
                Some(self.[<$managed_struct:snake s>].get(handle)?.borrow_mut())
            }
            /// Remove the struct associated with `handle` from the manager's ownership and drop its memory
            #[allow(dead_code)]
            pub fn [<remove_ $managed_struct:snake>](&mut self, handle: [<$handle_name:camel>])
            {
                self.[<$managed_struct:snake s>].remove(handle);
            }
            /// Bind the struct associated with `handle`
            /// `bound` is whether `handle` should be bound or unbound after this function call
            #[allow(dead_code)]
            pub fn [<bind_ $managed_struct:snake>](&self, handle: [<$handle_name:camel>], bound: bool) -> Result<(), GfxError>
            {
                if let Some(obj) = self.[<$managed_struct:snake s>].get(handle)
                {
                    let bound_struct = self.[<bound_ $managed_struct:snake>].get();
                    if bound && bound_struct != Some(handle)
                    {
                        self.[<bound_ $managed_struct:snake>].set(Some(handle));
                        obj.borrow().bind_internal();
                    }
                    else if !bound && bound_struct != None
                    {
                        self.[<bound_ $managed_struct:snake>].set(None);
                        obj.borrow().unbind_internal();
                    }
                }
                else
                {
                    return Err(GfxError::InvalidHandle(handle))
                }

                Ok(())
            }
            )+
            /// Reloads the state of all owned structs and re-binds the previously bound structs
            #[allow(dead_code)]
            pub fn reload_objects(&self, context: &Context)
            {
                $(
                for (_, obj) in &self.[<$managed_struct:snake s>] { obj.borrow_mut().reload(&context, &self).expect(concat!(stringify!([<$managed_struct:snake s>]), " reloaded")); }
                )+

                $(
                if let Some(handle) = self.[<bound_ $managed_struct:snake>].get()
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
    crate::gfx::gl_object::texture => Texture2d,
    crate::gfx::gl_object::uniform_buffer => UniformBuffer,
    crate::gfx::gl_object::vertex_array => VertexArray
);