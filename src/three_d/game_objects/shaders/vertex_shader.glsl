#version 330 core

in vec3 position;
in vec3 vertex_color;
out vec3 color;

void main()
{
    gl_Position = vec4(position, 1);
    color = vertex_color;
}