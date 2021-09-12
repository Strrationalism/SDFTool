use opencl3::*;

pub struct Context {
    platform: platform::Platform,
    opencl_context: context::Context,
    program: program::Program,
    edge_detect: kernel::Kernel,
    sdf_generate: kernel::Kernel
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
            for i in devices {
                let build_log = program.get_build_log(i).unwrap();
                println!("{}", build_log);
            }
        }

        let edge_detect = 
            kernel::Kernel::create(&program, "edge_detect").unwrap();

        panic!("SDF Generate not impl!");
        let sdf_generate =
            kernel::Kernel::create(&program, "sdf_generate").unwrap();

        Context {
            platform,
            opencl_context,
            program,
            edge_detect,
            sdf_generate
        }
    }

    pub fn _edge_detect(
        &self, 
        src: &memory::Buffer<u8>, 
        dst: &memory::Buffer<u8>,
        width: u32,
        height: u32)
    {
        self.edge_detect.set_arg(0, &src).unwrap();
        self.edge_detect.set_arg(1, &dst).unwrap();
        self.edge_detect.set_arg(2, &width).unwrap();
        self.edge_detect.set_arg(3, &height).unwrap();
    }
}