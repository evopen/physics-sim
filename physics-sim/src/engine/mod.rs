use std::pin::Pin;

use anyhow::{Context, Result};
use egui::FontDefinitions;
use futures::channel::mpsc::Receiver;
use wgpu::util::DeviceExt;

mod ui;
use ui::Ui;

pub struct Engine {
    window_size: winit::dpi::PhysicalSize<u32>,
    scale_factor: f64,
    swap_chain: wgpu::SwapChain,
    rt: tokio::runtime::Runtime,
    device: Pin<Box<wgpu::Device>>,
    queue: wgpu::Queue,
    debug_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    ui: Ui,
    ui_messenger: std::sync::mpsc::Receiver<ui::Message>,
}

impl Engine {
    pub async fn new(window: &winit::window::Window) -> Result<Pin<Box<Self>>> {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor();
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
            })
            .await
            .context("failed to request adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await?;
        let device = Box::pin(device);

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let debug_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("debug buffer"),
            size: 16,
            usage: wgpu::BufferUsage::STORAGE,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("main bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    min_binding_size: wgpu::BufferSize::new(16),
                    readonly: false,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(debug_buffer.slice(..)),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .worker_threads(4)
            .build()
            .unwrap();

        let ui_render_pass =
            egui_wgpu_backend::RenderPass::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);

        let (ui, ui_messenger) = Ui::new(
            device.as_ref(),
            ui::PlatformDescriptor {
                physical_width: size.width,
                physical_height: size.height,
                scale_factor: window.scale_factor(),
                font_definitions: egui::FontDefinitions::default_with_pixels_per_point(
                    window.scale_factor() as f32,
                ),
                style: Default::default(),
            },
        );

        Ok(Box::pin(Self {
            window_size,
            scale_factor,
            swap_chain,
            rt,
            device,
            queue,
            debug_buffer,
            bind_group,
            ui,
            ui_messenger,
        }))
    }

    pub fn input<T>(&mut self, winit_event: &winit::event::Event<T>) {
        self.ui.update(
            winit_event,
            &self.queue,
            &self.window_size,
            self.scale_factor,
        );
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) {
        let frame = self.swap_chain.get_current_frame().unwrap().output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });
        self.ui.render_pass.execute(
            &mut encoder,
            &frame.view,
            &self.ui.paint_jobs,
            &egui_wgpu_backend::ScreenDescriptor {
                physical_width: self.window_size.width,
                physical_height: self.window_size.height,
                scale_factor: self.scale_factor as f32,
            },
            Some(wgpu::Color::BLACK),
        );
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
