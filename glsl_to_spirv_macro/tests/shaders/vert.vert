#version 430
#include "utils.glsl"
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texcoord;

layout(location = 0) out vec3 long_name_to_make_sure_pvs_normal;
layout(location = 1) out vec3 long_name_to_make_sure_pvs_position;

layout(set = 0, binding = 0) uniform long_name_to_make_sure_pData {
    mat4 world;
    mat4 view;
    mat4 proj;
} long_name_to_make_sure_puniforms;
void main() {

    double x = 6.6;
    x = double_val(x);
}
