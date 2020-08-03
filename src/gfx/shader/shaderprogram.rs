use crate::
{
    Context,
    gl_get_error,
    gfx::gl_object::GLObject
};
use std::
{
    rc::Rc
};
use web_sys::{WebGlProgram, WebGlShader};

pub struct ShaderProgram
{
    internal: WebGlProgram,
    context: Rc<Context>,
}

impl ShaderProgram
{
    pub fn new(context: &Rc<Context>, vert_src: Option<&str>, frag_src: Option<&str>) -> Result<ShaderProgram, String>
    {
        if vert_src.is_none() && frag_src.is_none()
        {
            return Err("At least one shader source must be present".to_string())
        }
        let program = ShaderProgram
        {
            internal: context.create_program().ok_or_else(|| format!("Error creating shader program: {}", gl_get_error(context)))?,
            context: Rc::clone(context)
        };
        program.compile(vert_src, frag_src)?;
        Ok(program)
    }

    pub fn compile(&self, vert_src: Option<&str>, frag_src: Option<&str>) -> Result<(), String>
    {

        let vert = if let Some(src) = vert_src
        {
            Some(self.compile_shader(src, Context::VERTEX_SHADER)?)
        } else { None };

        let frag = if let Some(src) = frag_src
        {
            Some(self.compile_shader(src, Context::FRAGMENT_SHADER)?)
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
            Err(self.context.get_program_info_log(&self.internal).unwrap_or_else(|| "Unknown error linking shader".to_string()))
        }
    }
    
    fn compile_shader(&self, src: &str, shader_type: u32) -> Result<WebGlShader, String>
    {
        let shader = self.context.create_shader(shader_type).ok_or("Shader creation failed")?;
        self.context.shader_source(&shader, &src);
        self.context.compile_shader(&shader);
        self.context.attach_shader(&self.internal, &shader);

        if self.context.get_shader_parameter(&shader, Context::COMPILE_STATUS).as_bool().unwrap_or(false)
        {
            Ok(shader)
        }
        else
        {
            Err(self.context.get_shader_info_log(&shader).unwrap_or_else(|| format!("Unknown error creating {} shader", ShaderProgram::shader_name(shader_type))))
        }
    }

    fn shader_name(shader_type: u32) -> &'static str
    {
        match shader_type
        {
            Context::VERTEX_SHADER => "vertex",
            Context::FRAGMENT_SHADER => "fragment",
            _ => "unknown"
        }
    }

}

impl GLObject for ShaderProgram
{
    fn bind(&self)
    {
        self.context.use_program(Some(&self.internal));
    }

    fn unbind(&self)
    {
        self.context.use_program(None);
    }
}

impl Drop for ShaderProgram
{
    fn drop(&mut self)
    {
        self.context.delete_program(Some(&self.internal));
    }
}