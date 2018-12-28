#version 410

uniform sampler2D previous;
uniform sampler2D c3;
uniform float last_beat;
uniform float beat;
uniform float volume;
uniform float aspect;

smooth in vec2 frag_position;
smooth in vec2 frag_texcoord;

vec3 background() {
    vec2 p = frag_position;

    float t = last_beat + 1.0;

    p.x = p.x * aspect + 0.5;

    p -= vec2(0.5);
    p = p * t;
    p += vec2(0.5);

    if(p.x > 1.0 || p.x < 0.0 || p.y > 1.0 || p.y < 0.0){
        return vec3(0.0);
    }

    return texture(c3, p).rgb;
}

void main() {
    vec4 prev_color = texture(previous, frag_texcoord);
    vec3 bg_color = background();
    gl_FragColor = vec4(prev_color.rgb + (1.0 - prev_color.a) * bg_color, 1.0);
}
