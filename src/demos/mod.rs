use crate::{
    console_log,
    shared::{ready_paint::Scene, Shared},
    utils::split,
    App,
};
use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc, sync::Arc};

use update_scene::UpdateScene;
use wgpu::SurfaceTexture;
pub mod level2;
pub mod one_shot_scene;
pub mod test;
pub mod update_scene;
pub struct Demo {
    demos: HashMap<String, Box<dyn FnMut(Arc<Shared>)>>,
}
impl Demo {
    pub fn new() -> Self {
        Self {
            demos: HashMap::new(),
        }
    }
    pub fn add_demo(&mut self, name: &str, demo: Box<dyn FnMut(Arc<Shared>)>) {
        self.demos.insert(name.to_string(), demo);
    }
    pub fn run_all(&mut self, shared: Arc<Shared>, page_active: Rc<RefCell<bool>>) {
        console_log!("run_all");
        // 头两个 canvas 的 no update scene
        self.add_demo("canvas-1", Box::new(test::test));
        self.add_demo("canvas-2", Box::new(one_shot_scene::scene2));

        // updatable scene
        self.demos.iter_mut().for_each(|(name, demo)| {
            console_log!("run_all: {}", name);
            demo(shared.clone());
        });

        console_log!("run_all end");
        UpdateScene::run(shared.clone(), page_active);
    }
}
