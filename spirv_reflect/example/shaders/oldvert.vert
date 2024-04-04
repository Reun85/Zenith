#version 430
layout(location = 0) in vec3 long_name_to_make_sure_position;
layout(location = 1) in vec3 long_name_to_make_sure_normal;
layout(location = 2) in vec2 long_name_to_make_sure_tex;

layout(location = 0) out vec3 long_name_to_make_sure_vs_normal;
layout(location = 1) out vec3 long_name_to_make_sure_vs_position;
layout(location = 2) out vec2 long_name_to_make_sure_vs_tex;

layout(set = 0, binding = 0) uniform long_name_to_make_sure_Data {
    mat4 world;
    mat4 worldIT;
    mat4 viewProj;
} long_name_to_make_sure_uniforms;

void main()
{
    vec4 p = (long_name_to_make_sure_uniforms.world   * vec4(long_name_to_make_sure_position,  1));
	gl_Position = long_name_to_make_sure_uniforms.viewProj * p;
	long_name_to_make_sure_vs_position  = p.xyz;
	long_name_to_make_sure_vs_normal = (long_name_to_make_sure_uniforms.worldIT * vec4(long_name_to_make_sure_normal, 0)).xyz;
	long_name_to_make_sure_vs_tex = long_name_to_make_sure_tex;

}
