use crate::{
    console_log,
    shared::{ready_paint::Scene, Shared},
    utils::split,
    App,
};
use std::{collections::HashMap, hash::Hash, sync::Arc};

use update_scene::UpdateScene;
use wgpu::SurfaceTexture;

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
    pub fn run_all(&mut self, shared: Arc<Shared>) {
        console_log!("run_all");
        self.demos.iter_mut().for_each(|(name, demo)| {
            console_log!("run_all: {}", name);
            demo(shared.clone());
        });

        console_log!("run_all end");
        UpdateScene::run(shared.clone());
    }
}
