#version 330

in vec2 v_uv;
in vec4 v_color;

uniform sampler2D buffer;

void main(void) {
	gl_FragColor = texture(buffer, v_uv) * v_color;
}
