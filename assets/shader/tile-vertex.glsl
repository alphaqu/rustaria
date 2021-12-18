#version 330

layout (location=0) in vec2 pos;
layout (location=1) in vec2 texCoord;

uniform vec2 glPlayerPos;
uniform float glZoom;

out vec2 outTexCoord;

void main()
{
    gl_Position = vec4((pos - glPlayerPos) / glZoom, 0.0, 1.0);
    outTexCoord = texCoord;
}