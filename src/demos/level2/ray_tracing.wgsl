struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// 顶点着色器
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 1.0);
    out.tex_coords = in.tex_coords;
    return out;
}

// 光线结构
struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

// 光线碰撞记录
struct HitRecord {
    t: f32,
    p: vec3<f32>,
    normal: vec3<f32>,
    front_face: bool,
    material: u32, // 材质ID
    color: vec3<f32>,
}

// 球体结构
struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material: u32,
    color: vec3<f32>,
}

// 创建光线
fn create_ray(origin: vec3<f32>, direction: vec3<f32>) -> Ray {
    var ray: Ray;
    ray.origin = origin;
    ray.direction = normalize(direction);
    return ray;
}

// 获取光线在时间t的点
fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + ray.direction * t;
}

// 检测球体碰撞
fn hit_sphere(sphere: Sphere, ray: Ray, t_min: f32, t_max: f32, hit: ptr<function, HitRecord>) -> bool {
    let oc = ray.origin - sphere.center;
    let a = dot(ray.direction, ray.direction);
    let half_b = dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = half_b * half_b - a * c;
    
    if (discriminant < 0.0) {
        return false;
    }
    
    let sqrtd = sqrt(discriminant);
    
    // 找到最近的t满足t_min <= t <= t_max
    var root = (-half_b - sqrtd) / a;
    if (root < t_min || t_max < root) {
        root = (-half_b + sqrtd) / a;
        if (root < t_min || t_max < root) {
            return false;
        }
    }
    
    (*hit).t = root;
    (*hit).p = ray_at(ray, root);
    let outward_normal = ((*hit).p - sphere.center) / sphere.radius;
    let front_face = dot(ray.direction, outward_normal) < 0.0;
    (*hit).normal = select(-outward_normal, outward_normal, front_face);
    (*hit).front_face = front_face;
    (*hit).material = sphere.material;
    (*hit).color = sphere.color;
    
    return true;
}

