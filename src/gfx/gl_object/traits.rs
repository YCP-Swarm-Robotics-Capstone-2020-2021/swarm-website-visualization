/// WebGl wrapper  trait
///
/// All WebGl object wrappers should implement this trait

use crate::gfx::gl_object::manager::{Index, GlObjectManager};

pub trait Bindable
{
    fn bind(&self);
    fn unbind(&self);
}

pub trait Reloadable
{
    /// Recreates internal webgl program(s) and
    /// reloads all webgl states and data associated with this webgl object
    fn reload(&mut self, context: &crate::Context) -> Result<(), crate::gfx::GfxError>;
}

pub trait GlObject: Bindable + Reloadable + Drop + downcast_rs::Downcast
{
    fn bind(manager: &mut GlObjectManager, handle: Index) where Self: Sized { manager.bind(handle); }
    fn unbind(manager: &mut GlObjectManager, handle: Index) where Self: Sized { manager.unbind(handle); }
}
impl_downcast!(GlObject);