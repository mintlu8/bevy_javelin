
#import bevy_pbr::{
    forward_io::VertexOutput,
    pbr_fragment::pbr_input_from_standard_material,
    mesh_view_bindings::globals,
}

@group(2) @binding(100) var voronoi: texture_2d<f32>;
@group(2) @binding(101) var voronoi_sampler: sampler;
@group(2) @binding(102) var by: f32;

@fragment
fn fragment(
    input: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    var pbr_input = pbr_input_from_standard_material(input, is_front);
    let voronoi = textureSample(voronoi, voronoi_sampler, input.uv);
    return pbr_input.material.base_color * mix(vec4(0.0), voronoi, noise);
}