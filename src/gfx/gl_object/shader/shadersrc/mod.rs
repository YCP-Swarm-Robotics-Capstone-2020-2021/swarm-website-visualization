macro_rules! shader_source
{
    ($path:expr) =>
    {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), concat!("/", $path)))
    }
}

pub const BASIC_VERT: &'static str = shader_source!("shaders/basic_vert.glsl");
pub const BASIC_FRAG: &'static str = shader_source!("shaders/basic_frag.glsl");

pub const TEXTURE_VERT: &'static str = shader_source!("shaders/texture_vert.glsl");
pub const TEXTURE_FRAG: &'static str = shader_source!("shaders/texture_frag.glsl");