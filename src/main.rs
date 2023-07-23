use bytemuck::{Pod, Zeroable};
use vulkano::{
    device::{DeviceExtensions, physical::{PhysicalDevice, PhysicalDeviceType}, DeviceCreateInfo, QueueCreateInfo, Device},
    instance::{Instance, InstanceCreateInfo}, swapchain::{Swapchain, SwapchainCreateInfo}, image::ImageUsage, impl_vertex, buffer::{CpuAccessibleBuffer, BufferUsage},
};
use vulkano_win::VkSurfaceBuild;
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
    println!("Hello, vulkano!");

    let required_extensions = vulkano_win::required_extensions();

    let instance = Instance::new(InstanceCreateInfo {
        enabled_extensions: required_extensions,
        enumerate_portability: true,
        ..Default::default()
    })
    .unwrap();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };

    let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance).filter(|&physical_device| {
        physical_device.supported_extensions().is_superset_of(&device_extensions)
    }).filter_map(|physical_device| {
        physical_device.queue_families().find(|&queue_family| {
            queue_family.supports_graphics() && queue_family.supports_surface(&surface).unwrap_or(false)
        }).map(|queue_family| (physical_device, queue_family))
    }).min_by_key(|(physical_device, _)| {
        println!("Physical device: {:?}, type: {:?}", physical_device.properties().device_name, physical_device.properties().device_type);
        match physical_device.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
        }
    }).expect("Noe suitable physical device found");

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            ..Default::default()
        },
    ).unwrap();

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let surface_capabilities = physical_device.surface_capabilities(&surface, Default::default()).unwrap();
        let image_format = Some(physical_device.surface_formats(&surface, Default::default()).unwrap()[0].0);
        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: surface.window().inner_size().into(),
                image_usage: ImageUsage::color_attachment(),
                composite_alpha: surface_capabilities.supported_composite_alpha.iter().next().unwrap(),
                ..Default::default()
            },
        ).unwrap()
    };

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
    struct Vertex {
        position: [f32; 2],
    }
    impl_vertex!(Vertex, position);

    let vertices = [
        Vertex {
            position: [-0.5, -0.25],
        },
        Vertex {
            position: [0.0, 0.5],
        },
        Vertex {
            position: [0.25, -0.1],
        },
    ];

    let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, vertices).unwrap();
}
