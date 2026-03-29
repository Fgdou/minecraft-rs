use std::{boxed, sync::{Arc, Mutex}};

use vulkano::{VulkanLibrary, command_buffer::allocator::StandardCommandBufferAllocator, device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags, physical::PhysicalDeviceType}, image::{Image, ImageUsage, view::ImageView}, instance::{Instance, InstanceCreateFlags, InstanceCreateInfo}, memory::allocator::StandardMemoryAllocator, pipeline::{DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo, graphics::{GraphicsPipelineCreateInfo, color_blend::{ColorBlendAttachmentState, ColorBlendState}, viewport::Viewport}, layout::PipelineDescriptorSetLayoutCreateInfo}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, swapchain::{Surface, Swapchain, SwapchainCreateInfo}, sync::{self, GpuFuture}};
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::Window};

pub struct Engine {
    context: Arc<Mutex<Option<RenderContext>>>,
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    memory_allocator: Arc<StandardMemoryAllocator>,
}
pub struct RenderContext {
    window: Arc<Window>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl Engine {
    pub fn new(window: &mut crate::Window) -> Self {
        let context = Arc::new(Mutex::new(None::<RenderContext>));

        let (instance, device, queue) = select_device(window.get_eventloop().unwrap());

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));


        {
            let context = context.clone();
            let device = device.clone();
            let instance = instance.clone();
            window.set_resize_callback(Box::new(move |window| {
                let mut context_option = context.lock().unwrap();
                let old_context = context_option.take();
                *context_option = Some(resize(old_context, window, device.clone(), instance.clone()));
            }));
        }

        {
            let context = context.clone();
            window.set_redraw_callback(Box::new(move || {
                if let Some(ctx) = context.lock().unwrap().as_ref() {
                    ctx.draw();
                }
            }));
        }

        Self {
            context,
            instance,
            device,
            queue,
            command_buffer_allocator,
            memory_allocator,
        }
    }
}

fn select_device(event_loop: &EventLoop<()>) -> (Arc<Instance>, Arc<Device>, Arc<Queue>) {
    let library = VulkanLibrary::new().unwrap();
    let required_extensions = Surface::required_extensions(event_loop).unwrap();

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    ).unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|device| device.supported_extensions().contains(&device_extensions))
        .filter_map(|device| {
            device.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(index, family)| {
                    family.queue_flags.intersects(QueueFlags::GRAPHICS) && device.presentation_support(index as u32, event_loop).unwrap()
                })
                .map(|i| (device, i as u32))
            })
        .min_by_key(|(device, _)| {
            match device.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            }
        })
        .unwrap();

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        }
    ).unwrap();

    let queue = queues.next().unwrap();

    (instance, device, queue)
}

fn resize(old_context: Option<RenderContext>, window: Arc<winit::window::Window>, device: Arc<Device>, instance: Arc<Instance>) -> RenderContext {
    println!("resize");

    let window_size = window.inner_size();
    let surface = Surface::from_window(instance, window.clone()).unwrap();

    let (swapchain, images) = generate_swapchain(device.clone(), window_size, surface);

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {}
        },
    ).unwrap();

    let framebuffers = generate_frame_buffers(&images, render_pass.clone());

    let pipeline = generate_pipeline(device.clone(), render_pass.clone());

    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: window_size.into(),
        depth_range: (0.0 ..= 1.0),
    };

    let previous_frame_end = Some(sync::now(device.clone()).boxed());

    RenderContext { 
        window,
        swapchain,
        render_pass,
        framebuffers,
        pipeline,
        viewport,
        previous_frame_end,
    }
}

fn generate_pipeline(device: Arc<Device>, render_pass: Arc<RenderPass>) -> Arc<GraphicsPipeline> {
    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "./shaders/shader.vert"
        }
    }
    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "./shaders/shader.frag"
        }
    }
    let vs = vs::load(device.clone()).unwrap().entry_point("main").unwrap();
    let fs = fs::load(device.clone()).unwrap().entry_point("main").unwrap();
    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages).into_pipeline_layout_create_info(device.clone()).unwrap(),
    ).unwrap();
    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: None,
            input_assembly_state: Some(Default::default()),
            viewport_state: Some(Default::default()),
            rasterization_state: Some(Default::default()),
            multisample_state: Some(Default::default()),
            color_blend_state: Some(ColorBlendState {
                attachments: vec![ColorBlendAttachmentState::default()],
                ..Default::default()
            }),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        }
    ).unwrap()
}

fn generate_swapchain(device: Arc<Device>, window_size: PhysicalSize<u32>, surface: Arc<Surface>) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let surface_capabilities = device.physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();
    let (image_format, _) = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .unwrap()[0];
    Swapchain::new(
        device,
        surface,
        SwapchainCreateInfo {
            min_image_count: surface_capabilities.min_image_count.max(2),
            image_format,
            image_extent: window_size.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .unwrap(),
            ..Default::default()
        },
    ).unwrap()
}

impl RenderContext {
    fn draw(&self) {
        println!("draw")
    }
}

fn generate_frame_buffers(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            ).unwrap()
        }).collect()
}