use std::sync::Arc;

use web_sys::{console, window, Performance};
use wgpu::SurfaceTexture;

use crate::shared::Shared;

pub mod shader_debug;
#[macro_export]
macro_rules! console_log {
    ($($arg:tt)*) => {
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

pub fn split<'a>(
    canvas_id_name: &'a String,
    shared: Arc<Shared>,
) -> (Arc<wgpu::Queue>, SurfaceTexture, wgpu::CommandEncoder) {
    let device = shared.device.clone();
    let queue = shared.queue.clone();
    let adapter = shared.adapter.clone();

    let surface = shared
        .canvas
        .canvases
        .get(canvas_id_name)
        .unwrap()
        .surface
        .as_ref();
    let surface_clone_ref = surface.clone().unwrap();

    let default_config = surface_clone_ref
        .get_default_config(&adapter, 300, 300)
        .unwrap();
    surface_clone_ref.configure(&device, &default_config);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });
    (
        queue,
        surface_clone_ref.get_current_texture().unwrap(),
        encoder,
    )
}

pub fn split_for_update<'a>(
    canvas_id_name: &'a String,
    shared: Arc<Shared>,
    width: u32,
    height: u32,
) -> (
    Arc<wgpu::Adapter>,
    Arc<wgpu::Queue>,
    Arc<wgpu::Surface<'static>>,
) {
    let device = shared.device.clone();
    let queue = shared.queue.clone();
    let adapter = shared.adapter.clone();
    let arc_surface = shared
        .canvas
        .canvases
        .get(canvas_id_name)
        .unwrap()
        .surface
        .as_ref() // 先转换为引用
        .unwrap() // 然后解包引用
        .clone();
    let surface = arc_surface.clone();

    let default_config = surface.get_default_config(&adapter, width, height).unwrap();
    surface.configure(&device, &default_config);

    (adapter, queue, surface)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct CubeMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}
impl Default for CubeMesh {
    fn default() -> Self {
        create_cube(2.0)
    }
}
pub fn create_cube(size: f32) -> CubeMesh {
    let half = size / 2.0;

    // Define the 8 vertices of the cube
    let vertices = vec![
        // Front face
        Vertex {
            position: [-half, -half, half],
            color: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }, // 0
        Vertex {
            position: [half, -half, half],
            color: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }, // 1
        Vertex {
            position: [half, half, half],
            color: [0.0, 0.0, 1.0],
            normal: [0.0, 0.0, 1.0],
        }, // 2
        Vertex {
            position: [-half, half, half],
            color: [1.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }, // 3
        // Back face
        Vertex {
            position: [-half, -half, -half],
            color: [1.0, 0.0, 1.0],
            normal: [0.0, 0.0, -1.0],
        }, // 4
        Vertex {
            position: [half, -half, -half],
            color: [0.0, 1.0, 1.0],
            normal: [0.0, 0.0, -1.0],
        }, // 5
        Vertex {
            position: [half, half, -half],
            color: [1.0, 1.0, 1.0],
            normal: [0.0, 0.0, -1.0],
        }, // 6
        Vertex {
            position: [-half, half, -half],
            color: [0.5, 0.5, 0.5],
            normal: [0.0, 0.0, -1.0],
        }, // 7
    ];

    // Define the indices for the triangles (12 triangles, 6 faces)
    let indices = vec![
        // Front face
        0, 1, 2, 2, 3, 0, // Back face
        4, 7, 6, 6, 5, 4, // Right face
        1, 5, 6, 6, 2, 1, // Left face
        4, 0, 3, 3, 7, 4, // Top face
        3, 2, 6, 6, 7, 3, // Bottom face
        4, 5, 1, 1, 0, 4,
    ];

    CubeMesh { vertices, indices }
}

pub struct Time {
    pub delta: f32,
    pub elapsed: f32,
    pub fps: f32,
    pub frame_count: u32,
    pub last_frame_time: f32,
    pub performance: Performance,
}

impl Time {
    pub fn new() -> Self {
        let performance = window()
            .expect("No global window object found")
            .performance()
            .expect("Performance object not available");

        Self {
            delta: 0.0,
            elapsed: 0.0,
            fps: 0.0,
            frame_count: 0,
            last_frame_time: 0.0,
            performance,
        }
    }

    pub fn now(&self) -> f32 {
        (self.performance.now() / 1000.0) as f32
    }

    pub fn smooth_fps(&self) -> f32 {
        // Return current FPS if we don't have enough frames to calculate
        if self.elapsed < 1.0 || self.frame_count < 10 {
            return self.fps;
        }

        // Calculate average FPS over total runtime
        self.frame_count as f32 / self.elapsed
    }
}
