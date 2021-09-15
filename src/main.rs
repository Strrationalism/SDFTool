mod context;
mod charset;

use std::fs::File;
use clap::{App, SubCommand, Arg};
use opencl3::*;
use std::path::Path;

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
                .arg(search_radius_arg.clone())
                .arg(stride_arg.clone()))
            .subcommand(SubCommand::with_name("cl-devices")
                .about("List OpenCL devices"))
            .subcommand(SubCommand::with_name("font")
                .about("Create the sdf font")
                .arg(Arg::with_name("outdir")
                    .help("Output path"))
                .arg(search_radius_arg)
                .arg(stride_arg.default_value("64"))
                .arg(Arg::with_name("no-ascii")
                    .long("no-ascii")
                    .multiple(false)
                    .help("Do not generate ascii charset"))
                .arg(Arg::with_name("schinese1")
                    .long("schinese-1")
                    .multiple(false)
                    .help("Generate common standard chinese table 1"))
                .arg(Arg::with_name("schinese2")
                    .long("schinese-2")
                    .multiple(false)
                    .help("Generate common standard chinese table 2"))
                .arg(Arg::with_name("schinese3")
                    .long("schinese-3")
                    .multiple(false)
                    .help("Generate common standard chinese table 3"))
                .arg(Arg::with_name("page-width")
                    .long("page-width")
                    .default_value("1024")
                    .help("Single page width in pixels"))
                .arg(Arg::with_name("page-height")
                    .long("page-height")
                    .default_value("1024")
                    .help("Single page height in pixels"))
                .arg(Arg::with_name("margin-x")
                    .long("margin-x")
                    .default_value("0")
                    .help("Margin X on every character in pixels"))
                .arg(Arg::with_name("margin-y")
                    .long("margin-y")
                    .default_value("0")
                    .help("Margin Y on every character in pixels"))
                .arg(Arg::with_name("origin-size")
                    .long("origin-size")
                    .default_value("4096")
                    .help("Basic font size before downsample")));

    if std::env::args().nth(1) == None {
        app.print_help().unwrap();
    }

    let matches = app.get_matches();

    if let Some(_) = matches.subcommand_matches("cl-devices") {
        show_cl_devices();
    }
    
    else if let Some(matches) = matches.subcommand_matches("symbol") {
        symbol(matches);
    }
}

fn show_cl_devices() {
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

    wait_for_read_buffer.wait().unwrap();

    save_mono_png(
        &Path::new(
                matches
                    .value_of("OUTPUT")
                    .expect("Output path not given.")), 
        sdf_width,
        sdf_height, 
        &output_buf);
}

fn save_mono_png(out: &Path, width: usize, height: usize, pixels: &[u8]) {
    let output = File::create(out).unwrap();
    let w = std::io::BufWriter::new(output);

    let mut enc = 
        png::Encoder::new(w, width as u32, height as u32);

    enc.set_color(png::ColorType::Grayscale);
    enc.set_depth(png::BitDepth::Eight);

    let mut writer = enc.write_header().unwrap();
    writer.write_image_data(&pixels).unwrap();
}