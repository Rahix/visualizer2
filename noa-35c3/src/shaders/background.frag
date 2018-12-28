#version 410

uniform sampler2D previous;
uniform float last_beat;
uniform float beat;
uniform float volume;
uniform float aspect;

smooth in vec2 frag_position;
smooth in vec2 frag_texcoord;

vec3 background() {
    vec2 p = frag_position;
    p.x = p.x * aspect;
    p.y -= 0.5;
    float radius = sqrt(p.x * p.x + p.y * p.y);
    float t = smoothstep(0.4, 0.38, radius * (last_beat + 1.0));
    // float t = smoothstep(0.1 + beat, 0.08 + beat, radius);
    vec3 color = vec3(0.953495, 0.476371, 0.000000);
    return color * ((radius * 2.0 + 0.5)) * t + vec3(0.0) * (1.0 - t) * (1.0 / radius);
}

void main() {
    vec4 prev_color = texture(previous, frag_texcoord);
    vec3 bg_color = background();
    gl_FragColor = vec4(prev_color.rgb + (1.0 - prev_color.a) * bg_color, 1.0);
}
