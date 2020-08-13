use std::rc::Rc;
use web_sys::{WebGlProgram, WebGlShader};
use crate::gfx::
{
    Context,
    GfxError,
    gl_get_errors,
    gl_object::GlObject,
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
    context: Rc<Context>,

    vert_src: Option<String>,
    frag_src: Option<String>,
    // Indexed by block binding, holds block names
    block_bindings: Vec<Option<String>>,
}

impl ShaderProgram
{
    fn new_program(context: &Context) -> Result<WebGlProgram, GfxError>
    {
        context.create_program().ok_or_else(|| GfxError::ShaderProgramCreationError(gl_get_errors(&context).to_string()))
    }

    pub fn new(context: &Rc<Context>, vert_src: Option<String>, frag_src: Option<String>) -> Result<ShaderProgram, GfxError>
    {
        if vert_src.is_none() && frag_src.is_none()
        {
            return Err(GfxError::NoShaderSource("At least one shader source must be present".to_string()))
        }
        let program = ShaderProgram
        {
            internal: ShaderProgram::new_program(&context)?,
            context: Rc::clone(&context),
            vert_src: vert_src,
            frag_src: frag_src,
            block_bindings: vec![]
        };
        program.compile()?;
        Ok(program)
    }

    /// Compiles the shader fragments and attaches them to the shader program
    fn compile(&self) -> Result<(), GfxError>
    {

        let vert = if let Some(src) = &self.vert_src
        {
            Some(self.compile_shader(src, ShaderType::VertexShader)?)
        } else { None };

        let frag = if let Some(src) = &self.frag_src
        {
            Some(self.compile_shader(src, ShaderType::FragmentShader)?)
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
    fn compile_shader(&self, src: &String, shader_type: ShaderType) -> Result<WebGlShader, GfxError>
    {
        let shader = self.context.create_shader(shader_type.into())
            .ok_or(GfxError::ShaderCreationError(shader_type, gl_get_errors(&self.context).to_string()))?;
        self.context.shader_source(&shader, &src.as_str());
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
}

impl GlObject for ShaderProgram
{
    fn bind(&self) { self.context.use_program(Some(&self.internal)); }
    fn unbind(&self) { self.context.use_program(None); }
    fn reload(&mut self, context: &Rc<Context>) -> Result<(), GfxError>
    {
        self.context = Rc::clone(context);
        self.internal = ShaderProgram::new_program(&self.context)?;
        self.compile()?;
        self.bind();

        for (block_binding, block_name) in self.block_bindings.to_owned().iter().enumerate()
        {
            if let Some(block_name) = block_name
            {
                self.add_uniform_block_binding(block_name.as_str(), block_binding as u32)?;
            }
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