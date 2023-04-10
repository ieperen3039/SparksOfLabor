#version 330

// normal of the vertex in model space
in vec3 view_vertex_normal;
// position of the vertex in model space
in vec3 view_vertex_position;
// texture coordinates
in vec2 view_texture_coord;

out vec4 fragment_color;

struct PointLight
{
    vec3 color;
    vec3 position;
    float intensity;
};

struct DirectionalLight
{
    vec3 color;
    vec3 direction;
    float intensity;
};

struct AmbientLight
{
    vec3 color;
};

const float SPECULAR_REFLECTANCE = 0.1;

const int MAX_POINT_LIGHTS = 2;
const float LIGHT_CUTOFF = 0.001;

const float ATT_LIN = 0.1;
const float ATT_EXP = 0.0;

uniform light_data {
    AmbientLight ambient;
    DirectionalLight directional;
    PointLight point[MAX_POINT_LIGHTS];
} lights;

uniform vec3 camera_position;
uniform sampler2D texture_sampler;

// texture color cache
vec4 material_color;

// Blinn-Phong lighting
// calculates the diffuse and specular color component caused by one light
vec3 calcBlinnPhong(vec3 position, vec3 light_direction, vec3 normal, vec3 light_color, float light_intensity) {
    // Diffuse component
    float diffuse_power = max(dot(normal, light_direction), 0.0);
    vec3 diffuse = material_color.xyz * light_color * diffuse_power;

    // Specular component
    vec3 view_dir = normalize(camera_position - position);
    vec3 halfway_dir = normalize(light_direction + view_dir);
    float specular_power = pow(max(dot(normal, halfway_dir), 0.0), SPECULAR_REFLECTANCE);
    vec3 specular = specular_power * light_color;

    return (diffuse + specular) * light_intensity;
}

// Calculate Attenuation
// calculates the falloff of light on a given distance vector
float calcAttenuation(vec3 light_direction) {
    float distance = length(light_direction);
    return (1.0 / (1.0 + ATT_LIN * distance + ATT_EXP * distance * distance));
}

// caluclates the color addition caused by a point-light
vec3 calcPointLightComponents(PointLight light) {
    if (light.intensity == 0) return vec3(0, 0, 0);

    vec3 light_direction = light.position - view_vertex_position;
    float att = calcAttenuation(light_direction);

    if (att < LIGHT_CUTOFF) {
        return vec3(0, 0, 0);

    } else {
        float light_remainder = att * light.intensity;
        vec3 light_dir_normalized = normalize(light_direction);
        return calcBlinnPhong(light.color, view_vertex_position, light_dir_normalized, view_vertex_normal, light_remainder);
    }
}

// caluclates the color addition caused by an infinitely far away light
vec3 calcDirectionalLightComponents(DirectionalLight light) {
    if (light.intensity == 0.0){
        return vec3(0, 0, 0);

    } else {
        vec3 light_dir_normalized = normalize(light.direction);
        return calcBlinnPhong(light.color, view_vertex_position, light_dir_normalized, view_vertex_normal, light.intensity);
    }
}

float sigm(float x){
    return x / sqrt(1 + x * x * x);
}

void main() {
    material_color = texture(texture_sampler, view_texture_coord);

    // Calculate directional light
    vec3 diffuse_specular_effect = calcDirectionalLightComponents(lights.directional);

    // Calculate Point Lights
    vec3 point_light_effect[MAX_POINT_LIGHTS];
    for (int i = 0; i < MAX_POINT_LIGHTS; i++)
    {
        point_light_effect[i] = calcPointLightComponents(lights.point[i]);
    }

    // accumulate (separately to emphasise parallelism)
    for (int i = 0; i < MAX_POINT_LIGHTS; i++)
    {
        diffuse_specular_effect += point_light_effect[i];
    }

    // camera lighting
    float dot_value = dot(normalize(view_vertex_normal), normalize(camera_position - view_vertex_position));
    diffuse_specular_effect += material_color.xyz * 0.1 * max(0, dot_value);

    vec4 real_color = material_color * vec4(lights.ambient.color, 1.0) + vec4(diffuse_specular_effect, 0.0);

    fragment_color = real_color;
}
