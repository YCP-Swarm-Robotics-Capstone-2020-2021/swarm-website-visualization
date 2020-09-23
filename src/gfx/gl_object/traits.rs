/// GlObject and its associated traits

use crate::gfx::gl_object::manager::{GlObjectHandle, GlObjectManager};

pub trait Bindable
{
    fn bind_internal(&self);
    fn unbind_internal(&self);
}

pub trait Reloadable
{
    /// Recreates internal webgl program(s) and
    /// reloads all webgl states and data associated with this webgl object
    fn reload(&mut self, context: &crate::Context, manager: &GlObjectManager) -> Result<(), crate::gfx::GfxError>;
}

pub trait GlObject: Bindable + Reloadable + Drop
{
    fn bind(manager: &mut GlObjectManager, handle: GlObjectHandle) where Self: Sized;
    fn unbind(manager: &mut GlObjectManager, handle: GlObjectHandle) where Self: Sized;
}

macro_rules! impl_globject
{
    ($implementor:ident) =>
    {paste::paste!
    {
        impl crate::gfx::gl_object::traits::GlObject for $implementor
        {
            fn bind(manager: &mut crate::gfx::gl_object::manager::GlObjectManager, handle: crate::gfx::gl_object::manager::GlObjectHandle) where Self: Sized
            {
                manager.[<bind_ $implementor:snake>](handle, true).expect(concat!(stringify!($implementor), " bound"));
            }

            fn unbind(manager: &mut crate::gfx::gl_object::manager::GlObjectManager, handle: crate::gfx::gl_object::manager::GlObjectHandle) where Self: Sized
            {
                manager.[<bind_ $implementor:snake>](handle, false).expect(concat!(stringify!($implementor), " unbound"));
            }
        }
    }}
}