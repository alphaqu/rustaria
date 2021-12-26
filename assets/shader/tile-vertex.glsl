#version 330

layout (location=0) in vec3 pos;
layout (location=1) in vec2 texCoord;

uniform vec2 player_pos;
uniform float zoom;

out vec2 outTexCoord;

void main()
{
    gl_Position = vec4((pos.xy - player_pos) / zoom, pos.z, 1.0);
    outTexCoord = texCoord;
}