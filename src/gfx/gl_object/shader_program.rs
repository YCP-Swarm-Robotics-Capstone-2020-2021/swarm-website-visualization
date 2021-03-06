use web_sys::{WebGlProgram, WebGlShader};
use twox_hash::XxHash32;
use std::
{
    hash::BuildHasherDefault,
    collections::HashMap,
};
use crate::gfx::
{
    Context,
    GfxError,
    gl_get_errors,
    gl_object::
    {
        manager::{GlObjectManager},
        traits::{Bindable, Reloadable},
    },
};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum ShaderType
{
    VertexShader,
    FragmentShader,
    UnknownShader,
}

impl std::fmt::Display for ShaderType
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "{}", self)
    }
}
impl From<u32> for ShaderType
{
    fn from(webgl_enum: u32) -> Self
    {
        match webgl_enum
        {
            Context::VERTEX_SHADER => ShaderType::VertexShader,
            Context::FRAGMENT_SHADER => ShaderType::FragmentShader,
            _ => ShaderType::UnknownShader
        }
    }
}
impl Into<u32> for ShaderType
{
    fn into(self) -> u32
    {
        match self
        {
            ShaderType::VertexShader => Context::VERTEX_SHADER,
            ShaderType::FragmentShader => Context::FRAGMENT_SHADER,
            ShaderType::UnknownShader => 0
        }
    }
}

pub struct ShaderProgram
{
    internal: WebGlProgram,
    context: Context,

    vert_src: Option<String>,
    frag_src: Option<String>,
    // Indexed by block binding, holds block names
    block_bindings: Vec<Option<String>>,
    uniforms_i32: HashMap<String, Vec<i32>, BuildHasherDefault<XxHash32>>
}

impl ShaderProgram
{
    fn new_program(context: &Context) -> Result<WebGlProgram, GfxError>
    {
        context.create_program().ok_or_else(|| GfxError::ShaderProgramCreationError(gl_get_errors(&context).to_string()))
    }

    pub fn new(context: &Context, vert_src: Option<String>, frag_src: Option<String>) -> Result<ShaderProgram, GfxError>
    {
        if vert_src.is_none() && frag_src.is_none()
        {
            return Err(GfxError::NoShaderSource("At least one shader source must be present".to_string()))
        }
        let program = ShaderProgram
        {
            internal: ShaderProgram::new_program(&context)?,
            context: context.clone(),
            vert_src: vert_src,
            frag_src: frag_src,
            block_bindings: vec![],
            uniforms_i32: Default::default(),
        };
        program.compile()?;
        Ok(program)
    }

    /// Compiles the shader fragments and attaches them to the shader program
    fn compile(&self) -> Result<(), GfxError>
    {

        let vert = if let Some(src) = &self.vert_src
        {
            Some(self.compile_shader(src.as_str(), ShaderType::VertexShader)?)
        } else { None };

        let frag = if let Some(src) = &self.frag_src
        {
            Some(self.compile_shader(src.as_str(), ShaderType::FragmentShader)?)
        } else { None };

        self.context.link_program(&self.internal);

        if self.context.get_program_parameter(&self.internal, Context::LINK_STATUS).as_bool().unwrap_or(false)
        {
            self.context.delete_shader(vert.as_ref());
            self.context.delete_shader(frag.as_ref());
            Ok(())
        }
        else
        {
            let info_log = self.context.get_program_info_log(&self.internal)
                .unwrap_or_else(|| format!("Error getting shader program info logs after linking. GlErrors: {}", gl_get_errors(&self.context)).to_string());
            Err(GfxError::ShaderProgramLinkingError(info_log))
        }
    }

    /// Compiles a shader fragment
    fn compile_shader(&self, src: &str, shader_type: ShaderType) -> Result<WebGlShader, GfxError>
    {
        let shader = self.context.create_shader(shader_type.into())
            .ok_or_else(|| GfxError::ShaderCreationError(shader_type, gl_get_errors(&self.context).to_string()))?;
        self.context.shader_source(&shader, &src);
        self.context.compile_shader(&shader);
        self.context.attach_shader(&self.internal, &shader);

        if self.context.get_shader_parameter(&shader, Context::COMPILE_STATUS).as_bool().unwrap_or(false)
        {
            Ok(shader)
        }
        else
        {
            let info_log = self.context.get_shader_info_log(&shader)
                .unwrap_or_else(|| format!("Error getting shader compilation info log. GlErrors: {}", gl_get_errors(&self.context)).to_string());
            Err(GfxError::ShaderCompilationError(shader_type, info_log))
        }
    }

