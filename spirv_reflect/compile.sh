#!/usr/bin/bash

cd example/shaders || exit
glslc vert.vert -fshader-stage=vert --target-env=vulkan1.3 --target-spv=spv1.6 -o vert
glslc frag.frag -fshader-stage=frag --target-env=vulkan1.3 --target-spv=spv1.6 -o frag
glslc sampler.frag -fshader-stage=frag --target-env=vulkan1.3 --target-spv=spv1.6 -o sampler
