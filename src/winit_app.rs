use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use image::{ImageReader, RgbaImage};
use indexmap::IndexMap;
use log::info;
use nalgebra::Vector3;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

use crate::models::{Camera, Model};
use crate::renderer::{self, Renderer};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = MyWinitApp::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

#[derive(Default)]
struct MyWinitApp {
    state: Option<AppState<'static>>,
    dragging: (bool, Option<(f64, f64)>),
}
struct AppState<'a> {
    window: Arc<Window>,
    renderer: Renderer<'a>,
    scene: Scene,
}
impl<'a> AppState<'a> {
    fn window_size(&self) -> [u32; 2] {
        [
            self.window.inner_size().width,
            self.window.inner_size().height,
        ]
    }
}

impl ApplicationHandler for MyWinitApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let viewport_size = [window.inner_size().width, window.inner_size().height];
        let scene = Scene::new(viewport_size);
        let (instance, _adapter, device, queue) = pollster::block_on(renderer::init());
        let mut renderer = Renderer::new(
            Cow::Owned(device),
            Cow::Owned(queue),
            scene.textures_map.len(),
        );
        let surface = instance.create_surface(window.clone()).unwrap();
        renderer.add_surface(viewport_size, surface);
        self.state = Some(AppState {
            window,
            renderer,
            scene,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Closing");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let scene = &self.state.as_ref().unwrap().scene;
                let renderer = &self.state.as_ref().unwrap().renderer;
                let window_size = self.state.as_ref().unwrap().window_size();
                renderer.render(window_size, scene).unwrap();
            }
            WindowEvent::KeyboardInput {
                device_id: _device_id,
                event,
                is_synthetic: _is_synthetic,
            } => match event.physical_key {
                winit::keyboard::PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.state.as_mut().unwrap().scene.camera.rotate(0.0, -0.1);
                    self.state.as_ref().unwrap().window.request_redraw();
                }
                winit::keyboard::PhysicalKey::Code(KeyCode::ArrowRight) => {
                    self.state.as_mut().unwrap().scene.camera.rotate(0.0, 0.1);
                    self.state.as_ref().unwrap().window.request_redraw();
                }
                winit::keyboard::PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.state.as_mut().unwrap().scene.camera.rotate(0.1, 0.0);
                    self.state.as_ref().unwrap().window.request_redraw();
                }
                winit::keyboard::PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.state.as_mut().unwrap().scene.camera.rotate(-0.1, 0.0);
                    self.state.as_ref().unwrap().window.request_redraw();
                }
                _ => {}
            },
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => self.dragging = (state.is_pressed(), None),
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if self.dragging.0 {
                    match self.dragging.1 {
                        Some(previous_position) => {
                            let delta_x = position.x - previous_position.0;
                            let delta_y = position.y - previous_position.1;
                            self.state
                                .as_mut()
                                .unwrap()
                                .scene
                                .camera
                                .rotate((-delta_y / 200.0) as f32, (-delta_x / 200.0) as f32);
                            self.state.as_ref().unwrap().window.request_redraw();
                            self.dragging.1 = Some((position.x, position.y))
                        }
                        None => self.dragging.1 = Some((position.x, position.y)),
                    }
                }
            }
            _ => {}
        }
    }
}

pub struct Scene {
    pub models: Vec<Model>,
    pub textures_map: IndexMap<String, RgbaImage>,
    pub camera: Camera,
}
impl Scene {
    fn new(viewport_dimensions: [u32; 2]) -> Self {
        let teapot = Model::new(
            "./models/teapot.obj",
            (
                Vector3::new(1.0, 1.0, 1.0),
                Vector3::default(),
                Vector3::new(0.01, 0.01, 0.01),
            ),
        );
        let cube = Model::new(
            "./models/cube.obj",
            (
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::default(),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );
        let models = vec![teapot, cube];
        let mut textures_map = IndexMap::new();
        for model in &models {
            for material in &model.materials {
                match &material.diffuse_texture {
                    Some(dt_name) => {
                        let dt_path = format!("./models/{dt_name}");
                        let dt_data = ImageReader::open(&dt_path)
                            .unwrap()
                            .decode()
                            .unwrap()
                            .to_rgba8();
                        textures_map.insert(material.name.clone(), dt_data);
                    }
                    None => {}
                }
            }
        }
        let camera = Camera::new(viewport_dimensions[0] as f32 / viewport_dimensions[1] as f32);
        Self {
            models,
            camera,
            textures_map,
        }
    }
}
