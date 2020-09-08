/// WebGl wrapper  trait
///
/// All WebGl object wrappers should implement this trait

pub trait GlObject: Drop + downcast_rs::Downcast
{
    fn bind(&self);
    fn unbind(&self);
    /// Recreates internal webgl program(s) and
    /// reloads all webgl states and data associated with this webgl object
    fn reload(&mut self, context: &crate::Context) -> Result<(), crate::gfx::GfxError>;
}
impl_downcast!(GlObject);