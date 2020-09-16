#[macro_use]
pub mod traits;
pub mod manager;

//pub mod buffer_old;
#[macro_use]
pub mod buffer;
use traits::{GlObject, Bindable, Reloadable};
use buffer::Buffer;
impl_buffer!(ARRAY_BUFFER, ArrayBuffer);
impl_buffer!(ELEMENT_ARRAY_BUFFER, ElementArrayBuffer);

pub mod shader_program;
pub mod uniform_buffer;
pub mod texture;
pub mod vertex_array;