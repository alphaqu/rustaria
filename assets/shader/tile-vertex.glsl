#version 330

layout (location=0) in float id;
layout (location=1) in vec2 pos;

uniform vec2 playerPos;

out vec4 exColour;

void main()
{
    switch (int(id)) {
        case 0:
        exColour = vec4(0.0, 0.0, 0.0, 0.5);
        break;
        case 1:
        exColour = vec4(1.0, 1.0, 0.0, 0.5);
        break;
        case 2:
        exColour = vec4(0.0, 1.0, 0.0, 1.0);
        break;
        case 3:
        exColour = vec4(0.0, 1.0, 1.0, 1.0);
        break;
        case 4:
        exColour = vec4(0.0, 0.0, 1.0, 1.0);
        break;
        case 5:
        exColour = vec4(1.0, 0.0, 1.0, 1.0);
        break;
        default:
        exColour = vec4(1.0, 1.0, 1.0, 1.0);
        break;
    }
    gl_Position = vec4(pos - playerPos, 0.0, 1.0);
}