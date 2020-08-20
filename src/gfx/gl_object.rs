/// OpenGL object trait
///
/// All OpenGL object wrappers that can be bound/unbound
/// should implement this trait

pub trait GlObject: Drop
{
    fn bind(&self);
    fn unbind(&self);
    /// Recreates internal webgl program(s)
    /// The only states/data that should be reloaded here are those that are set within
    /// a `new()` function or something similar
    fn recreate(&mut self, context: &crate::Context) -> Result<(), crate::gfx::GfxError>;
    /// Reloads all webgl states and data associated with this webgl object
    fn reload(&mut self) -> Result<(), crate::gfx::GfxError>;
    /// Calls `recreate()` and then `reload()`
    fn recreate_and_reload(&mut self, context: &crate::Context) -> Result<(), crate::gfx::GfxError>
    {
        self.recreate(&context)?;
        self.reload()?;
        Ok(())
    }
}