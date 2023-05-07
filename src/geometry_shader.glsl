#version 330 core

in uint PolarAngleIndex[];
in uint AzimuthAngleIndex[];

layout (points) in;
layout (line_strip, max_vertices = 3) out;

uniform uint uStepsCount;
uniform float uRadius;

uniform mat4 uScale;
uniform mat4 uCameraPos;

vec3 angles_to_pos(uint polarAngleIndex, uint azimuthAngleIndex, uint stepsCount) {
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

void main() {
  gl_Position = uCameraPos * uScale
    * vec4(angles_to_pos(PolarAngleIndex[0] + uint(1), AzimuthAngleIndex[0], uStepsCount), 1.0);
  EmitVertex();

  gl_Position = uCameraPos * uScale
    * vec4(angles_to_pos(PolarAngleIndex[0], AzimuthAngleIndex[0], uStepsCount), 1.0);
  EmitVertex();

  gl_Position = uCameraPos * uScale
    * vec4(
      angles_to_pos(PolarAngleIndex[0], (AzimuthAngleIndex[0] + uint(1)) % uStepsCount, uStepsCount),
      1.0
    );
  EmitVertex();

  EndPrimitive();
}
