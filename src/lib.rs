use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use demos::{level2::NextLevelPage, one_shot_scene, test};
use shared::Shared;
use web_sys::{window, HtmlCanvasElement};

mod demos;
mod shared;
mod utils;
mod web;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
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
            active_pages: HashMap::new(),
        };
        let page_1_rc_active = Rc::new(RefCell::new(true));
        app.set_page("page-1", page_1_rc_active.clone());
        let app = Rc::new(RefCell::new(app));
        APP_INSTANCE.with(|app_instance| {
            *app_instance.borrow_mut() = Some(app.clone());
        });
        app.clone()
            .borrow_mut()
            .demo
            .run_all(shared.clone(), page_1_rc_active.clone());
    });

    web::interactions::init_interactions().unwrap();
}

struct App {
    share: Arc<Shared>,
    pub demo: demos::Demo,
    active_pages: HashMap<String, Rc<RefCell<bool>>>,
}

impl App {
    pub fn next_level(&mut self) {
        console_log!("next_level");
        *self.active_pages.get_mut("page-1").unwrap().borrow_mut() = false;
        let page_2_active = Rc::new(RefCell::new(true));
        self.set_page("page-2", page_2_active.clone());
        NextLevelPage::run(self.share.clone(), page_2_active.clone());
    }
    pub fn set_page(&mut self, name: &str, active: Rc<RefCell<bool>>) {
        console_log!("set_page: {:?} , {:?}", name, active);
        self.active_pages.insert(name.to_string(), active);
        console_log!("active_pages: {:?}", self.active_pages);
    }
    pub fn return_level1(&mut self) {
        console_log!("return_level1");
        *self.active_pages.get_mut("page-2").unwrap().borrow_mut() = false;
        let page_1_active = Rc::new(RefCell::new(true));
        self.set_page("page-1", page_1_active.clone());
        self.demo.run_all(self.share.clone(), page_1_active.clone());
    }
}

thread_local! {
    static APP_INSTANCE : RefCell<Option<Rc<RefCell<App>>>> = RefCell::new(None);
}
