mod context;

use std::fs::File;
use clap::{App, SubCommand, Arg};
use opencl3::*;

fn main() {
    let search_radius_arg =
        Arg::with_name("search-radius")
            .help("Set the radius for edge searching")
            .long("search-radius")
            .short("r")
            .default_value("128")
            .multiple(false);

    let stride_arg =
        Arg::with_name("stride")
            .help("Set the downsample stride size (1 will not downsample)")
            .long("stride")
            .short("s")
            .default_value("4")
            .multiple(false);

    let mut app = 
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
                .arg(Arg::with_name("platform-id")
                    .help("Select the platform to use")
                    .long("--platform-id")
                    .multiple(false)
                    .default_value("0"))
                .arg(Arg::with_name("device-id")
                    .help("Select the device to use")
                    .long("--device-id")
                    .multiple(false)
                    .default_value("0"))
                .arg(search_radius_arg)
                .arg(stride_arg))
            .subcommand(SubCommand::with_name("cl-devices")
                .about("List OpenCL devices"));

    if std::env::args().nth(1) == None {
        app.print_help().unwrap();
    }

    let matches = app.get_matches();

    if let Some(_) = matches.subcommand_matches("cl-devices") {
        let platforms = 
        platform::get_platforms()
            .expect("Can not get opencl platforms.");

        let mut platform_id = 0;

        for platform in platforms {
            println!(
                "Platform {}: {}", 
                platform_id, 
                platform.name().expect("Can not get platform."));

            platform_id = platform_id + 1;

            let devices = 
                platform.get_devices(device::CL_DEVICE_TYPE_ALL)
                    .expect("Can not get devices.");
                    
            let mut device_id = 0;

            for device in devices {
                let device = device::Device::new(device);

                println!("    {}. {} ({})", 
                    device_id,
                    device.name().unwrap(), 
                    device::device_type_text(
                        device.dev_type().unwrap()));

                device_id = device_id + 1;
            }
        }
    }

    if let Some(matches) = matches.subcommand_matches("symbol") {
        symbol(matches);
    }
}

fn symbol(matches: &clap::ArgMatches) {
    let first_opencl_platform =
        platform::get_platforms()
            .unwrap()
            .into_iter()
            .nth(matches.value_of("platform-id").unwrap_or("0").parse::<usize>().unwrap())
            .expect("Can not get the platform.");

    let device_id =
        matches.value_of("device-id").unwrap_or("0").parse::<usize>().unwrap();

    let context = crate::context::Context::new(first_opencl_platform);

    let (image, width, height) =
        context.load_png(device_id, matches.value_of("INPUT").expect("No input png given."));

    let mut edge = 
        memory::Buffer::<u8>::create(
            &context.opencl_context,
            0,
            width * height,
            std::ptr::null_mut()
        ).unwrap();

    let wait_for_edge_detect =
        context.edge_detect(
            device_id, 
            &image, 
            &mut edge, 
            width, 
            height,
            &[]);

    let stride = 
        matches.value_of("stride").unwrap().parse().unwrap();

    let search_radius = 
        matches.value_of("search-radius").unwrap().parse().unwrap();

    if stride <= 0 {
        panic!("Stride must greate or equals 1.")
    }

    let sdf_width = width / stride;
    let sdf_height = height / stride;
    let mut sdf =
        memory::Buffer::<u8>::create(
            &context.opencl_context,
            memory::CL_MEM_READ_ONLY,
            sdf_width * sdf_height,
            std::ptr::null_mut()
        ).unwrap();

    let wait_for_sdf_generate =
        context.sdf_generate(
            device_id,
            &edge,
            &mut sdf,
            width,
            height,
            sdf_width,
            sdf_height,
            stride,
            search_radius,
            &[wait_for_edge_detect]
        );

    let mut output_buf: Vec<u8> = vec![0; sdf_width * sdf_height];

    let wait_for_read_buffer =
        context.read_buffer(device_id, &sdf, &mut output_buf, &[wait_for_sdf_generate]);

    let output = 
        File::create(matches
            .value_of("OUTPUT")
            .expect("Output path not given.")
        ).unwrap();

    let ref mut w = std::io::BufWriter::new(output);

    let mut enc = 
        png::Encoder::new(w, sdf_width as u32, sdf_height as u32);

    enc.set_color(png::ColorType::Grayscale);
    enc.set_depth(png::BitDepth::Eight);

    let mut writer = enc.write_header().unwrap();
    wait_for_read_buffer.wait().unwrap();

    writer.write_image_data(&output_buf).unwrap();
}