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
    canvas_id_name: &str,
    shared: Arc<Shared>,
    width: u32,
    height: u32,
) -> (
    Arc<wgpu::Adapter>,
    Arc<wgpu::Queue>,
    Arc<wgpu::Surface<'static>>,
) {
    console_log!("2 split_for_update");
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
}
pub struct CubeMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}
impl Default for CubeMesh {
    fn default() -> Self {
        create_cube(1.)
    }
}
pub fn create_cube(size: f32) -> CubeMesh {
    let half = size / 2.0;

    //                    0 -------> 1
    //                   |           |
    //                7 -|--------4  |
    //                |  |        |  |
    //                |  |        |  |
    //                |  3 --------> 2
    //                |           |
    //                6 ----------5

    // Define the 8 vertices of the cube
    let vertices = vec![
        Vertex {
            position: [-half, half, -half],
        },
        Vertex {
            position: [half, half, -half],
        },
        Vertex {
            position: [half, -half, -half],
        },
        Vertex {
            position: [-half, -half, -half],
        },
        Vertex {
            position: [half, half, half],
        },
        Vertex {
            position: [half, -half, half],
        },
        Vertex {
            position: [-half, -half, half],
        },
        Vertex {
            position: [-half, half, half],
        },
    ];
    //                    0 -------> 1
    //                   |           |
    //                7 -|--------4  |
    //                |  |        |  |
    //                |  |        |  |
    //                |  3 --------> 2
    //                |           |
    //                6 ----------5
    // Define the indices for the triangles (12 triangles, 6 faces)
    let indices = vec![
        7, 4, 5, 5, 6, 7, // front
        5, 4, 2, 2, 4, 1, // right
        0, 1, 4, 4, 7, 0, // top
        2, 1, 0, 0, 3, 2, // back
        3, 0, 7, 7, 6, 3, // left
        3, 6, 5, 5, 2, 3, // bottom
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
 