    /// Binds the uniform block `block_name` to the given `block_binding`
    pub fn add_uniform_block_binding(&mut self, block_name: &str, block_binding: u32) -> Result<(), GfxError>
    {
        let index = self.context.get_uniform_block_index(&self.internal, block_name);
        if index == Context::INVALID_INDEX
        {
            Err(GfxError::InvalidUniformBlockName(String::from(block_name)))
        }
        else
        {
            self.context.uniform_block_binding(&self.internal, index, block_binding);

            if self.block_bindings.len() <= block_binding as usize
            {
                self.block_bindings.resize_with(block_binding as usize + 1, || None);
            }
            self.block_bindings[block_binding as usize] = Some(String::from(block_name));
            Ok(())
        }
    }

    // TODO: Add more set_uniform functions as necessary

    /// Set the uniform with `name` to `value`
    /// If `name` is a scalar, then give a one element slice. i.e. `&[5]`
    pub fn set_uniform_i32(&mut self, name: &str, value: &[i32]) -> Result<(), GfxError>
    {
        let location = self.context.get_uniform_location(&self.internal, name).ok_or_else(|| GfxError::InvalidUniformName(name.to_string()))?;
        self.context.uniform1iv_with_i32_array(Some(&location), value);
        self.uniforms_i32.insert(name.to_string(), value.to_vec());
        Ok(())
    }

}

impl_globject!(ShaderProgram);

impl Bindable for ShaderProgram
{
    fn bind_internal(&self) { self.context.use_program(Some(&self.internal)); }
    fn unbind_internal(&self) { self.context.use_program(None); }
}

impl Reloadable for ShaderProgram
{
    fn reload(&mut self, context: &Context, _manager: &GlObjectManager) -> Result<(), GfxError>
    {
        self.context = context.clone();
        self.internal = ShaderProgram::new_program(&self.context)?;
        self.compile()?;
        self.bind_internal();

        // Restore all block bindings
        for (block_binding, block_name) in self.block_bindings.to_owned().iter().enumerate()
        {
            if let Some(block_name) = block_name
            {
                self.add_uniform_block_binding(block_name.as_str(), block_binding as u32)?;
            }
        }
        // Restore i32 uniforms
        for (name, value) in self.uniforms_i32.to_owned()
        {
            self.set_uniform_i32(&name, &value)?;
        }

        Ok(())
    }
}

impl Drop for ShaderProgram
{
    fn drop(&mut self)
    {
        self.context.delete_program(Some(&self.internal));
    }
}

#[cfg(test)]
mod tests
{
    inject_wasm_test_boilerplate!();

    use crate::gfx::
    {
        gl_object::
        {
            shader_program::ShaderProgram,
        },
    };
    use crate::gfx::gl_object::traits::Bindable;

    // Embed shaders into test executable so that we can test ShaderProgram alone instead of
    //      going through ResourceLoader. Since this is in a cfg(test) module, it won't be
    //      included in the normal builds
    macro_rules! shader_source
    {
        ($path:expr) =>
        {
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), concat!("/", $path)))
        };
    }
    pub const BASIC_VERT: &'static str = shader_source!("/resources/shaders/basic_vert.glsl");
    pub const BASIC_FRAG: &'static str = shader_source!("/resources/shaders/basic_frag.glsl");

    fn get_shader_program() -> (Context, ShaderProgram)
    {
        let context = get_context();
        let shader_program = ShaderProgram::new(
            &context,
            Some(BASIC_VERT.to_string()),
            Some(BASIC_FRAG.to_string())
        ).expect("shader program");
        shader_program.bind_internal();
        (context, shader_program)
    }

    #[wasm_bindgen_test]
    fn test_creation()
    {
        let (context, _shader_program) = get_shader_program();
        assert_eq!(GfxError::GlErrors(vec![GlError::NoError]), gl_get_errors(&context));
    }

    // TODO: Custom test shader so that these tests can pass
    //       Or would using existing shaders be could so the
    //       shaders themselves can be tested too?
    #[wasm_bindgen_test]
    fn test_block_bindings()
    {
        //let (context, mut shader_program) = get_shader_program();

        // TODO:
        //shader_program.add_uniform_block_binding("fake_block", 0).unwrap();
        //assert_eq!(shader_program.block_bindings[0], Some(String::from("fake_block")))
    }

    #[wasm_bindgen_test]
    fn test_set_uniform()
    {
        //let (context, mut shader_program) = get_shader_program();

        // TODO:
        //shader_program.set_uniform_i32("fake_uniform", &[0]).unwrap();
        //assert_eq!(GfxError::GlErrors(vec![GlError::NoError]), gl_get_errors(&context));
    }
}