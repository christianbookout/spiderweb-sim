#version 330 core
out vec4 FragColor;

uniform vec4 inputColor;
uniform float radius;

void main() {
    if(gl_PointCoord.x > radius && gl_PointCoord.y > radius) {
        discard;
    }
    FragColor = inputColor;
}
