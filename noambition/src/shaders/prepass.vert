#version 410

uniform mat4 perspective_matrix;
uniform mat4 view_matrix;
uniform mat4 model_matrix;
layout(std140) uniform Colors {
    vec4 colors[32];
};
uniform float volume;

in vec4 position;
in uint color_id;

smooth out vec4 frag_position;
smooth out vec4 frag_color;

void main() {
    frag_position = model_matrix * position;
    frag_position.z += exp(-pow(frag_position.y / 5.0 - 4.0, 2.0)) * (pow(frag_position.x / 8.0, 2.0) * 2.0 + 0.1) * (volume * 30.0 + 0.1);
    frag_color = colors[color_id];
    frag_color.a = frag_color.a * (1.0 - smoothstep(15.0, 25.0, frag_position.y));
    gl_Position = perspective_matrix * view_matrix * frag_position;
}
