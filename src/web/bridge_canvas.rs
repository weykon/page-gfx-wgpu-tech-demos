use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use wgpu::{Device, Queue, Surface};

pub struct Canvas {
    pub canvases: HashMap<String, CanvasContext>,
}
pub struct CanvasContext {
    element: HtmlCanvasElement,
    pub surface: Option<Arc<Surface<'static>>>,
}
use crate::console_log;
use std::sync::Mutex;
use wgpu::{Adapter, Backends, InstanceDescriptor, RequestAdapterOptions, SurfaceTarget};

#[cfg(feature = "webgpu")]
pub async fn connect_canvas_arr(canvas_arr: Vec<web_sys::HtmlCanvasElement>) -> (Canvas, Adapter) {
    console_log!("准备canvas_arr");

    let mut canvas_contexts = HashMap::new();

    for canvas in canvas_arr.iter() {
        let id = canvas.id();
        canvas_contexts.insert(
            id,
            CanvasContext {
                element: canvas.clone(),
                surface: None,
            },
        );
    }
    let instance = wgpu::Instance::new(&InstanceDescriptor::default());
    console_log!("创建instance成功");
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .unwrap();
    console_log!("创建adapter成功");
    for canvas in canvas_arr.iter() {
        let surface = unsafe { instance.create_surface(SurfaceTarget::Canvas(canvas.clone())) };
        let id = canvas.id();
        canvas_contexts.get_mut(&id).unwrap().surface = Some(Arc::new(surface.unwrap()));
        console_log!("创建surface成功 : {}", id);
    }
    let canvas = Canvas {
        canvases: canvas_contexts,
    };
    console_log!("创建总canvas成功");
    (canvas, adapter)
}

#[wasm_bindgen]
#[cfg(features = "webgl")]
pub fn connect_canvas_arr(canvas_arr: Vec<web_sys::HtmlCanvasElement>) {
    let mut canvases = HashMap::new();

    for canvas in canvas_arr {
        let id = canvas.id();
        canvases.insert(
            id,
            CanvasContext {
                element: canvas,
                surface: None,
            },
        );
    }
    let canvas = Canvas { canvases };
    let canvas = Arc::new(Mutex::new(canvas));
    let canvas = Arc::clone(&canvas);
}