// 随机数生成 (基于哈希函数)
fn rand(seed: vec2<f32>) -> f32 {
    return fract(sin(dot(seed, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// 材质枚举
const MATERIAL_LAMBERTIAN: u32 = 0u;
const MATERIAL_METAL: u32 = 1u;
const MATERIAL_DIELECTRIC: u32 = 2u;

// 散射函数 - Lambertian材质
fn scatter_lambertian(hit: HitRecord, rand_seed: vec2<f32>, scattered: ptr<function, Ray>, attenuation: ptr<function, vec3<f32>>) -> bool {
    let scatter_direction = hit.normal + random_unit_vector(rand_seed);
    
    // 捕获接近零的散射方向
    let adjusted_direction = select(scatter_direction, hit.normal, length_squared(scatter_direction) < 0.001);
    
    *scattered = create_ray(hit.p, adjusted_direction);
    *attenuation = hit.color;
    return true;
}

// 散射函数 - Metal材质
fn scatter_metal(ray: Ray, hit: HitRecord, fuzz: f32, rand_seed: vec2<f32>, scattered: ptr<function, Ray>, attenuation: ptr<function, vec3<f32>>) -> bool {
    let reflected = reflect(ray.direction, hit.normal);
    *scattered = create_ray(hit.p, reflected + fuzz * random_in_unit_sphere(rand_seed));
    *attenuation = hit.color;
    return dot((*scattered).direction, hit.normal) > 0.0;
}

// 散射函数 - Dielectric材质
fn scatter_dielectric(ray: Ray, hit: HitRecord, ir: f32, rand_seed: vec2<f32>, scattered: ptr<function, Ray>, attenuation: ptr<function, vec3<f32>>) -> bool {
    *attenuation = vec3<f32>(1.0);
    let refraction_ratio = select(ir, 1.0 / ir, hit.front_face);
    
    let unit_direction = normalize(ray.direction);
    let cos_theta = min(dot(-unit_direction, hit.normal), 1.0);
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    
    let cannot_refract = refraction_ratio * sin_theta > 1.0;
    var direction: vec3<f32>;
    
    if (cannot_refract || reflectance(cos_theta, refraction_ratio) > rand(rand_seed + hit.p.xy)) {
        direction = reflect(unit_direction, hit.normal);
    } else {
        direction = refract(unit_direction, hit.normal, refraction_ratio);
    }
    
    *scattered = create_ray(hit.p, direction);
    return true;
}

// 反射方程
fn reflect(v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    return v - 2.0 * dot(v, n) * n;
}

// 折射方程
fn refract(uv: vec3<f32>, n: vec3<f32>, etai_over_etat: f32) -> vec3<f32> {
    let cos_theta = min(dot(-uv, n), 1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -sqrt(abs(1.0 - length_squared(r_out_perp))) * n;
    return r_out_perp + r_out_parallel;
}

// Schlick近似
fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    var r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    return r0 + (1.0 - r0) * pow((1.0 - cosine), 5.0);
}

// 随机单位向量
fn random_unit_vector(seed: vec2<f32>) -> vec3<f32> {
    let a = rand(seed) * 2.0 * 3.14159265359;
    let z = rand(seed + vec2<f32>(0.1, 0.1)) * 2.0 - 1.0;
    let r = sqrt(1.0 - z * z);
    return vec3<f32>(r * cos(a), r * sin(a), z);
}

// 随机单位球内向量
fn random_in_unit_sphere(seed: vec2<f32>) -> vec3<f32> {
    let a = rand(seed) * 2.0 * 3.14159265359;
    let z = rand(seed + vec2<f32>(0.1, 0.1)) * 2.0 - 1.0;
    let r = rand(seed + vec2<f32>(0.2, 0.2));
    return vec3<f32>(r * cos(a), r * sin(a), z);
}

// 向量长度平方
fn length_squared(v: vec3<f32>) -> f32 {
    return dot(v, v);
}

// 获取场景中的球体
fn get_scene_spheres(time: f32, spheres: ptr<function, array<Sphere, 5>>) {
    // 地面
    (*spheres)[0] = Sphere(
        vec3<f32>(0.0, -100.5, -1.0), // 中心
        100.0, // 半径
        MATERIAL_LAMBERTIAN, // 漫反射材质
        vec3<f32>(0.5, 0.5, 0.5) // 颜色
    );
    
    // 中心球 - 玻璃材质
    (*spheres)[1] = Sphere(
        vec3<f32>(0.0, 0.0, -1.0), // 中心
        0.5, // 半径
        MATERIAL_DIELECTRIC, // 电介质材质
        vec3<f32>(1.0, 1.0, 1.0) // 颜色
    );
    
    // 左侧球 - 带漫反射材质
    (*spheres)[2] = Sphere(
        vec3<f32>(-1.0, 0.0, -1.0), // 中心
        0.5, // 半径
        MATERIAL_LAMBERTIAN, // 漫反射材质
        vec3<f32>(0.1, 0.2, 0.8) // 颜色
    );
    
    // 右侧球 - 金属材质
    (*spheres)[3] = Sphere(
        vec3<f32>(1.0, 0.0, -1.0), // 中心
        0.5, // 半径
        MATERIAL_METAL, // 金属材质
        vec3<f32>(0.8, 0.6, 0.2) // 颜色
    );
    
    // 可移动的小球
    (*spheres)[4] = Sphere(
        vec3<f32>(0.0 + sin(time) * 0.5, 0.3, -0.5), // 中心
        0.2, // 半径
        MATERIAL_METAL, // 金属材质
        vec3<f32>(0.9, 0.2, 0.2) // 颜色
    );
}

// 检查是否任何物体被击中
fn hit_world(ray: Ray, t_min: f32, t_max: f32, hit: ptr<function, HitRecord>) -> bool {
    var temp_record: HitRecord;
    var hit_anything = false;
    var closest_so_far = t_max;
    var spheres: array<Sphere, 5>;
    
    get_scene_spheres(uniforms.time, &spheres);
    
    for (var i = 0u; i < 5u; i++) {
        if (hit_sphere(spheres[i], ray, t_min, closest_so_far, &temp_record)) {
            hit_anything = true;
            closest_so_far = temp_record.t;
            *hit = temp_record;
        }
    }
    
    return hit_anything;
}

// 渲染等式 - 递归光线追踪
fn ray_color(ray: Ray, pixel_coords: vec2<f32>, depth: i32) -> vec3<f32> {
    var hit: HitRecord;
    var current_ray = ray;
    var current_attenuation = vec3<f32>(1.0);
    
    for (var i = 0; i < depth; i++) {
        if (hit_world(current_ray, 0.001, 1000.0, &hit)) {
            var scattered: Ray;
            var attenuation: vec3<f32>;
            var did_scatter = false;
            
            let seed = pixel_coords + vec2<f32>(f32(i) * 0.1, uniforms.time);
            
            if (hit.material == MATERIAL_LAMBERTIAN) {
                did_scatter = scatter_lambertian(hit, seed, &scattered, &attenuation);
            } else if (hit.material == MATERIAL_METAL) {
                let fuzz = 0.1; // 金属模糊度
                did_scatter = scatter_metal(current_ray, hit, fuzz, seed, &scattered, &attenuation);
            } else if (hit.material == MATERIAL_DIELECTRIC) {
                let refraction_index = 1.5; // 玻璃折射率
                did_scatter = scatter_dielectric(current_ray, hit, refraction_index, seed, &scattered, &attenuation);
            }
            
            if (did_scatter) {
                current_attenuation = current_attenuation * attenuation;
                current_ray = scattered;
            } else {
                return vec3<f32>(0.0);
            }
        } else {
            // 天空渐变色
            let unit_direction = normalize(current_ray.direction);
            let t = 0.5 * (unit_direction.y + 1.0);
            let sky = (1.0 - t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);
            return current_attenuation * sky;
        }
    }
    
    // 达到最大深度
    return vec3<f32>(0.0);
}

// 片元着色器 - 实现光线追踪
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect_ratio = uniforms.resolution.x / uniforms.resolution.y;
    
    // 相机参数
    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;
    
    let origin = vec3<f32>(0.0, 0.0, 0.0);
    let horizontal = vec3<f32>(viewport_width, 0.0, 0.0);
    let vertical = vec3<f32>(0.0, viewport_height, 0.0);
    let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - vec3<f32>(0.0, 0.0, focal_length);
    
    // 获取规范化设备坐标
    let uv = in.tex_coords;
    
    // 抗锯齿采样
    let pixel_coords = vec2<f32>(uv.x * uniforms.resolution.x, uv.y * uniforms.resolution.y);
    let samples = 4; // 每像素采样次数
    var pixel_color = vec3<f32>(0.0);
    
    // 为每个像素进行多次采样
    for (var s = 0; s < samples; s++) {
        let offset = vec2<f32>(
            rand(pixel_coords + vec2<f32>(f32(s) * 0.1, uniforms.time)) - 0.5,
            rand(pixel_coords + vec2<f32>(uniforms.time, f32(s) * 0.1)) - 0.5
        ) / uniforms.resolution;
        
        let adjusted_uv = uv + offset;
        
        let ray_direction = lower_left_corner + adjusted_uv.x * horizontal + adjusted_uv.y * vertical - origin;
        let ray = create_ray(origin, ray_direction);
        
        pixel_color += ray_color(ray, pixel_coords, 4); // 最大递归深度为4
    }
    
    // 平均采样结果并执行gamma校正
    pixel_color = pixel_color / f32(samples);
    pixel_color = sqrt(pixel_color); // 简单的gamma校正
    
    // 添加开发中标记水印
    let watermark_color = vec3<f32>(1.0, 1.0, 0.0);
    let watermark_text = "开发中，敬请期待";
    let is_watermark = false; // 水印逻辑，可根据坐标判断是否为水印区域
    
    // 返回最终颜色
    return vec4<f32>(pixel_color, 1.0);
}