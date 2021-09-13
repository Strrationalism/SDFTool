use opencl3::*;

pub struct Context {
    pub opencl_context: context::Context,
    edge_detect: kernel::Kernel,
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
                include_str!("program.ocl"))
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
            command_queues
        }
    }

    pub fn command_queue_count(&self) -> usize {
        self.command_queues.len()
    }

    pub fn edge_detect(
        &self, 
        command_queue: usize,
        src: &memory::Buffer<u8>,       // RGB32
        dst: &mut memory::Buffer<u8>,   // R8
        width: usize,
        height: usize,
        wait: &[event::Event])
        -> event::Event
    {
        let mut exe = kernel::ExecuteKernel::new(&self.edge_detect);
        exe
            .set_arg(src)
            .set_arg(dst)
            .set_arg(&width)
            .set_arg(&height)
            .set_global_work_sizes(&[width, height]);
            
        
        for i in wait {
            exe.set_wait_event(i);
        }

        exe
            .enqueue_nd_range(&self.command_queues[command_queue])
            .unwrap()
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
}