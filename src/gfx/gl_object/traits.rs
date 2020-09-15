/// WebGl wrapper  trait
///
/// All WebGl object wrappers should implement this trait

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
    fn reload(&mut self, context: &crate::Context, manager: &mut GlObjectManager) -> Result<(), crate::gfx::GfxError>;
}

pub trait GlObject: Bindable + Reloadable + Drop
{
    fn bind(manager: &mut GlObjectManager, handle: GlObjectHandle) where Self: Sized;
    fn unbind(manager: &mut GlObjectManager, handle: GlObjectHandle) where Self: Sized;
}

macro_rules! impl_globject
{
    ($implementor_name:ident, $implementor_type:ident) =>
    {paste::paste!
    {
        impl GlObject for $implementor_type
        {
            fn bind(manager: &mut GlObjectManager, handle: GlObjectHandle) where Self: Sized
            {
                manager.[<bind_ $implementor_name>](Some(handle)).expect(concat!(stringify!($implementor_name), " bound"));
            }

            fn unbind(manager: &mut GlObjectManager, handle: GlObjectHandle) where Self: Sized
            {
                manager.[<bind_ $implementor_name>](None).expect(concat!(stringify!($implementor_name), " unbound"));
            }
        }
    }}
}