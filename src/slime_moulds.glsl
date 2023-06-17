#version 460
#define M_PI 3.1415926535897932384626433832795

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
    int width;
    int height;

    float turn_speed;
} push_constants;


// Hash function www.cs.ubc.ca/~rbridson/docs/schechter-sca08-turbulence.pdf
uint hash(uint state)
{
    state ^= 2747636419u;
    state *= 2654435769u;
    state ^= state >> 16;
    state *= 2654435769u;
    state ^= state >> 16;
    state *= 2654435769u;
    return state;
}

float scaleToRange01(uint state)
{
    return state / 4294967295.0;
}


// fills image with black
void init() {
    ivec2 size = imageSize(img);
    ivec2 pos = ivec2(gl_GlobalInvocationID.x % push_constants.width, gl_GlobalInvocationID.x / push_constants.width);


    imageStore(img, pos, vec4(0.0, 0.0, 0.0, 1.0));
}

void update() {
    uint id = gl_GlobalInvocationID.x;

    if (id >= push_constants.num_agents) {
        return;
    }

    // process data
    SlimeAgent agent = agents[id];
    vec2 dir = vec2(cos(agent.angle), sin(agent.angle));
    vec2 new_pos = agent.pos + dir;

    ivec2 pos = ivec2(agent.pos);
    uint random = hash(pos.y * push_constants.width + pos.x * hash(gl_GlobalInvocationID.x));


    // random movement
    agents[id].angle += (scaleToRange01(random) - 0.5) * 4 * M_PI * push_constants.turn_speed;

    

    // bounce off image walls
    if (new_pos.x < 0 || new_pos.x >= push_constants.width || new_pos.y < 0 || new_pos.y >= push_constants.height) {
        new_pos.x = min(push_constants.width - 1, max(0, new_pos.x));
        new_pos.y = min(push_constants.height - 1, max(0, new_pos.y));

        agents[id].angle -= M_PI / 2;
    }


    // update position and write to image
    agents[id].pos = new_pos;
    imageStore(img, ivec2(agents[id].pos.xy), vec4(1.0, 1.0, 1.0, 1.0));
}

void main() {

    if (push_constants.step == 0) {
        init();
    } else {
        update();
    }

    
}