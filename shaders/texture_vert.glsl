#version 300 es
precision mediump float;

layout(location = 0) in vec3 Vertex;
layout(location = 1) in vec2 UV;

out vec2 TexCoord;

layout(std140) uniform VertData
{
    mat4 MVP;
};

void main()
{
    TexCoord = UV;
    gl_Position = MVP * vec4(Vertex, 1.0f);
}