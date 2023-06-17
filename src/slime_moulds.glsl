#version 460
#define M_PI 3.1415926535897932384626433832795

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct SlimeAgent {
    vec2 pos;
    float angle;
};


layout(set = 0, binding = 0, rgba8) uniform image2D img;


layout(set = 0, binding = 1) buffer Agents {
    SlimeAgent[] agents;
};

layout(push_constant) uniform PushConstants {
    int step;
    int num_agents;
    int width;
    int height;

    float turn_speed;
    float move_speed;
    float sense_distance;
    float sensor_angle;
    int sensor_size;

    float decay_rate;
    float diffuse_rate;
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

float sense(SlimeAgent agent, float sensor_angle_offset) {
    float sensor_angle = agent.angle + sensor_angle_offset;
    vec2 sensor_dir = vec2(cos(sensor_angle), sin(sensor_angle));

    vec2 sensor_centre = agent.pos + sensor_dir * push_constants.sense_distance;
    int centre_x = clamp(int(sensor_centre.x), 0, push_constants.width - 1);
    int centre_y = clamp(int(sensor_centre.y), 0, push_constants.height - 1);

    float sum = 0;

	for (int offset_x = -push_constants.sensor_size; offset_x <= push_constants.sensor_size; offset_x ++) {
		for (int offset_y = -push_constants.sensor_size; offset_y <= push_constants.sensor_size; offset_y ++) {
			int sample_x = clamp(centre_x + offset_x, 0, push_constants.width - 1);
			int sample_y = clamp(centre_y + offset_y, 0, push_constants.height - 1);
			sum += imageLoad(img, ivec2(sample_x, sample_y)).z;
		}
	}

    return sum;
}


// fills image with black
void init() {
    ivec2 pos = ivec2(gl_GlobalInvocationID.x % (push_constants.width + 2), gl_GlobalInvocationID.x / (push_constants.width + 2));


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
    vec2 new_pos = agent.pos + dir * push_constants.move_speed;

    ivec2 pos = ivec2(agent.pos);
    uint random = hash(pos.y * push_constants.width + pos.x * hash(gl_GlobalInvocationID.x));


    // sensing

    float random_steer = scaleToRange01(random);

    float sensorAngleRad = push_constants.sensor_angle;
	float weightForward = sense(agent, 0);
	float weightLeft = sense(agent, sensorAngleRad);
	float weightRight = sense(agent, -sensorAngleRad);

    // Continue in same direction
	if (weightForward > weightLeft && weightForward > weightRight) {
		agents[id].angle += 0;
	}
	else if (weightForward < weightLeft && weightForward < weightRight) {
		agents[id].angle += (random_steer - 0.5) * 2 * push_constants.turn_speed;
	}
	// Turn right
	else if (weightRight > weightLeft) {
		agents[id].angle -= random_steer * push_constants.turn_speed;
	}
	// Turn left
	else if (weightLeft > weightRight) {
		agents[id].angle += random_steer * push_constants.turn_speed;
	}


    // bounce off image walls
    if (new_pos.x < 0 || new_pos.x >= push_constants.width || new_pos.y < 0 || new_pos.y >= push_constants.height) {
        new_pos.x = min(push_constants.width - 1, max(0, new_pos.x));
        new_pos.y = min(push_constants.height - 1, max(0, new_pos.y));

        agents[id].angle = scaleToRange01(hash(random)) * 2 * M_PI;
    }


    // update position and write to image
    agents[id].pos = new_pos;
    imageStore(img, ivec2(agents[id].pos.xy), vec4(0.7, 0.0, 1.0, 1.0));
}

void diffuse() {
    ivec2 pos = ivec2(gl_GlobalInvocationID.x % push_constants.width, gl_GlobalInvocationID.x / push_constants.width);

    if (pos.x < 0 || pos.x >= push_constants.width || pos.y < 0 || pos.y >= push_constants.height) {
		return;
	}

	vec4 sum = vec4(0.0);
	vec4 originalCol = imageLoad(img, pos).xyzw;
	// 3x3 blur
	for (int offset_x = -1; offset_x <= 1; offset_x ++) {
		for (int offset_y = -1; offset_y <= 1; offset_y ++) {
			int sample_x = min(push_constants.width-1, max(0, pos.x + offset_x));
			int sample_y = min(push_constants.height-1, max(0, pos.y + offset_y));
			sum += imageLoad(img, ivec2(sample_x, sample_y)).xyzw;
		}
	}

	vec4 blurredCol = sum / 9;
	float diffuseWeight = clamp(push_constants.diffuse_rate, 0, 1);
	blurredCol = originalCol * (1 - diffuseWeight) + blurredCol * (diffuseWeight);

    vec4 new_col = max(vec4(0.0), blurredCol - vec4(push_constants.decay_rate));
    // vec3 old_colour = imageLoad(img, pos).xyz;
    // imageStore(img, pos, vec4(old_colour - push_constants.decay_rate, 1.0));
     imageStore(img, pos, vec4(new_col.xyz, 1.0));
}

void main() {

    if (push_constants.step == 0) {
        init();
    } else if (push_constants.step == 1){
        update();
    } else {
        diffuse();
    }

    
}