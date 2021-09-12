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

    pub fn edge_detect(
        &self, 
        command_queue: usize,
        src: &svm::SvmVec<u8>,
        dst: &mut svm::SvmVec<u8>,
        width: usize,
        height: usize)
    {
        kernel::ExecuteKernel::new(&self.edge_detect)
            .set_arg_svm(src.as_ptr())
            .set_arg_svm(dst.as_mut_ptr())
            .set_arg(&width)
            .set_arg(&height)
            .set_global_work_sizes(&[width, height])
            .enqueue_nd_range(&self.command_queues[command_queue])
            .unwrap();
        
    }

    pub fn do_svm<T, F>(&self, cmd_queue: usize, flags: u64, buf: &mut svm::SvmVec<T>, operation: F) 
        where F: Fn(&mut svm::SvmVec<T>) 
    {
        let cmd_queue = &self.command_queues[cmd_queue];
        if !buf.is_fine_grained() {
            cmd_queue.enqueue_svm_map(
                types::CL_BLOCKING, 
                flags, 
                buf, 
                &[]
            ).unwrap();
        }

        operation(buf);

        if !buf.is_fine_grained() {
            let unmap_test_values_event = 
                cmd_queue.enqueue_svm_unmap(&buf, &[]).unwrap();
            unmap_test_values_event.wait().unwrap();
        }
    }

    pub fn upload_svm<T>(&self, cmd_queue: usize, buf: &mut svm::SvmVec<T>, data: &mut [T]) where T: Clone {
        self.do_svm(cmd_queue, memory::CL_MAP_WRITE, buf, |buf| buf.clone_from_slice(data));
    }
}