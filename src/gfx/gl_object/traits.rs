/// WebGl wrapper  trait
///
/// All WebGl object wrappers should implement this trait

use crate::gfx::gl_object::manager::{GlWrapperHandle, GlWrapperManager};

pub trait GlObject: Drop
{
    fn bind(&self, manager: &GlWrapperManager);
    fn unbind(&self, manager: &GlWrapperManager);
    fn reload(&mut self, context: &crate::Context) -> Result<(), crate::gfx::GfxError>;
}