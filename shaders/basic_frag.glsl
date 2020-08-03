#version 300 es
precision mediump float;

out vec4 outColor;

/*layout(std140) uniform FragData
{
    vec3 color;
};*/

void main()
{
//    outColor = vec4(color, 1.0f);
    outColor = vec4(253.0f/255.0f, 94.0f/255.0f, 0.0f, 1.0f);
}