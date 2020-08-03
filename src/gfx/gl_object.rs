/// OpenGL object trait
///
/// All OpenGL object wrappers that can be bound/unbound
/// should implement this trait

pub trait GLObject: Drop
{
    fn bind(&self);
    fn unbind(&self);
}