use opencl3::*;

pub struct Context {
    pub opencl_context: context::Context,
    edge_detect: kernel::Kernel,
    sdf_generate: kernel::Kernel,
    rgba_to_grayscale: kernel::Kernel,
    command_queues: Vec<command_queue::CommandQueue>
}

impl Context {
    pub fn new(platform: platform::Platform) -> Context {
        let devices = 
            platform
                .get_devices(device::CL_DEVICE_TYPE_ALL)
                .expect("Can not get devices from the platform.");

        let opencl_context =
            context::Context::from_devices(
                &devices,
                &vec![], 
                None, 
                std::ptr::null_mut())
                .unwrap();

        let mut program = 
            program::Program::create_from_source(
                &opencl_context,
                include_str!("program.cl"))
                .unwrap();


        if let Result::Err(_) = program.build(&devices, "") {
            println!("= Program build error =");
            for i in &devices {
                let build_log = program.get_build_log(*i).unwrap();
                println!("{}", build_log);
            }
        }

        let edge_detect = 
            kernel::Kernel::create(&program, "edge_detect").unwrap();

        let sdf_generate =
            kernel::Kernel::create(&program, "sdf_generate").unwrap();

        let rgba_to_grayscale =
            kernel::Kernel::create(&program, "rgba_to_grayscale").unwrap();

        let command_queues =
            devices
                .iter()
                .map(|x| command_queue::CommandQueue::create(
                        &opencl_context, 
                        *x, 
                        command_queue::CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE
                    ).expect("Can not create command queue.")
                )
                .collect();

        Context {
            opencl_context,
            edge_detect,
            command_queues,
            sdf_generate,
            rgba_to_grayscale
        }
    }

    pub fn _command_queue_count(&self) -> usize {
        self.command_queues.len()
    }

    pub fn edge_detect(
        &self, 
        command_queue: usize,
        src: &memory::Buffer<u8>,       // RGB32
        edge: &mut memory::Buffer<u8>,   // R8
        width: usize,
        height: usize,
        wait: &[event::Event])
        -> event::Event
    {
        let mut exe = kernel::ExecuteKernel::new(&self.edge_detect);
        exe
            .set_arg(src)
            .set_arg(edge)
            .set_arg(&(width as i32))
            .set_arg(&(height as i32))
            .set_global_work_sizes(&[width, height]);
            
        
        for i in wait {
            exe.set_wait_event(i);
        }

        exe
            .enqueue_nd_range(&self.command_queues[command_queue])
            .unwrap()
    }

    pub fn sdf_generate(
        &self,
        command_queue: usize,
        edge: &memory::Buffer<u8>,
        sdf: &mut memory::Buffer<u8>,
        edge_width: usize,
        edge_height: usize,
        sdf_width: usize,
        sdf_height: usize,
        stride: usize,
        search_radius: usize,
        wait: &[event::Event])
        -> event::Event
    {
        let mut exe = kernel::ExecuteKernel::new(&self.sdf_generate);

        exe
            .set_arg(edge)
            .set_arg(sdf)
            .set_arg(&(edge_width as i32))
            .set_arg(&(edge_height as i32))
            .set_arg(&(sdf_width as i32))
            .set_arg(&(sdf_height as i32))
            .set_arg(&(stride as i32))
            .set_arg(&(search_radius as i32))
            .set_global_work_sizes(&[sdf_width, sdf_height]);

        for i in wait {
            exe.set_wait_event(i);
        }

        exe.enqueue_nd_range(&self.command_queues[command_queue]).unwrap()
    }

    pub fn write_buffer<T>(
        &self,
        command_queue: usize,
        src: &[T],
        dst: &mut memory::Buffer<T>,
        wait: &[event::Event])
        -> event::Event 
    {
        let wait: Vec<*mut core::ffi::c_void> =
            wait
                .iter()
                .map(|x| x.get())
                .collect();

        self.command_queues[command_queue].enqueue_write_buffer(
            dst, types::CL_NON_BLOCKING, 0, src, &wait).unwrap()
    }

    pub fn read_buffer<T>(
        &self,
        command_queue: usize,
        src: &memory::Buffer<T>,
        dst: &mut [T],
        wait: &[event::Event])
        -> event::Event
    {
        let wait: Vec<*mut core::ffi::c_void> =
            wait
                .iter()
                .map(|x| x.get())
                .collect();

        self.command_queues[command_queue].enqueue_read_buffer(
            src, types::CL_NON_BLOCKING, 0, dst, &wait
        ).unwrap()
    }

    pub fn load_png(
        &self,
        command_queue: usize,
        png: &str)
        -> (memory::Buffer<u8>, usize, usize)
    {
        let input_png = 
            png::Decoder::new(
                std::fs::File::open(png)
                    .expect("Can not open the input file."));

        let mut png_info = 
            input_png
                .read_info()
                .expect("Can not read information of png.");

        let mut buf = 
            vec![0; png_info.output_buffer_size()];

        let frame_info = 
            png_info
                .next_frame(&mut buf)
                .expect("Can not read frame from png.");

        let image_bytes = &mut buf[..frame_info.buffer_size()];

        if frame_info.bit_depth != png::BitDepth::Eight {
            panic!("PNG Frame must in 8 bits.");
        }

        let mut input_image = 
            memory::Buffer::<u8>::create(
                &self.opencl_context,
                memory::CL_MEM_WRITE_ONLY,
                image_bytes.len(),
                std::ptr::null_mut()
            ).unwrap();

        self.write_buffer(command_queue, &image_bytes, &mut input_image, &[]).wait().unwrap();

        let convert_image = |stride| {
            let size = (frame_info.width * frame_info.height) as usize;
            let mut gray_scale = 
                memory::Buffer::<u8>::create(
                    &self.opencl_context,
                    0,
                    size,
                    std::ptr::null_mut()
                ).unwrap();

            let mut exe = kernel::ExecuteKernel::new(&self.rgba_to_grayscale);
            
            exe
                .set_arg(&input_image)
                .set_arg(&mut gray_scale)
                .set_arg(&stride)
                .set_global_work_sizes(&[size])
                .enqueue_nd_range(&self.command_queues[command_queue])
                .unwrap()
                .wait()
                .unwrap();

            gray_scale
        };

        let image_buffer =
            match frame_info.color_type {
                | png::ColorType::Grayscale 
                | png::ColorType::GrayscaleAlpha => 
                    input_image,
                | png::ColorType::Rgb => convert_image(3),
                | png::ColorType::Rgba => convert_image(4),
                | _ => panic!("PNG frame must in grayscale/rgb type.")
            };

        (
            image_buffer, 
            frame_info.width as usize, 
            frame_info.height as usize
        )
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        for i in &self.command_queues {
            i.finish().unwrap();
        }
    }
}
