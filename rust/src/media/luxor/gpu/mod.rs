pub mod buffers;

use std::sync::Arc;
use anyhow::{Result, anyhow};
use tracing::{info, warn};
use wgpu;

use super::camera::CameraRig;
use super::lighting::LightingRig;
use super::raytracer::RenderQuality;
use super::scene_capture::SceneGeometry;
use buffers::{pack_scene, GpuUniforms};

pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    adapter_name: String,
}

impl GpuRenderer {
    pub fn try_new() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL | wgpu::Backends::VULKAN | wgpu::Backends::DX12,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))?;

        let adapter_name = adapter.get_info().name.clone();
        info!("[LUXOR GPU] Adapter: {} ({:?})", adapter_name, adapter.get_info().backend);

        let (device, queue) = match pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Luxor GPU"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_storage_buffer_binding_size: 256 * 1024 * 1024,
                    max_buffer_size: 256 * 1024 * 1024,
                    max_compute_workgroups_per_dimension: 65535,
                    ..wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )) {
            Ok(pair) => pair,
            Err(e) => {
                warn!("[LUXOR GPU] Device request failed: {}", e);
                return None;
            }
        };

        let shader_source = include_str!("shader.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Luxor Raytracer"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Luxor Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Luxor Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Luxor Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        info!("[LUXOR GPU] Pipeline ready — {} backend", adapter_name);
        Some(Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
            adapter_name,
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn render(
        &self,
        scene: &SceneGeometry,
        camera: &CameraRig,
        lighting: &LightingRig,
        quality: RenderQuality,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let start = std::time::Instant::now();
        let packed = pack_scene(scene, camera, lighting, quality, width, height);

        let uniforms_buf = self.create_buffer("uniforms", &packed.uniforms_bytes, wgpu::BufferUsages::UNIFORM);
        let prims_buf = self.create_storage_buffer("prims", &packed.prims_bytes);
        let bvh_buf = self.create_storage_buffer("bvh", &packed.bvh_bytes);
        let indices_buf = self.create_storage_buffer("indices", &packed.indices_bytes);
        let lights_buf = self.create_storage_buffer("lights", &packed.lights_bytes);
        let terrain_buf = self.create_storage_buffer("terrain", &packed.terrain_bytes);

        let output_size = (width * height * 4) as u64;
        let output_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging"),
            size: output_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Luxor Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniforms_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: prims_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: bvh_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: indices_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: lights_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: terrain_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 6, resource: output_buf.as_entire_binding() },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Luxor Encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Luxor Raytrace"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                (width + 7) / 8,
                (height + 7) / 8,
                1,
            );
        }

        encoder.copy_buffer_to_buffer(&output_buf, 0, &staging_buf, 0, output_size);
        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = staging_buf.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device.poll(wgpu::Maintain::Wait);

        receiver.recv()
            .map_err(|e| anyhow!("GPU readback channel error: {}", e))?
            .map_err(|e| anyhow!("GPU buffer map error: {:?}", e))?;

        let data = buffer_slice.get_mapped_range();
        let packed_pixels: &[u32] = bytemuck::cast_slice(&data);

        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for &packed in packed_pixels {
            let r = (packed >> 0) & 0xFF;
            let g = (packed >> 8) & 0xFF;
            let b = (packed >> 16) & 0xFF;
            let a = (packed >> 24) & 0xFF;
            rgba.push(r as u8);
            rgba.push(g as u8);
            rgba.push(b as u8);
            rgba.push(a as u8);
        }

        drop(data);
        staging_buf.unmap();

        let elapsed = start.elapsed();
        info!("[LUXOR GPU] Render {}x{} {:?} done in {:.1}ms ({} prims, {} lights)",
            width, height, quality, elapsed.as_secs_f64() * 1000.0,
            scene.prims.len(), lighting.lights.len());

        Ok(rgba)
    }

    fn create_buffer(&self, label: &str, data: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage,
        })
    }

    fn create_storage_buffer(&self, label: &str, data: &[u8]) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        if data.is_empty() {
            return self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 16,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
        }
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: wgpu::BufferUsages::STORAGE,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_adapter_detection() {
        match GpuRenderer::try_new() {
            Some(gpu) => {
                println!("GPU detected: {}", gpu.adapter_name());
                assert!(!gpu.adapter_name().is_empty());
            }
            None => {
                println!("No GPU available — CPU fallback would be used");
            }
        }
    }
}
