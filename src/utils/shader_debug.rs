use wasm_bindgen::JsValue;
use wgpu::*;

// 打印着色器编译错误的辅助函数
pub fn create_shader_with_debug(
    device: &Device,
    label: &str,
    source: &str,
) -> Result<ShaderModule, JsValue> {
    let result = device.create_shader_module(ShaderModuleDescriptor {
        label: Some(label),
        source: ShaderSource::Wgsl(source.into()),
    });

    // 在控制台记录着色器创建结果
    web_sys::console::log_1(&JsValue::from_str(&format!(
        "创建着色器模块 '{}': 成功",
        label
    )));

    // 记录着色器源码摘要
    let preview = if source.len() > 100 {
        format!("{}...", &source[..100])
    } else {
        source.to_string()
    };
    web_sys::console::log_1(&JsValue::from_str(&format!("着色器源码预览:\n{}", preview)));

    Ok(result)
}

// 验证WGSL语法
pub fn validate_wgsl(source: &str) -> bool {
    // 检查关键入口点
    let has_vertex = source.contains("fn vs_main");
    let has_fragment = source.contains("fn fs_main");

    if !has_vertex || !has_fragment {
        web_sys::console::error_1(&JsValue::from_str(&format!(
            "着色器缺少入口点: vs_main={}, fs_main={}",
            has_vertex, has_fragment
        )));
        return false;
    }

    // 基本语法检查
    let basic_checks = [
        ("@vertex", "缺少@vertex标记"),
        ("@fragment", "缺少@fragment标记"),
        ("@location", "缺少@location属性"),
        ("@builtin", "缺少@builtin属性"),
    ];

    for (pattern, error) in basic_checks {
        if !source.contains(pattern) {
            web_sys::console::error_1(&JsValue::from_str(error));
            return false;
        }
    }

    true
}
