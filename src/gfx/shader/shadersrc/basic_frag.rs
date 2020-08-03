pub const SRC: &'static str = r#"

#version 300 es

out vec4 outColor;

layout(std140) uniform FragData
{
    vec3 color;
};

void main()
{
    outColor = vec4(color, 1.0f);
}
"#;