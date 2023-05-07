#version 330 core

in uint PolarAngleIndex[1];
in uint AzimuthAngleIndex[1];

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

out vec3 Normal;
out vec3 FragPos;

uniform uint uSlicesCount;
uniform float uRadius;

uniform mat4 uScale;
uniform mat4 uCamera;

vec3 anglesToPos(uint polarAngleIndex, uint azimuthAngleIndex, uint stepsCount) {
  #define PI 3.1415926538
  #define FRAC_PI_2 1.5707963267

  float polarAngleStep = PI / float(stepsCount);
  float azimuthAngleStep = PI * 2 / float(stepsCount);

  float polarAngle = (float(polarAngleIndex) - float(stepsCount) / 2.0) * polarAngleStep;
  float azimuthAngle = float(azimuthAngleIndex) * azimuthAngleStep;

  float x = uRadius * cos(polarAngle) * sin(azimuthAngle);
  float y = uRadius * cos(polarAngle) * cos(azimuthAngle);
  float z = uRadius * sin(polarAngle) - uRadius
          * (polarAngle / FRAC_PI_2)
          * (polarAngle / FRAC_PI_2)
          * (polarAngle / FRAC_PI_2);

  return vec3(x, z, y);
}

vec3 calcNormal(vec3 p1, vec3 p2, vec3 p3) {
  vec3 u = p2 - p1;
  vec3 v = p3 - p1;

  return vec3(u.y * v.z - u.z * v.y, u.z * v.x - u.x * v.z, u.x * v.y - u.y * v.x);
}

void main() {
  vec4 p1 = uCamera * uScale
    * vec4(anglesToPos(PolarAngleIndex[0] + uint(1), AzimuthAngleIndex[0], uSlicesCount), 1.0);
  vec4 p2 = uCamera * uScale
    * vec4(anglesToPos(PolarAngleIndex[0], AzimuthAngleIndex[0], uSlicesCount), 1.0);
  vec4 p3 = uCamera * uScale
    * vec4(
      anglesToPos(PolarAngleIndex[0] + uint(1), (AzimuthAngleIndex[0] + uint(1)) % uSlicesCount, uSlicesCount),
      1.0
    );
  vec4 p4 = uCamera * uScale
    * vec4(
      anglesToPos(PolarAngleIndex[0], (AzimuthAngleIndex[0] + uint(1)) % uSlicesCount, uSlicesCount),
      1.0
    );

  Normal = calcNormal(p1.xyz, p2.xyz, p3.xyz);
  FragPos = vec3((p1.x + p4.x) / 3.0, (p1.y + p4.y) / 3.0, (p1.z + p4.z) / 3.0);

  gl_Position = p1;
  EmitVertex();
  gl_Position = p2;
  EmitVertex();
  gl_Position = p3;
  EmitVertex();
  gl_Position = p4;
  EmitVertex();
  EndPrimitive();
}
