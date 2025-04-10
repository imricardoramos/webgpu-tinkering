struct VertexInput{
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) modelIdx: u32,
}
struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @location(1) modelIdx: u32,
    @builtin(position) position: vec4<f32>,
};

struct ObjectData {
  modelMatrix: mat4x4<f32>, // Position, rotation, scale
  //textureIndex: u32,        // Which texture in the array to use
};
@group(0) @binding(0) var<storage, read> objects: array<ObjectData>;
@group(0) @binding(1) var myTextures: binding_array<texture_2d<f32>>;
@group(0) @binding(2) var mySampler: sampler;
@group(0) @binding(3) var<uniform> projection: mat4x4<f32>;

@vertex
fn vs_main( input: VertexInput ) -> VertexOutput {
    let obj = objects[input.modelIdx];

    var output: VertexOutput;
    output.position = projection * obj.modelMatrix * vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    output.modelIdx = input.modelIdx;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(myTextures[in.modelIdx], mySampler, in.uv);
    //return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
