@vertex
fn vs_main(
    @location(0) particle_pos: vec2<f32>,
    @location(1) particle_vel: vec2<f32>,
    @location(2) position: vec2<f32>,
) -> @builtin(position) vec4<f32> {
    // Add a scale factor to make the triangle smaller
    let scale = 0.1;
    
    let angle = -atan2(particle_vel.x, particle_vel.y);
    let pos = vec2<f32>(
        (position.x * scale) * cos(angle) - (position.y * scale) * sin(angle),
        (position.x * scale) * sin(angle) + (position.y * scale) * cos(angle)
    );
    return vec4<f32>(pos + particle_pos, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
