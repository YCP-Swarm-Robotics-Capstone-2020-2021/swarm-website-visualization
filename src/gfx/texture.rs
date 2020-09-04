use web_sys::WebGlTexture;
use crate::gfx::{Context, GfxError, gl_object::GlObject, gl_get_errors};

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
    fn new_texture(context: &Context, target: u32) -> Result<WebGlTexture, GfxError>
    {
        context.create_texture().ok_or_else(|| GfxError::TextureCreationError(gl_get_errors(&context).to_string()))
    }

    pub fn new(context: &Context, params: TextureParams) -> Result<Texture, GfxError>
    {
        let texture = Texture
        {
            internal: Texture::new_texture(&context, params.target)?,
            context: context.clone(),
            params
        };

        context.bind_texture(texture.params.target, Some(&texture.internal));
        context.tex_parameteri(texture.params.target, Context::TEXTURE_WRAP_S, texture.params.wrap_type as i32);
        context.tex_parameteri(texture.params.target, Context::TEXTURE_WRAP_T, texture.params.wrap_type as i32);
        context.tex_parameteri(texture.params.target, Context::TEXTURE_MIN_FILTER, texture.params.filter_type as i32);
        context.tex_parameteri(texture.params.target, Context::TEXTURE_MAG_FILTER, texture.params.filter_type as i32);
        context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            texture.params.target,
            0,
            texture.params.format as i32,
            texture.params.size.0,
            texture.params.size.1,
            0,
            texture.params.format,
            Context::UNSIGNED_BYTE,
            Some(texture.params.data.as_slice())
        ).or_else(|_| Err(GfxError::TextureCreationError(gl_get_errors(&context).to_string())))?;

        Ok(texture)
    }
}

impl GlObject for Texture
{
    fn bind(&self) { self.context.bind_texture(self.params.target, Some(&self.internal)); }

    fn unbind(&self) { self.context.bind_texture(self.params.target, None); }

    fn recreate(&mut self, context: &Context) -> Result<(), GfxError>
    {
        *self = Texture::new(&context, self.params.clone())?;
        Ok(())
    }

    fn reload(&mut self) -> Result<(), GfxError>
    {
        Ok(())
    }
}

impl Drop for Texture
{
    fn drop(&mut self)
    {
        self.context.delete_texture(Some(&self.internal));
    }
}