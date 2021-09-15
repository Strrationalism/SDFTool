mod context;
mod charset;
mod basic_font_generator;
mod mono_image;

use basic_font_generator::*;
use clap::{App, Arg, ArgMatches, SubCommand};
use opencl3::*;
use std::path::Path;

use crate::mono_image::MonoImage;

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
                .arg(Arg::with_name("INPUT")
                    .help("Input ttf/otf file")
                    .required(true)
                    .multiple(false))
                .arg(Arg::with_name("OUTDIR")
                    .help("Output path")
                    .required(true)
                    .multiple(false))
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
                .arg(Arg::with_name("origin-scale")
                    .long("origin-scale")
                    .default_value("4096")
                    .help("Basic font scale before downsample"))
                /*.arg(Arg::with_name("page-width")
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
                    .help("Margin X on every sdf character in pixels"))
                .arg(Arg::with_name("margin-y")
                    .long("margin-y")
                    .default_value("0")
                    .help("Margin Y on every sdf character in pixels"))*/
                .arg(Arg::with_name("padding-x")
                    .long("padding-x")
                    .default_value("0")
                    .help("Padding X on every basic character in pixels"))
                .arg(Arg::with_name("padding-y")
                    .long("padding-y")
                    .default_value("0")
                    .help("Padding Y on every basic character in pixels")));

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

    else if let Some(matches) = matches.subcommand_matches("font") {
        font(matches);
    }
}

fn font(args: &ArgMatches) {
    let basic_gen = BasicFontGenerator::from(args);
    basic_gen.generate('あ').save_png(&Path::new("out.png"));
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
            .nth(matches.value_of("platform-id").unwrap().parse::<usize>().unwrap())
            .expect("Can not get the platform.");

    let device_id =
        matches.value_of("device-id").unwrap().parse::<usize>().unwrap();

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

    let mut result_sdf = 
        MonoImage::new(width / stride, height / stride);

    let mut gpu_sdf =
        memory::Buffer::<u8>::create(
            &context.opencl_context,
            memory::CL_MEM_READ_ONLY,
            result_sdf.width * result_sdf.height,
            std::ptr::null_mut()
        ).unwrap();

    let wait_for_sdf_generate =
        context.sdf_generate(
            device_id,
            &edge,
            &mut gpu_sdf,
            width,
            height,
            result_sdf.width,
            result_sdf.height,
            stride,
            search_radius,
            &[wait_for_edge_detect]
        );

    

    let wait_for_read_buffer =
        context.read_buffer(
            device_id, 
            &gpu_sdf, 
            &mut result_sdf.pixels, 
            &[wait_for_sdf_generate]);

    wait_for_read_buffer.wait().unwrap();

    result_sdf.save_png(&Path::new(
        matches
            .value_of("OUTPUT")
            .expect("Output path not given.")));
}

