#version 430

const float TIME_MUL = 1;
const float MAX_NEEDED_TIME=3;
// VBO-ból érkező változók
layout( location = 0 ) in vec3 vs_in_pos;
layout( location = 1 ) in vec3 vs_in_norm;
layout( location = 2 ) in vec2 vs_in_tex;
layout( location = 3 ) in vec3[6] test;
layout( location = 9 ) in mat3 testmat3inp;

// a pipeline-ban tovább adandó értékek
layout(location = 0) out vec3 vs_out_pos;
layout(location = 1) out vec3 vs_out_norm;
layout(location = 2) out vec2 vs_out_tex;

// shader külső paraméterei - most a három transzformációs mátrixot külön-külön vesszük át
layout(set = 0, binding = 0) uniform UBO {
	vec3 vecofsize3;
    mat3 matrixsize3;
    mat4 world;
    mat4 worldIT;
    mat4 viewProj;
    float time;
} ubo;

// Just to test.
layout(binding = 1) uniform sampler2D texSampler;
void main()
{
	
	float plane_y = abs(mod(ubo.time*TIME_MUL,MAX_NEEDED_TIME*2)-MAX_NEEDED_TIME);
	// float plane_y = abs(MAX_NEEDED_TIME-(time*TIME_MUL)%(MAX_NEEDED_TIME*2));
	vec4 pos  = (ubo.world * vec4(vs_in_pos,  1));
	vec3 norm = (ubo.worldIT * vec4(vs_in_norm, 0)).xyz;
	float dist = pow((1-clamp(abs(pos.y - plane_y)*3.0,0.0,1.0))/1.5,2.0)*0.5;
	// float dist = abs(pos.y-plane_y)*1000.0;
	vec4 projected_norm=vec4(norm.x,0.0,norm.z,0.0);
	// vec3 projected_norm=vec3(1.0,0.0,0.0);
	pos =pos+projected_norm*dist;
	// pos = vec3(0.0,0.0,0.0);
	
	gl_Position = ubo.viewProj*pos;
	vs_out_norm = norm;
	vs_out_pos = pos.xyz;
	vs_out_tex = vs_in_tex;
}
