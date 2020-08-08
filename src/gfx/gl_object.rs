/// OpenGL object trait
///
/// All OpenGL object wrappers that can be bound/unbound
/// should implement this trait

pub trait GLObject: Drop
{
    fn bind(&self);
    fn unbind(&self);
    /// Called if a WebGL context is lost,
    /// must re-initialize itself and reload all relevant states and data
    type ReloadError;
    fn reload(&mut self, context: &std::rc::Rc<crate::Context>) -> Result<(), Self::ReloadError>;
}