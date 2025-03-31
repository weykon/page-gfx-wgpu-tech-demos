use std::cell::RefCell;
use std::sync::Arc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, Event};

use crate::APP_INSTANCE;

thread_local! {
    static NEXT_LEVEL: RefCell<bool> = RefCell::new(false);
}

#[wasm_bindgen]
pub fn init_interactions() -> Result<(), JsValue> {
    let doc = window().unwrap().document().unwrap();
    let button = doc
        .get_element_by_id("button")
        .expect("Element with id `button` not found");

    let restore_button = doc
        .get_element_by_id("restore-layout-btn")
        .expect("Element with id `restore_button` not found");

    let closure = Closure::wrap(Box::new(move |_: Event| {
        let level2 = NEXT_LEVEL.with(|next_level| *next_level.borrow());
        if level2 {
            console::log_1(&"Level 2".into());
            APP_INSTANCE.with(|app_instance| {
                let app_instance = app_instance.borrow();
                let app_instance = app_instance.as_ref().expect("App instance not found");
                let mut app_instance = app_instance.borrow_mut();
                app_instance.next_level();
            })
        } else {
            console::log_1(&"Level 1".into());
            APP_INSTANCE.with(|app_instance| {
                let app_instance = app_instance.borrow();
                let app_instance = app_instance.as_ref().expect("App instance not found");
                let mut app_instance = app_instance.borrow_mut();
                app_instance.return_level1();
            });
        }
    }) as Box<dyn FnMut(_)>);
    button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    restore_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

#[wasm_bindgen]
pub fn toggle_level() -> bool {
    NEXT_LEVEL.with(|next_level| {
        let mut next_level = next_level.borrow_mut();
        *next_level = !*next_level;
        *next_level
    })
}
