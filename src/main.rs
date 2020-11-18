#![allow(dead_code, unused_imports, unused_variables)]
use std::sync::Arc;
use image::{
    GenericImage,
    GenericImageView,
    ImageBuffer,
    RgbaImage,
    DynamicImage,
    Rgba
};

use vulkano::{
    instance::{
        Instance,
        InstanceExtensions,
        PhysicalDevice
    },
    device::{
        Device,
        DeviceExtensions,
        Features,
        QueuesIter
    },

    format::{
        Format,
        ClearValue
    },
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer
    },
    command_buffer::{
        CommandBuffer,
        AutoCommandBufferBuilder
    },
    sync::GpuFuture,
    swapchain::{
        Swapchain,
        Capabilities,
        SurfaceTransform,
        PresentMode,
        ColorSpace,
        FullscreenExclusive,
        acquire_next_image
    },
    image::{
        Dimensions,
        StorageImage,
        ImageUsage
    },
    pipeline::{
        GraphicsPipeline
    },
    framebuffer::{
        Subpass
    }
};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use vulkano_win::VkSurfaceBuild;

pub mod custom_image;
mod shaders;

fn main()  {
    // #region get image stuff
    let mut custom_image = custom_image::CustomImage::new("psyduck.png");
    custom_image.to_rgba_img();
    let custom_rgba_img = match custom_image.get_rgba_img() {
        Some(n) => n,
        None => {
            panic!("Error! No rgba image");
        }
    };
    // #endregion
    
    // #region set up vulkano
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
    };

    let physical = PhysicalDevice::enumerate(&instance)
    .next().expect("no devices available");

    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("could not find graphical queue family");


    let (device, mut queues) = {
        // do setup in this nested scope (acts like a fn)
        let device_extensions = DeviceExtensions {
            khr_storage_buffer_storage_class: true,
            khr_swapchain: true,
            .. DeviceExtensions::none()
        };
        
        Device::new(
            physical, 
            physical.supported_features(), 
            &device_extensions, 
            [(queue_family, 0.5)].iter().cloned()
        ).expect("failed to create device")
    };
    let queue = queues.next().unwrap();

    // #endregion

    // #region start using winit
    let events_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .with_title("fun app")
        .with_resizable(false)
        .build_vk_surface(&events_loop, instance.clone()).unwrap();
    // #endregion

    // #region set up swapchain
    let capabilities = surface.capabilities(physical)
        .expect("could not get device capabilities");

    let dimensions = capabilities.current_extent.unwrap_or([1280, 1024]);
    let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();
    let format = capabilities.supported_formats[0].0;

    let (swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        capabilities.min_image_count,
        format,
        dimensions,
        1,
        ImageUsage::color_attachment(),
        &queue,
        SurfaceTransform::Identity,
        alpha,
        PresentMode::Fifo,
        FullscreenExclusive::Default,
        true,
        ColorSpace::SrgbNonLinear
    ).expect("failed to create swapchain");

    let (image_index, was_sub_optimal, future)
    = acquire_next_image(swapchain.clone(), None).unwrap();

    // #endregion
    
    // #region set up image buffer (dead)

    // let img_buffer = {
    //     let vulk_img = StorageImage::new(
    //         device.clone(),
    //         Dimensions::Dim2d {width: 512, height: 512},
    //         Format::R8G8B8A8Unorm,
    //         Some(queue.family())
    //     ).unwrap();

    //     CpuAccessibleBuffer::from_data(
    //         device.clone(),
    //         BufferUsage::all(),
    //         false,
    //         custom_rgba_img
    //     ).unwrap()
    // };
    // #endregion

    // #region set up vertex buffer
    let vertex_buffer = {
        #[derive(Default, Debug, Clone)]
        struct Vertex {
            position: [f32; 2]
        }

        vulkano::impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            [
                Vertex{
                    position: [-0.5, -0.25]
                },
                Vertex{
                    position: [0.0, 0.5]
                },
                Vertex{
                    position: [0.25, -0.1]
                }
            ].iter().cloned()
        ).unwrap()
    };
    // #endregion

    // #region set up shaders
        let vs = shaders::vs::Shader::load(device.clone()).unwrap();
        let fs = shaders::fs::Shader::load(device.clone()).unwrap();
    // #endregion

    // #region set up render pass
    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap(),
    );
    // #endregion

    // #region set up pipeline
    let pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );
    // #endregion

    // #region event loop
    events_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            },
            Event::MainEventsCleared => {
                // Application update code.
    

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.
                // window.request_redraw();
                println!("dickwipe")

            },
            Event::RedrawRequested(_) => {

                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.
            },
            _ => ()
        }
    });

    // #endregion

    // #region deadcode

    //create an image buffer to send to back
    // let vulk_img = StorageImage::new(
    //     device.clone(),
    //     Dimensions::Dim2d {width: 512, height: 512},
    //     Format::R8G8B8A8Unorm,
    //     Some(queue.family())
    // ).unwrap();

    // let buffer = CpuAccessibleBuffer::from_data(
    //     device.clone(),
    //     BufferUsage::all(),
    //     false,
    //     custom_rgba_img
    // ).expect("failed to create buffer");

    // let builder = AutoCommandBufferBuilder::new(
    //     device.clone(),
    //     queue.family()
    // ).unwrap();

    // let cmd_buffer = builder.build().unwrap();
    // let finished = cmd_buffer.execute(queue.clone()).unwrap();
    // finished.then_signal_fence_and_flush().unwrap()
    //     .wait(None).unwrap();
    // #endregion

    // let buffer_content = buffer.read().unwrap();
    // // let img_from_gpu = ImageBuffer::from_raw(512, 512, &buffer_content[..]).unwrap();
    // buffer_content.save("image.png").unwrap();
}

fn set_up_vulkan() -> (std::sync::Arc<Device>, QueuesIter) {
    let instance = Instance::new(
        None,
        &InstanceExtensions::none(),
        None
    ).expect("failed to create vulkano instance");

    let physical = PhysicalDevice::enumerate(&instance)
        .next().expect("no devices available");

    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("could not find graphical queue family");

    let mut device_extensions = DeviceExtensions::none();
    device_extensions.khr_storage_buffer_storage_class = true;

    Device::new(
        physical, 
        &Features::none(), 
        &device_extensions, 
        [(queue_family, 0.5)].iter().cloned()
    ).expect("failed to create device")
}
