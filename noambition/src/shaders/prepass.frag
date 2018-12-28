#version 410

smooth in vec4 frag_position;
smooth in vec4 frag_color;

void main() {
    gl_FragColor = frag_color;
}
