#version 330 core

in uint iPolarAngleIndex;
in uint iAzimuthAngleIndex;

out uint PolarAngleIndex;
out uint AzimuthAngleIndex;

void main() {
  PolarAngleIndex = iPolarAngleIndex;
  AzimuthAngleIndex = iAzimuthAngleIndex;
}
