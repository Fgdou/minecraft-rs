use std::sync::Arc;

use vulkano::{VulkanLibrary, buffer::BufferContents, command_buffer::allocator::StandardCommandBufferAllocator, descriptor_set::allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}, device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags, physical::PhysicalDeviceType}, image::{Image, ImageUsage, view::ImageView}, instance::{Instance, InstanceCreateFlags, InstanceCreateInfo}, memory::allocator::StandardMemoryAllocator, pipeline::{DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo, graphics::{GraphicsPipelineCreateInfo, color_blend::{ColorBlendAttachmentState, ColorBlendState}, vertex_input::{Vertex, VertexDefinition}}, layout::PipelineDescriptorSetLayoutCreateInfo}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, swapchain::{Surface, Swapchain, SwapchainCreateInfo}, sync::{self, GpuFuture}};
use winit::{event_loop::ActiveEventLoop, window::Window};

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct MyVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

pub struct RenderContext {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    window: Arc<Window>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    commandbuffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptorset_allocator: Arc<StandardDescriptorSetAllocator>,
}
impl RenderContext {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let required_extensions = Surface::required_extensions(event_loop).unwrap();
        let instance = Instance::new(library, InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
            ..Default::default()
        }).unwrap();

        let window = Arc::new(event_loop.create_window(Default::default()).unwrap());
        let (device, queue) = select_device(&instance, event_loop);
        let (swapchain, images) = create_swapchain(&instance, &window, &device);
        let render_pass = create_renderpass(&device, &swapchain);
        let (memory_allocator, commandbuffer_allocator, descriptorset_allocator) = create_allocators(&device);
        let framebuffers = create_framebuffers(&images, &render_pass);
        let pipeline = create_pipeline(&device, &render_pass);
        let previous_frame_end = Some(sync::now(device.clone()).boxed());
        

        Self { 
            instance,
            window,
            swapchain,
            images,
            render_pass,
            pipeline,
            device,
            queue,
            previous_frame_end,
            framebuffers,
            memory_allocator, 
            commandbuffer_allocator, 
            descriptorset_allocator,
        }
    }

    pub fn resize(&mut self) {
        self.recreate_swapchain();
        self.recreate_framebuffers();
    }

    fn recreate_swapchain(&mut self) {
        (self.swapchain, self.images) = self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: self.window.inner_size().into(),
            ..self.swapchain.create_info()
        }).unwrap();
    }

    fn recreate_framebuffers(&mut self) {
        self.framebuffers = create_framebuffers(&self.images, &self.render_pass);
    }
    
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
    
    pub fn draw(&self) {
        ()
    }
}

fn create_pipeline(device: &Arc<Device>, render_pass: &Arc<RenderPass>) -> Arc<GraphicsPipeline> {
    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "./shaders/shader.vert",
        }
    }
    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "./shaders/shader.frag",
        }
    }

    let vs = vs::load(device.clone()).unwrap().entry_point("main").unwrap();
    let fs = fs::load(device.clone()).unwrap().entry_point("main").unwrap();

    let vertex_input_state = MyVertex::per_vertex().definition(&vs).unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        device.clone(), 
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages).into_pipeline_layout_create_info(device.clone()).unwrap()
    ).unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
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

fn create_framebuffers(images: &Vec<Arc<Image>>, render_pass: &Arc<RenderPass>) -> Vec<Arc<Framebuffer>> {
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

fn create_allocators(device: &Arc<Device>) -> (Arc<StandardMemoryAllocator>, Arc<StandardCommandBufferAllocator>, Arc<StandardDescriptorSetAllocator>) {
    (
        Arc::new(StandardMemoryAllocator::new_default(device.clone())),
        Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default())),
        Arc::new(StandardDescriptorSetAllocator::new(device.clone(), Default::default())),
    )
}

fn create_renderpass(device: &Arc<Device>, swapchain: &Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
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
                depth_stencil: {},
            },
        ).unwrap()
}

fn create_swapchain(instance: &Arc<Instance>, window: &Arc<Window>, device: &Arc<Device>) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();
    let (image_format, _) = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .unwrap()[0];

    Swapchain::new(
        device.clone(),
        surface,
        SwapchainCreateInfo {
            min_image_count: surface_capabilities.min_image_count.max(2),
            image_format,
            image_extent: window.inner_size().into(),
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

fn select_device(instance: &Arc<Instance>, event_loop: &ActiveEventLoop) -> (Arc<Device>, Arc<Queue>) {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.intersects(QueueFlags::GRAPHICS) && p.presentation_support(i as u32, event_loop).unwrap()
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| {
            match p.properties().device_type {
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

    (device, queue)
}