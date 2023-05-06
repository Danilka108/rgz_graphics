#version 330 core

in vec3 pointPos;

uniform mat4 uScale;
uniform mat4 uRotation;

void main() {
  gl_Position = uRotation * uScale * vec4(pointPos, 1.0);
}
