struct InstanceInput {
    @location(0) top_left_position: vec2<f32>, 
    @location(1) top_right_position: vec2<f32>, 
    @location(2) bottom_left_position: vec2<f32>, 
    @location(3) bottom_right_position: vec2<f32>, 

    @location(4) color: vec4<f32>, 

    @location(5) corner_center_and_half_whd_and_radius: vec4<f32>, 
    @location(6) corner_roundness_shifts: vec4<f32>
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>, 

    @location(0) color: vec4<f32>, 
    @location(1) center_pixel: vec2<f32>, 
    @location(2) corner_roundness: vec3<f32>
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, in_instance: InstanceInput) -> VertexOutput {    
    var vertex_output: VertexOutput;

    var position = in_instance.top_left_position;
    var corner_roundness_shift = in_instance.corner_roundness_shifts.x;

    if in_vertex_index == 1u || in_vertex_index == 3u {
        position = in_instance.top_right_position;
        corner_roundness_shift = in_instance.corner_roundness_shifts.y;
    }

    if in_vertex_index == 2u || in_vertex_index == 4u {
        position = in_instance.bottom_left_position;
        corner_roundness_shift = in_instance.corner_roundness_shifts.z;
    }

    if in_vertex_index == 5u {
        position = in_instance.bottom_right_position;
        corner_roundness_shift = in_instance.corner_roundness_shifts.w;
    }

    vertex_output.position = vec4<f32>(position, 0.0, 1.0);
    vertex_output.color = in_instance.color;
    
    vertex_output.center_pixel = in_instance.corner_center_and_half_whd_and_radius.xy;
    vertex_output.corner_roundness = vec3<f32>(corner_roundness_shift, in_instance.corner_center_and_half_whd_and_radius.zw);
    
    return vertex_output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var roundness_center = input.center_pixel;
    
    let position_relative_to_center = input.position.xy - roundness_center;
    
    let width_height_half_difference = input.corner_roundness.y;
    let width_height_half_difference_abs = abs(width_height_half_difference);

    if width_height_half_difference >= 0.0 {
        if abs(position_relative_to_center.x) <= width_height_half_difference_abs {
            return input.color;
        }

        if position_relative_to_center.x >= 0.0 {
            roundness_center.x += width_height_half_difference_abs;
        } else {
            roundness_center.x -= width_height_half_difference_abs;
        }
    } else {
        if abs(position_relative_to_center.y) <= width_height_half_difference_abs {
            return input.color;
        }

        if position_relative_to_center.y >= 0.0 {
            roundness_center.y += width_height_half_difference_abs;
        } else {
            roundness_center.y -= width_height_half_difference_abs;
        }
    }

    if position_relative_to_center.x >= 0.0 {
        roundness_center.x += input.corner_roundness.x;

        if input.position.x < roundness_center.x {
            return input.color;
        }
    } else {
        roundness_center.x -= input.corner_roundness.x;

        if input.position.x > roundness_center.x {
            return input.color;
        }
    }

    if position_relative_to_center.y >= 0.0 {
        roundness_center.y += input.corner_roundness.x;

        if input.position.y < roundness_center.y {
            return input.color;
        }
    } else {
        roundness_center.y -= input.corner_roundness.x;

        if input.position.y > roundness_center.y {
            return input.color;
        }
    }

    let pixel_distance = distance(input.position.xy, roundness_center);
    let check_distance = input.corner_roundness.z - input.corner_roundness.x;

    if check_distance - 0.2 > pixel_distance {
        return input.color;
    }

    if check_distance > pixel_distance {
        return vec4<f32>(input.color.xyz, (input.color.w / 4.0) * 3.0);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}