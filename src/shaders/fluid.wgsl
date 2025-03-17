// 顶点着色器
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

// 更简单的Uniforms结构，避免布局问题
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse_position: vec2<f32>,
    mouse_down: u32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// 简化的流体效果片段着色器
@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    // 正确计算UV坐标
    let uv = frag_coord.xy / uniforms.resolution;
    
    // 简单的流体效果 - 正弦波叠加
    let uv_x = uv.x + 0.1 * sin(uv.y * 10.0 + uniforms.time);
    let uv_y = uv.y + 0.1 * sin(uv.x * 10.0 + uniforms.time * 0.5);
    
    // 计算带有扭曲的颜色
    let color = vec3<f32>(
        0.5 + 0.5 * sin(uv_x * 6.28 + uniforms.time),
        0.5 + 0.5 * sin(uv_y * 6.28 + uniforms.time * 0.7),
        0.5 + 0.5 * sin((uv_x + uv_y) * 6.28 + uniforms.time * 1.3)
    );
    
    // 鼠标交互 - 创建波纹效果
    let mouse_dist = distance(uv, uniforms.mouse_position);
    let wave = sin((mouse_dist * 30.0) - uniforms.time * 5.0) * 0.1;
    let mouse_effect = 0.05 / (mouse_dist + 0.1);
    
    var final_color = color;
    
    // 根据鼠标状态应用不同效果
    if (uniforms.mouse_down != 0u) {
        // 鼠标点击时的效果
        final_color += vec3<f32>(mouse_effect * wave);
    } else {
        // 鼠标悬停时的轻微效果
        final_color += vec3<f32>(mouse_effect * 0.2);
    }
    
    return vec4<f32>(final_color, 1.0);
}
