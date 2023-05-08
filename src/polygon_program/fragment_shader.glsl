#version 330 core

struct DirectionalLight {
  vec3 direction;

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

struct PointLight {
  vec3 position;

  float constant;
  float linear;
  float quadratic;

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

struct Material {
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
  float shininess;
};

in vec3 Normal;
in vec3 FragPos;

out vec4 FragColor;

uniform DirectionalLight uDirLight;
uniform PointLight uPointLight;
uniform Material uMaterial;
uniform vec3 uViewPos;

vec3 calcDirectionalLight(
  Material meterial,
  DirectionalLight light,
  vec3 normal,
  vec3 viewDir
);

vec3 calcPointLight(
  Material material,
  PointLight light,
  vec3 normal,
  vec3 fragPos,
  vec3 viewDir
);

void main() {
  vec3 norm = normalize(Normal);
  vec3 viewDir = normalize(uViewPos - FragPos);
  
  vec3 result = calcDirectionalLight(uMaterial, uDirLight, norm, viewDir);
  result += calcPointLight(uMaterial, uPointLight, norm, FragPos, viewDir);

  FragColor = vec4(result, 1.0);
}

vec3 calcDirectionalLight(
  Material material,
  DirectionalLight light,
  vec3 normal,
  vec3 viewDir
) {
  vec3 lightDirection = normalize(-light.direction);

  float diff = max(dot(normal, lightDirection), 0.0);

  vec3 reflectDirection = reflect(-lightDirection, normal);
  float spec = pow(max(dot(viewDir, reflectDirection), 0.0), material.shininess * 128.0);

  vec3 ambient = light.ambient * material.ambient;
  vec3 diffuse = light.diffuse * diff * material.diffuse;
  vec3 specular = light.specular * spec * material.specular;

  return (ambient + diffuse + specular);
}

vec3 calcPointLight(
  Material material,
  PointLight light,
  vec3 normal,
  vec3 fragPos,
  vec3 viewDir
) {
  vec3 lightDirection = normalize(light.position - fragPos);

  float diff = max(dot(normal, lightDirection), 0.0);

  vec3 reflectDirection = reflect(-lightDirection, normal);
  float spec = pow(max(dot(viewDir, reflectDirection), 0.0), material.shininess * 128.0);

  float distance = length(light.position - fragPos);
  float attenuation = 1.0 /
    (light.constant + light.linear * distance + light.quadratic * distance * distance);

  vec3 ambient = attenuation * light.ambient * material.ambient;
  vec3 diffuse = attenuation * light.diffuse * diff * material.diffuse;
  vec3 specular = attenuation * light.specular * spec * material.specular;

  return (ambient + diffuse + specular);
}
