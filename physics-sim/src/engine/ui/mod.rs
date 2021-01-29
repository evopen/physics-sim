use std::time::Instant;

use anyhow::Result;
pub use egui_winit_platform::PlatformDescriptor;
use futures::channel::mpsc::Sender;
use winit::dpi::PhysicalSize;

pub enum Message {
    Open,
}

pub struct Ui {
    pub render_pass: egui_wgpu_backend::RenderPass,
    pub ui_instance: egui_winit_platform::Platform,
    time: Instant,
    tx: std::sync::mpsc::Sender<Message>,
    pub paint_jobs: egui::PaintJobs,
}

impl Ui {
    pub fn new(
        device: &wgpu::Device,
        platform_descriptor: PlatformDescriptor,
    ) -> (Self, std::sync::mpsc::Receiver<Message>) {
        let render_pass =
            egui_wgpu_backend::RenderPass::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
        let platform = egui_winit_platform::Platform::new(platform_descriptor);
        let time = Instant::now();

        let (tx, rx) = std::sync::mpsc::channel();

        let paint_jobs = vec![];

        (
            Self {
                render_pass,
                ui_instance: platform,
                time,
                tx,
                paint_jobs,
            },
            rx,
        )
    }

    pub fn update<T>(
        &mut self,
        winit_event: &winit::event::Event<T>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: &winit::dpi::PhysicalSize<u32>,
        scale_factor: f64,
    ) {
        self.ui_instance
            .update_time(self.time.elapsed().as_secs_f64());
        self.ui_instance.handle_event(winit_event);

        self.draw_ui();

        self.render_pass
            .update_texture(device, &queue, &self.ui_instance.context().texture());
        self.render_pass.update_buffers(
            device,
            queue,
            &self.paint_jobs,
            &egui_wgpu_backend::ScreenDescriptor {
                physical_width: size.width,
                physical_height: size.height,
                scale_factor: scale_factor as f32,
            },
        );
    }

    fn draw_ui(&mut self) {
        self.ui_instance.begin_frame();
        egui::TopPanel::top(egui::Id::new("menu bar"))
            .show(&self.ui_instance.context().clone(), |ui| self.menu_bar(ui));
        let (_output, paint_commands) = self.ui_instance.end_frame();
        self.paint_jobs = self.ui_instance.context().tesselate(paint_commands);
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu(ui, "File", |ui| {
                if ui.button("Open").clicked {
                    self.tx.send(Message::Open).unwrap();
                }
            });
        });
    }
}
