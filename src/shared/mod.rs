use crate::web::bridge_canvas::Canvas;
use std::{cell::RefCell, rc::Rc, sync::Arc};
use wgpu::{Adapter, Device, Queue};
pub mod ready_paint;
pub struct Shared {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub adapter: Arc<Adapter>,
    pub canvas: Canvas,
    pub configs: Arc<RefCell<Vec<wgpu::SurfaceConfiguration>>>,
}
impl Shared {
    pub fn new(device: Device, queue: Queue, adapter: Adapter, canvas: Canvas) -> Self {
        Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter: Arc::new(adapter),
            canvas,
            configs: Arc::new(RefCell::new(Vec::new())),
        }
    }
}
