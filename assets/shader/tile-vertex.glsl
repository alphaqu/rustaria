#version 330

layout (location=0) in vec3 pos;
layout (location=1) in vec2 texCoord;

uniform vec2 glPlayerPos;
uniform float glZoom;

out vec2 outTexCoord;

void main()
{
    gl_Position = vec4((pos.xy - glPlayerPos) / glZoom, pos.z, 1.0);
    outTexCoord = texCoord;
}