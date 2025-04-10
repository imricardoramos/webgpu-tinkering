use std::{borrow::Cow, f32::consts::PI};

use eframe::wgpu;
use egui::{Sense, Slider};
use egui_wgpu::CallbackTrait;
use nalgebra::Vector3;

use crate::{
    models::{Camera, Model},
    renderer::Renderer,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )?;
    Ok(())
}

pub struct MyEguiApp {
    models: Vec<Model>,
    camera: Camera,
}

impl MyEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let models = vec![Model::new(
            "./models/cube.obj",
            (Vector3::default(), Vector3::default(), Vector3::default()),
        )];
        let camera = Camera::new(1.0);

        Self { models, camera }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left(egui::Id::new(1234)).show(ctx, |ui| {
            ui.heading("Params");
            ui.label("Position");
            ui.add(Slider::new(&mut self.models[0].translation.x, -1.0..=1.0).text("X"));
            ui.add(Slider::new(&mut self.models[0].translation.y, -1.0..=1.0).text("Y"));
            ui.add(Slider::new(&mut self.models[0].translation.z, -1.0..=1.0).text("Z"));
            ui.label("Rotation");
            ui.add(Slider::new(&mut self.models[0].rotation.x, -PI..=PI).text("X"));
            ui.add(Slider::new(&mut self.models[0].rotation.y, -PI..=PI).text("Y"));
            ui.add(Slider::new(&mut self.models[0].rotation.z, -PI..=PI).text("Z"));
            ui.label("Scaling");
            ui.add(Slider::new(&mut self.models[0].scaling.x, -1.0..=1.0).text("X"));
            ui.add(Slider::new(&mut self.models[0].scaling.y, -1.0..=1.0).text("Y"));
            ui.add(Slider::new(&mut self.models[0].scaling.z, -1.0..=1.0).text("Z"));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.heading("Hello World!");
            let rect = ui.clip_rect();
            let response = ui.interact(rect.clone(), ui.id().with("drag_panel"), Sense::drag());
            self.camera.aspect_ratio = rect.width() / rect.height();
            if response.dragged() {
                self.camera.rotate(
                    -response.drag_motion().y / 100.0,
                    -response.drag_motion().x / 100.0,
                );
            }
            let wgpu_callback = WgpuCallback::new(self.models.clone(), self.camera.clone());
            let paint_callback = egui_wgpu::Callback::new_paint_callback(rect, wgpu_callback);
            ui.painter().add(paint_callback);
        });
    }
}

struct WgpuCallback {
    models: Vec<Model>,
    camera: Camera,
}
impl WgpuCallback {
    fn new(models: Vec<Model>, camera: Camera) -> Self {
        Self { models, camera }
    }
}
impl CallbackTrait for WgpuCallback {
    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        // let renderer = Renderer::new(Cow::Borrowed(device), Cow::Borrowed(queue));
        // let (vertex_buffer, index_buffer, bind_group, _depth_texture_view) =
        //     renderer.create_resources(screen_descriptor.size_in_pixels, &self.models, &self.camera);
        // callback_resources.insert(renderer.render_pipeline);
        // callback_resources.insert((vertex_buffer, index_buffer));
        // callback_resources.insert(bind_group);

        vec![]
    }
    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let render_pipeline = callback_resources.get::<wgpu::RenderPipeline>().unwrap();
        let (vertex_buffer, index_buffer) = callback_resources
            .get::<(wgpu::Buffer, wgpu::Buffer)>()
            .unwrap();
        let bind_group = callback_resources.get::<wgpu::BindGroup>().unwrap();
        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw_indexed(0..self.models[0].meshes[0].indices.len() as u32, 0, 0..1);
    }
}
