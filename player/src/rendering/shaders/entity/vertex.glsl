#version 330

layout (location=0) in vec3 in_vertex_position;
layout (location=1) in vec3 in_vertex_normal;
layout (location=2) in vec2 in_texture_coord;

// normal of the vertex in model space
out vec3 view_vertex_normal;
// position of the vertex in model space
out vec3 view_vertex_position;
// texture coordinates
out vec2 view_texture_coord;

uniform mat4 view_projection_matrix;
uniform mat4 model_matrix;
uniform mat3 normal_matrix;

void main()
{
    vec4 world_vertex_position = model_matrix * vec4(in_vertex_position, 1.0);
    gl_Position = camera.view_projection_matrix * world_vertex_position;

    view_vertex_normal = normalize(normal_matrix * in_vertex_normal);
    view_vertex_position = world_position.xyz;
    view_texture_coord = in_texture_coord; //vec2(in_texture_coord.x, -in_texture_coord.y);
}
