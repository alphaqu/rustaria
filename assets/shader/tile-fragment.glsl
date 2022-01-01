#version 330 core

in  vec2 outTexCoord;
out vec4 fragColor;

uniform sampler2D texture_sampler;

void main()
{
    // texture(texture_sampler, outTexCoord)
    fragColor = texture(texture_sampler, outTexCoord);
}