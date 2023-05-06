#version 330 core

in vec3 pointPos;

uniform mat4 uScale;
uniform mat4 uCameraPos;

void main() {
  gl_Position = uCameraPos * uScale * vec4(pointPos, 1.0);
}
