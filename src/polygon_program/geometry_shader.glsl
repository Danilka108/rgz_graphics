#version 330 core

in uint PolarAngleIndex[1];
in uint AzimuthAngleIndex[1];

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

out vec3 Normal;
out vec3 FragPos;

uniform uint uSlicesCount;
uniform float uRadius;
uniform mat4 uModelMat;
uniform mat4 uViewMat;
uniform mat4 uProjectionMat;

vec3 anglesToPos(uint polarAngleIndex, uint azimuthAngleIndex, uint stepsCount);
vec3 calcNormal(vec3 p1, vec3 p2, vec3 p3);

void main() {
  vec3 a1 = anglesToPos(PolarAngleIndex[0] + uint(1), AzimuthAngleIndex[0], uSlicesCount);
  vec4 p1 = uProjectionMat * uViewMat * uModelMat * vec4(a1, 1.0);

  vec3 a2 = anglesToPos(PolarAngleIndex[0], AzimuthAngleIndex[0], uSlicesCount);
  vec4 p2 = uProjectionMat * uViewMat * uModelMat * vec4(a2, 1.0);

  vec3 a3 = anglesToPos(
    PolarAngleIndex[0] + uint(1),
    (AzimuthAngleIndex[0] + uint(1)) % uSlicesCount,
    uSlicesCount
  );
  vec4 p3 = uProjectionMat * uViewMat * uModelMat * vec4(a3, 1.0);

  vec3 a4 = anglesToPos(PolarAngleIndex[0], (AzimuthAngleIndex[0] + uint(1)) % uSlicesCount, uSlicesCount);
  vec4 p4 = uProjectionMat * uViewMat * uModelMat * vec4(a4, 1.0);

  Normal = mat3(transpose(inverse(uModelMat))) * calcNormal(a1, a2, a3);
  FragPos = vec3(uModelMat * vec4((a1.x + a2.x + a3.x + a4.x) / 4.0, (a1.y + a2.y + a3.y + a4.y) / 4.0, (a1.z + a2.z + a3.z + a4.z) / 4.0, 1.0));

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
