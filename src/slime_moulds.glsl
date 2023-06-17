#version 460

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct SlimeAgent {
    vec2 pos;
    float angle;
};


layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;
layout(set = 0, binding = 1) buffer Agents {
    SlimeAgent[] agents;
};

layout(push_constant) uniform PushConstants {
    int step;
    int num_agents;
} push_constants;

// fills image with black
void init() {
    ivec2 size = imageSize(img);
    ivec2 pos = ivec2(gl_GlobalInvocationID.x % size.x, gl_GlobalInvocationID.x / size.x);

    imageStore(img, pos, vec4(0.0, 0.0, 0.0, 1.0));
}

void update() {
    uint id = gl_GlobalInvocationID.x;

    if (id >= push_constants.num_agents) {
        return;
    }

    vec2 dir = vec2(cos(agents[id].angle), sin(agents[id].angle));
    agents[id].pos += dir;
    imageStore(img, ivec2(agents[id].pos.xy), vec4(1.0, 1.0, 1.0, 1.0));
}

void main() {

    if (push_constants.step == 0) {
        init();
    } else {
        update();
    }

    
}