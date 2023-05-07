#version 330 core

struct DirectionalLight {
  vec3 direction;

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

struct PointLight {
  vec3 position;

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

in vec3 Normal;
in vec3 FragPos;

out vec4 FragColor;

uniform DirectionalLight DirLight;
uniform vec3 ViewPos;

vec3 calcDirectionalLight(
  vec3 meterial,
  float shininess,
  DirectionalLight light,
  vec3 normal,
  vec3 viewDir
);

void main() {
  vec3 norm = normalize(Normal);
  vec3 viewDir = normalize(ViewPos - FragPos);

  vec3 objectColor = vec3(0.2, 0.5, 0.7);
  vec3 result = calcDirectionalLight(objectColor, 33, DirLight, norm, viewDir);
  FragColor = vec4(result, 1.0);
}

vec3 calcDirectionalLight(
  vec3 material,
  float shininess,
  DirectionalLight light,
  vec3 normal,
  vec3 viewDir
) {
  vec3 lightDirection = normalize(-light.direction);

  float diff = max(dot(normal, lightDirection), 0.0);

  vec3 reflectDirection = reflect(-lightDirection, normal);
  float spec = pow(max(dot(viewDir, reflectDirection), 0.0), shininess);

  // vec3 ambient = light.ambient * material;
  // vec3 diffuse = light.diffuse * diff * material;
  // vec3 specular = light.specular * spec * material;
  vec3 ambient = light.ambient * material;
  vec3 diffuse = light.diffuse * diff * material;
  vec3 specular = light.specular * spec * material;

  return (ambient + diffuse + specular);
}
