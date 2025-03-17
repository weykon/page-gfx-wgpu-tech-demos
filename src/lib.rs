use std::sync::{Arc, Mutex};

use demos::{one_shot_scene, test};
use shared::Shared;
use web_sys::HtmlCanvasElement;

mod demos;
mod shared;
mod utils;
mod web;
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast,
};
use wgpu::{DeviceDescriptor, Limits};

#[wasm_bindgen]
pub async fn start(canvases: Vec<HtmlCanvasElement>) {
    console_error_panic_hook::set_once();
    console_log!("start");
    wasm_bindgen_futures::spawn_local(async move {
        console_log!("start async");
        let (canvas, adapter) = web::bridge_canvas::connect_canvas_arr(canvases).await;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("device descriptor"),
                    required_features: adapter.features(),
                    required_limits: Limits::downlevel_defaults(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();
        let shared = shared::Shared::new(device, queue, adapter, canvas);
        let shared = Arc::new(shared);
        let mut app = App {
            share: shared.clone(),
            demo: demos::Demo::new(),
        };
        app.demo.add_demo("canvas-1", Box::new(test::test));
        app.demo
            .add_demo("canvas-2", Box::new(one_shot_scene::scene2));
        app.demo.run_all(shared);
    });

    // web::interactions::init_interactions().unwrap();
}

struct App {
    share: Arc<Shared>,
    demo: demos::Demo,
}