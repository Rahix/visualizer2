#version 410

in vec4 position;
in vec2 texcoord;

smooth out vec2 frag_position;
smooth out vec2 frag_texcoord;

void main() {
    frag_texcoord = texcoord;
    frag_position = position.xy;
    gl_Position = position;
}
