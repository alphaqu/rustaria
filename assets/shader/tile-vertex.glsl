#version 330 core

layout (location=0) in vec3 in_Position;
layout (location=1) in vec2 in_TextureCoord;

uniform vec2 player_pos;
uniform float zoom;

out vec2 outTexCoord;

void main()
{
    gl_Position = vec4((in_Position.xy - player_pos) / zoom, in_Position.z, 1);
    outTexCoord = in_TextureCoord;
}