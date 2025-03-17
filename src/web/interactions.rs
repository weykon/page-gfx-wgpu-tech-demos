use std::sync::Arc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, Event};

#[wasm_bindgen]
pub fn init_interactions() -> Result<(), JsValue> {
    // 获取当前窗口中的文档对象
    let doc = window().unwrap().document().unwrap();
    // 获取页面上 id 为 "my_button" 的按钮元素
    let button = doc
        .get_element_by_id("my_button")
        .expect("Element with id `my_button` not found");
    // 创建一个闭包，用于监听点击事件
    let closure = Closure::wrap(Box::new(move |_: Event| {
        console::log_1(&"Button clicked!".into());
    }) as Box<dyn FnMut(_)>);
    // 将闭包的引用注册为按钮点击事件的回调函数
    button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    // 忘却该闭包，避免被回收
    closure.forget();
    Ok(())
}