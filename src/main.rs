mod context;

use std::fs::File;
use clap::{App, SubCommand, Arg};
use opencl3::*;

fn main() {
    let matches = 
        App::new("SDF Tool")
            .bin_name("sdftool")
            .version("v0.1")
            .author("Strrationalism Studio")
            .about("The tool for creating signed distance field image and font")
            .subcommand(SubCommand::with_name("symbol")
                .about("Create the SDF image from a symbol in PNG format")
                .arg(Arg::with_name("INPUT")
                    .help("Symbol image in PNG format")
                    .multiple(false)
                    .required(true)
                )
                .arg(Arg::with_name("OUTPUT")
                    .help("Output path for SDF image in PNG format")
                    .multiple(false)
                    .required(true))
            )
            .subcommand(SubCommand::with_name("cl-devices")
                .about("List OpenCL devices"))
            .get_matches();

    if let Some(_) = matches.subcommand_matches("cl-devices") {
        let platforms = 
        platform::get_platforms()
            .expect("Can not get opencl platforms.");

        for platform in platforms {
            println!("Platform: {}", platform.name().expect("Can not get platform."));


            let devices = 
                platform.get_devices(device::CL_DEVICE_TYPE_ALL)
                    .expect("Can not get devices.");
                    
            for device in devices {
                let device = device::Device::new(device);
                println!("    {} ({})", 
                    device.name().unwrap(), 
                    device::device_type_text(
                        device.dev_type().unwrap()));
            }
        }
    }

    if let Some(matches) = matches.subcommand_matches("symbol") {
        let input_png = 
            png::Decoder::new(
                File::open(
                    matches
                        .value_of("INPUT")
                        .expect("Input file not given."))
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

        if frame_info.color_type != png::ColorType::Rgba {
            panic!("PNG Frame must in RGBA type.");
        }     
        
        let first_opencl_platform =
            platform::get_platforms()
                .unwrap()
                .into_iter()
                .nth(0)
                .expect("Can not get first device.");

        let context = crate::context::Context::new(first_opencl_platform);

        let mut input_image = 
            svm::SvmVec::<u8>::allocate(
                &context.opencl_context,
                image_bytes.len()
            ).unwrap();

        context.upload_svm(0, &mut input_image, image_bytes);

        let mut edge = 
            svm::SvmVec::<u8>::allocate(
                &context.opencl_context, 
                image_bytes.len() / 4
            ).unwrap();

        context.edge_detect(
            0, 
            &input_image, 
            &mut edge, 
            frame_info.width as usize, 
            frame_info.height as usize);
    }
}
