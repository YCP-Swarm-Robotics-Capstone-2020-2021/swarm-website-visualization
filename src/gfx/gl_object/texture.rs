use web_sys::WebGlTexture;
use crate::gfx::
{
    Context,
    GfxError,
    gl_get_errors,
    gl_object::
    {
        manager::{GlObjectManager},
        traits::{Bindable, Reloadable}
    },
};

#[derive(Debug, Clone)]
pub struct TextureParams
{
    // Target texture, i.e. GL_TEXTURE_2D
    pub target: u32,
    // Texture format, i.e. GL_BGR, GL_BGRA, etc
    pub format: u32,
    // Width & height of texture
    pub size: (i32, i32),
    // One of the wrap type constants
    pub wrap_type: u32,
    // One of the filter type constants
    pub filter_type: u32,
    // Image data
    pub data: Vec<u8>,
}
pub struct Texture
{
    internal: WebGlTexture,
    context: Context,
    params: TextureParams
}

impl Texture
{
    fn new_texture(context: &Context) -> Result<WebGlTexture, GfxError>
    {
        context.create_texture().ok_or_else(|| GfxError::TextureCreationError(gl_get_errors(&context).to_string()))
    }

    pub fn new(context: &Context, params: TextureParams) -> Result<Texture, GfxError>
    {
        let texture = Texture
        {
            internal: Texture::new_texture(&context)?,
            context: context.clone(),
            params
        };

        texture.fill_texture()?;

        Ok(texture)
    }

    fn fill_texture(&self) -> Result<(), GfxError>
    {
        self.context.bind_texture(self.params.target, Some(&self.internal));
        self.context.tex_parameteri(self.params.target, Context::TEXTURE_WRAP_S, self.params.wrap_type as i32);
        self.context.tex_parameteri(self.params.target, Context::TEXTURE_WRAP_T, self.params.wrap_type as i32);
        self.context.tex_parameteri(self.params.target, Context::TEXTURE_MIN_FILTER, self.params.filter_type as i32);
        self.context.tex_parameteri(self.params.target, Context::TEXTURE_MAG_FILTER, self.params.filter_type as i32);
        self.context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            self.params.target,
            0,
            self.params.format as i32,
            self.params.size.0,
            self.params.size.1,
            0,
            self.params.format,
            Context::UNSIGNED_BYTE,
            Some(self.params.data.as_slice())
        ).or_else(|_| Err(GfxError::TextureCreationError(gl_get_errors(&self.context).to_string())))
    }
}

impl_globject!(Texture);

impl Bindable for Texture
{
    fn bind_internal(&self) { self.context.bind_texture(self.params.target, Some(&self.internal)); }

    fn unbind_internal(&self) { self.context.bind_texture(self.params.target, None); }
}

impl Reloadable for Texture
{
    fn reload(&mut self, context: &Context, _manager: &GlObjectManager) -> Result<(), GfxError>
    {
        self.context = context.clone();
        self.internal = Texture::new_texture(&context)?;
        self.fill_texture()
    }
}

impl Drop for Texture
{
    fn drop(&mut self)
    {
        self.context.delete_texture(Some(&self.internal));
    }
}