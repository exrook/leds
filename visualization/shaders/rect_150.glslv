#version 150 core

in vec3 a_Pos;
in vec3 a_Color;
uniform mat4 u_Proj;

out vec4 v_Color;

void main() {
    v_Color = vec4(a_Color, 1.0);
    gl_Position = u_Proj * vec4(a_Pos,1.0);
}
