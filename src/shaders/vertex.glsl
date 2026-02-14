#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 ModelView;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * ModelView * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
