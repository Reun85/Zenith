#version 430 

layout(location=0) in vec3 vs_normal;
layout(location=1) in vec3 vs_position;

layout(location=0) out vec4 fs_out_col;

const vec3 cameraPos = vec3(-100, 0.0, 0.0);

// This can be changed before creating the pipeline. (Before calling .entrypoint on the shadermodule.)
layout(constant_id=0) const int MAX_LIGHT_COUNT = 64;

vec4 lightPos = vec4( 1.0, 0.0, 0.0, 0.0);

 vec3 La = vec3(0.0, 0.0, 0.0 );
 vec3 Ld = vec3(1.0, 1.0, 1.0 );
 vec3 Ls = vec3(1.0, 1.0, 1.0 );

 float lightConstantAttenuation    = 1.0;
 float lightLinearAttenuation      = 0.0;
 float lightQuadraticAttenuation   = 0.0;


 vec3 Ka = vec3( 1.0 );
 vec3 Kd = vec3( 1.0 );
 vec3 Ks = vec3( 1.0 );

 float Shininess = 1.0;

void main()
{
	vec3 normal = normalize( vs_normal );
	vec3 ToLight; 
	float LightDistance=0.0; 
	
	if ( lightPos.w == 0.0 ) 
	{
		
		ToLight	= lightPos.xyz;
		
	}
	else				  
	{
	
		ToLight	= lightPos.xyz - vs_position;
		LightDistance = length(ToLight);
	}
	
	ToLight = normalize(ToLight);
	
	
	float Attenuation = 1.0;
	
	
	
	vec3 Ambient = La * Ka;

	
	
	
	float DiffuseFactor = max(dot(ToLight,normal), 0.0) * Attenuation;
	vec3 Diffuse = DiffuseFactor * Ld * Kd;
	
	
	vec3 viewDir = normalize( cameraPos - vs_position ); 
	vec3 reflectDir = reflect( -ToLight, normal ); 
	
	
	
	
	float SpecularFactor = pow(max( dot( viewDir, reflectDir) ,0.0), Shininess) * Attenuation;
	vec3 Specular = SpecularFactor*Ls*Ks;

	
	
	fs_out_col = vec4( Ambient+Diffuse+Specular, 1.0 )  *vec4(0.5,0.5,0,1.0);
}
