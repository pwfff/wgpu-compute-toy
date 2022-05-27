use crate::utils::set_panic_hook;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WgpuContext {
    #[wasm_bindgen(skip)]
    pub event_loop: Option<winit::event_loop::EventLoop<()>>,
    #[wasm_bindgen(skip)]
    pub window: winit::window::Window,
    #[wasm_bindgen(skip)]
    pub device: wgpu::Device,
    #[wasm_bindgen(skip)]
    pub queue: wgpu::Queue,
    #[wasm_bindgen(skip)]
    pub surface: wgpu::Surface,
    #[wasm_bindgen(skip)]
    pub surface_format: wgpu::TextureFormat,
}

#[cfg(target_arch = "wasm32")]
fn init_window(event_loop: &winit::event_loop::EventLoop<()>, bind_id: String) -> Result<winit::window::Window, Box<dyn std::error::Error>> {
    console_log::init(); // FIXME only do this once
    set_panic_hook();
    let win = web_sys::window().ok_or("window is None")?;
    let doc = win.document().ok_or("document is None")?;
    let element = doc.get_element_by_id(&bind_id).ok_or(format!("cannot find element {bind_id}"))?;
    use wasm_bindgen::JsCast;
    let canvas = element.dyn_into::<web_sys::HtmlCanvasElement>().or(Err("cannot cast to canvas"))?;
    canvas.get_context("webgpu").or(Err("no webgpu"))?.ok_or("no webgpu")?;
    use winit::platform::web::WindowBuilderExtWebSys;
    let window = winit::window::WindowBuilder::new()
        .with_canvas(Some(canvas))
        .build(event_loop)?;
    Ok(window)
}

#[cfg(not(target_arch = "wasm32"))]
fn init_window(event_loop: &winit::event_loop::EventLoop<()>, _: String) -> Result<winit::window::Window, Box<dyn std::error::Error>> {
    env_logger::init();
    winit::window::Window::new(event_loop).map_err(Box::from)
}

// FIXME: async fn(&str) doesn't currently work with wasm_bindgen: https://stackoverflow.com/a/63655324/78204
#[wasm_bindgen]
pub async fn init_wgpu(bind_id: String) -> Result<WgpuContext, String> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = init_window(&event_loop, bind_id).map_err(|e| e.to_string())?;
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: Default::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await.ok_or("unable to create adapter")?;
    let (device, queue) = adapter
        .request_device(&Default::default(), None)
        .await.map_err(|e| e.to_string())?;
    let size = window.inner_size();
    let surface_format = surface.get_preferred_format(&adapter).unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);
    surface.configure(&device, &wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo, // vsync
    });
    Ok(WgpuContext {
        event_loop: Some(event_loop),
        window,
        device,
        queue,
        surface,
        surface_format,
    })
}
