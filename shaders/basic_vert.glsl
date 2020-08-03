#version 300 es

layout(location = 0) in vec3 VertPos;

layout(std140) uniform VertData
{
    mat4 MVP;
};

void main()
{
    gl_Position = MVP * vec4(VertPos, 1.0f);
}