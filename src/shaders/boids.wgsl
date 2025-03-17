// 顶点着色器
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse_position: vec2<f32>,
    boid_count: u32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// 简单的Boid效果片段着色器
@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    // 正确计算UV坐标
    let uv = frag_coord.xy / uniforms.resolution;
    
    // 创建一个简单的背景
    let bg_color = vec3<f32>(0.1, 0.2, 0.3);
    
    // 为简单起见，我们直接在片段着色器中模拟几个boid
    // 实际应用中应该在compute shader中计算boid位置
    var color = bg_color;
    var boid_count = min(10u, uniforms.boid_count);
    
    // 创建几个简单的boid
    for(var i = 0u; i < boid_count; i = i + 1u) {
        // 使用时间和索引创建伪随机移动
        let t = uniforms.time * 0.5 + f32(i) * 0.628;
        let speed = 0.2 + f32(i) * 0.02;
        
        // 生成圆形运动
        let bx = 0.5 + cos(t) * speed;
        let by = 0.5 + sin(t * 1.5) * speed;
        
        // 计算距离
        let boid_pos = vec2<f32>(bx, by);
        let dist = distance(uv, boid_pos);
        
        // 绘制boid (简单的点)
        if (dist < 0.02) {
            // 为每个boid分配不同颜色
            let boid_color = vec3<f32>(
                0.5 + 0.5 * sin(f32(i) * 0.628),
                0.5 + 0.5 * cos(f32(i) * 0.628),
                0.8
            );
            
            // 平滑混合
            color = mix(color, boid_color, smoothstep(0.02, 0.01, dist));
        }
        
        // 添加鼠标交互 - 鼠标附近的boid会发光
        let mouse_dist = distance(boid_pos, uniforms.mouse_position);
        if (mouse_dist < 0.2) {
            let glow = (0.2 - mouse_dist) * 2.0;
            color += vec3<f32>(glow * 0.5, glow * 0.3, 0.0);
        }
    }
    
    return vec4<f32>(color, 1.0);
}
