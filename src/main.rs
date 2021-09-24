mod context;
mod charset;
mod basic_font_generator;
mod mono_image;
mod atlas_generator;
mod program_cpu;

use atlas_generator::AtlasGenerator;
use basic_font_generator::*;
use charset::CharsetRequest;
use clap::{App, Arg, ArgMatches, SubCommand};
use context::Context;
use opencl3::*;
use opencl3::platform::Platform;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use progress_bar::progress_bar::ProgressBar;
use progress_bar::color::{Color, Style};

use crate::mono_image::MonoImage;

const DEVICE_TYPE : types::cl_bitfield = 
    device::CL_DEVICE_TYPE_ALL;

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

    let cpu_arg =
        Arg::with_name("cpu")
            .help("Do not use OpenCL.")
            .long("cpu");

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
                .arg(stride_arg.clone())
                .arg(cpu_arg.clone()))
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
                .arg(cpu_arg)
                .arg(search_radius_arg.default_value("24"))
                .arg(stride_arg.default_value("8"))
                .arg(Arg::with_name("no-ascii")
                    .long("no-ascii")
                    .multiple(false)
                    .help("Do not generate ascii charset"))
                .arg(Arg::with_name("schinese-punc")
                    .long("schinese-punc")
                    .multiple(false)
                    .help("Generate punctuations for schinese"))
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
                    .default_value("384")
                    .help("Basic font scale before downsample"))
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
                    .help("Margin X on every sdf character in pixels"))
                .arg(Arg::with_name("margin-y")
                    .long("margin-y")
                    .default_value("0")
                    .help("Margin Y on every sdf character in pixels"))
                .arg(Arg::with_name("padding-x")
                    .long("padding-x")
                    .default_value("24")
                    .help("Padding X on every basic character in pixels"))
                .arg(Arg::with_name("padding-y")
                    .long("padding-y")
                    .default_value("24")
                    .help("Padding Y on every basic character in pixels"))
                .arg(Arg::with_name("charset")
                    .long("charset")
                    .short("c")
                    .multiple(true)
                    .required(false)
                    .takes_value(true)
                    .help("Additional charset to generate")));

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
    let basic_gen = Arc::new(BasicFontGenerator::from(args));
    let charset = CharsetRequest::from_args(args).get_charset();
    
    let progress_bar = Arc::new(Mutex::new(ProgressBar::new(charset.len())));

    {
        progress_bar
            .lock()
            .unwrap()
            .set_action("Rendering", Color::LightCyan, Style::Bold);
    }

    let task = Arc::new(Mutex::new(None));

    let run = Arc::new(AtomicBool::new(true));

    let cvar = Arc::new(Condvar::new());

    let (stride, search_radius) = 
        get_stride_and_search_radius(&args);

    let atlas_generator = Arc::new(Mutex::new(AtlasGenerator::new(
        args.value_of("page-width").unwrap().parse().unwrap(),
        args.value_of("page-height").unwrap().parse().unwrap(),
        args.value_of("OUTDIR").unwrap().parse().unwrap(),
        args.value_of("margin-x").unwrap().parse().unwrap(),
        args.value_of("margin-y").unwrap().parse().unwrap()
    )));

    let enable_opencl_workers = !args.is_present("cpu");
    let mut cpu_workers_num = num_cpus::get() as i32;

    let create_opencl_workers = |platform: Platform| {
        let devices =
            platform
                .get_devices(DEVICE_TYPE)
                .unwrap()
                .into_iter();
        
        let mut threads = Vec::new();
        for device_id in devices {
            {
                let device = device::Device::new(device_id);
                if device.dev_type().unwrap() == device::CL_DEVICE_TYPE_CPU {
                    cpu_workers_num = 0;
                }
            }
            let progress_bar = progress_bar.clone();
            let basic_gen = basic_gen.clone();
            let run = run.clone();
            let cvar = cvar.clone();
            let task = task.clone();
            let device_ptr = device_id as usize;
            let atlas_generator = atlas_generator.clone();
            
            threads.push(thread::spawn(move ||{
                let context = Context::new(
                    device_ptr as *mut core::ffi::c_void);

                {
                    progress_bar.lock().unwrap().print_info(
                        "Info", 
                        &format!(
                            "OpenCL Worker: {}", 
                            context.device_name),
                        Color::Green,
                        Style::Bold
                    );
                }
                

                let mut generate_sdf_task: Option<char> = None;
                let mut generate_basic_task: Option<char> = None;

                let mut basic_gen_buf = MonoImage::new(0, 0);
                let mut basic_load_to_ocl_buf = MonoImage::new(0, 0);
                let mut result_buf = MonoImage::new(0, 0);

                let mut ocl_buf_load = None;
                let mut ocl_buf_edge = None;
                let mut ocl_buf_result = None;
                let mut ocl_buf_len: usize = 0;

                let mut str_buf = String::new();
                
                loop {

                    {   // Test break condition
                        let run =
                            run.load(Ordering::Relaxed)
                            || generate_basic_task.is_some()
                            || generate_sdf_task.is_some();

                        if !run {
                            break;
                        }
                    }

                    // Create SDF Task
                    let sdf_task =  
                        if let Some(ch) = generate_sdf_task {
                            generate_sdf_task = None;

                            let width = basic_load_to_ocl_buf.width;
                            let height = basic_load_to_ocl_buf.height;

                            let buffer_size = width * height;

                            if ocl_buf_load.is_none() || ocl_buf_len < buffer_size {
                                ocl_buf_len = buffer_size;
                                ocl_buf_load = 
                                    Some(memory::Buffer::<u8>::create(
                                        &context.opencl_context,
                                        memory::CL_MEM_WRITE_ONLY,
                                        ocl_buf_len,
                                        std::ptr::null_mut()
                                    ).unwrap());

                                ocl_buf_edge = 
                                    Some(memory::Buffer::<u8>::create(
                                        &context.opencl_context,
                                        0,
                                        ocl_buf_len,
                                        std::ptr::null_mut()
                                    ).unwrap());

                                ocl_buf_result = 
                                    Some(memory::Buffer::<u8>::create(
                                        &context.opencl_context,
                                        memory::CL_MEM_READ_ONLY,
                                        ocl_buf_len / stride / stride,
                                        std::ptr::null_mut()
                                    ).unwrap());
                            }

                            let wait_load = 
                                context.write_buffer_to_cl(
                                    &basic_load_to_ocl_buf.pixels, 
                                    ocl_buf_load.as_mut().unwrap(), 
                                    &[]);

                            let wait_edge_detect =
                                context.edge_detect(
                                    ocl_buf_load.as_ref().unwrap(),
                                    ocl_buf_edge.as_mut().unwrap(),
                                    width,
                                    height,
                                    &[wait_load]
                                );

                            let sdf_width = width / stride;
                            let sdf_height = height / stride;

                            let wait_sdf_generate =
                                context.sdf_generate(
                                    ocl_buf_edge.as_ref().unwrap(),
                                    ocl_buf_result.as_mut().unwrap(),
                                    width,
                                    height,
                                    sdf_width,
                                    sdf_height,
                                    stride,
                                    search_radius,
                                    &[wait_edge_detect]
                                );

                            Some((ch, wait_sdf_generate, sdf_width, sdf_height))
                        } else {
                            None
                        };
                    

                    {   // Generate Basic Task
                        if let Some(task) = generate_basic_task {
                            generate_basic_task = None;
                            str_buf.clear();
                            str_buf.push(task);
                            if basic_gen.generate(&str_buf, &mut basic_gen_buf) {
                                generate_sdf_task = Some(task);
                            } else {
                                progress_bar
                                    .lock()
                                    .unwrap()
                                    .print_info(
                                        "Warning", 
                                        &format!("Can not render {}", task), 
                                        Color::Yellow, 
                                        Style::Bold);
                            }
                        }
                    }

                    
                    {   // Get next task
                        let mut task = task.lock().unwrap();
                        if let Some(ch) = *task {
                            generate_basic_task = Some(ch);
                            *task = None;
                        }
                        cvar.notify_one();
                    }

                    {   // Send Result
                        if let Some((ch, event, sdf_width, sdf_height)) = sdf_task {
                            result_buf.resize(sdf_width, sdf_height);
                            context.read_buffer_to_cpu(
                                ocl_buf_result.as_ref().unwrap(), 
                                &mut result_buf.pixels, 
                                &[event]).wait().unwrap();
                            
                            atlas_generator.lock().unwrap().push(ch, &result_buf);
                        }
                    }

                    std::mem::swap(&mut basic_gen_buf, &mut basic_load_to_ocl_buf);
                }
            }));
        }

        threads
    };

    let opencl_workers: Vec<_> =
        if enable_opencl_workers {
            match platform::get_platforms() {
                Err(_) => {
                    progress_bar
                        .lock()
                        .unwrap()
                        .print_info(
                            "Warning", 
                            "Can not get OpenCL platforms, rendering on CPU...",
                            Color::Yellow, 
                            Style::Bold);
                    vec![]
                },
                Ok(x) => 
                    x
                        .into_iter()
                        .flat_map(create_opencl_workers)
                        .collect()
            }
                
        } else {
            vec![]
        };

    cpu_workers_num = cpu_workers_num - (opencl_workers.len() as i32);
    cpu_workers_num = cpu_workers_num.clamp(0, i32::MAX);

    let cpu_workers : Vec<_> = 
        (0 .. cpu_workers_num)
            .into_iter()
            .map(|_| {
                let progress_bar = progress_bar.clone();
                let basic_gen = basic_gen.clone();
                let run = run.clone();
                let cvar = cvar.clone();
                let task = task.clone();
                let atlas_generator = atlas_generator.clone();
                
                thread::spawn(move ||{
                    let mut str_buf = String::new();
                    let mut basic_gen_buf = MonoImage::new(0, 0);
                    let mut edge_buf = MonoImage::new(0, 0);
                    let mut sdf_buf = MonoImage::new(0, 0);

                    loop {
                        {   // Test break condition
                            if !run.load(Ordering::Relaxed) {
                                break;
                            }
                        }

                        let mut generate_basic_task = None;

                        {   // Get Next Task
                            let mut task = task.lock().unwrap();
                            if let Some(ch) = *task {
                                generate_basic_task = Some(ch);
                                *task = None;
                            }
                            cvar.notify_one();
                        }

                        if let Some(ch) = generate_basic_task{   
                            // Do task
                            str_buf.clear();
                            str_buf.push(ch);
                            if basic_gen.generate(&str_buf, &mut basic_gen_buf) {
                                edge_buf.resize(basic_gen_buf.width, basic_gen_buf.height);
                                basic_gen_buf.edge_detect(&mut edge_buf);
                                sdf_buf.resize(
                                    edge_buf.width / stride, 
                                    edge_buf.height / stride);
                                edge_buf.edge_generate_sdf(&mut sdf_buf, stride, search_radius);
                                atlas_generator.lock().unwrap().push(ch, &sdf_buf);
                            } else {
                                progress_bar
                                    .lock()
                                    .unwrap()
                                    .print_info(
                                        "Warning", 
                                        &format!("Can not render {}", ch), 
                                        Color::Yellow, 
                                        Style::Bold);
                            }
                        }
                    }
                })
            })
            .collect();


    progress_bar
        .lock()
        .unwrap()
        .print_info(
            "Info", 
            &format!("OpenCL Workers: {}, CPU Workers: {}",
                opencl_workers.len(), 
                cpu_workers.len()),
            Color::Green, 
            Style::Bold);
    
    for i in charset {
        let mut task = task.lock().unwrap();
        while task.is_some() {
            task = cvar.wait(task).unwrap();
        }

        *task = Some(i);

        drop(task);

        progress_bar.lock().unwrap().inc();
    }

    loop {
        let mut task = task.lock().unwrap();
        if task.is_some() {
            task = cvar.wait(task).unwrap();
            
            if *task == None {
                break;
            }
        } else {
            break;
        }
    }

    run.store(false, Ordering::Release);

    for i in opencl_workers {
        i.join().unwrap();
    }

    for i in cpu_workers {
        i.join().unwrap();
    }

    let atlas_generator = atlas_generator.lock().unwrap();
    atlas_generator.save_current_page();
    atlas_generator.save_metadata();

    progress_bar
        .lock()
        .unwrap()
        .print_final_info(
            "Complete", 
            "", 
            Color::Green, 
            Style::Bold);
}

fn show_cl_devices() {
    let platforms = 
        match platform::get_platforms() {
            Ok(x) if x.len() == 0 => {
                println!("Can not get OpenCL platforms.");
                return;
            },
            Err(_) => {
                println!("Can not get OpenCL platforms.");
                return;
            },
            Ok(x) => x
        };

    let mut platform_id = 0;

    for platform in platforms {
        println!(
            "Platform {}: {}", 
            platform_id, 
            platform.name().expect("Can not get platform."));

        platform_id += 1;

        let devices = 
            platform.get_devices(DEVICE_TYPE)
                .expect("Can not get devices.");
                
        let mut device_id = 0;

        for device in devices {
            let device = device::Device::new(device);

            println!("    {}. {} ({})", 
                device_id,
                device.name().unwrap(), 
                device::device_type_text(
                    device.dev_type().unwrap()));

            device_id += 1;
        }
    }
}

fn get_stride_and_search_radius(matches: &clap::ArgMatches) -> (usize, usize) {
    let stride = 
        matches.value_of("stride").unwrap().parse().unwrap();

    let search_radius = 
        matches.value_of("search-radius").unwrap().parse().unwrap();

    if stride <= 0 {
        panic!("Stride must greate or equals 1.")
    }

    (stride, search_radius)
}

fn symbol_cpu(matches: &clap::ArgMatches) {
    println!("Info: Rendering on CPU.");
    
    let image =
        MonoImage::load_from_file(
            matches.value_of("INPUT").expect("No input png given."));

    let mut edge = MonoImage::new(image.width, image.height);
    image.edge_detect(&mut edge);

    let (stride, search_radius) = 
        get_stride_and_search_radius(&matches);

    let mut result_sdf = 
        MonoImage::new(edge.width / stride, edge.height / stride);

    edge.edge_generate_sdf(&mut result_sdf, stride, search_radius);

    result_sdf.save_png(&Path::new(
        matches
            .value_of("OUTPUT")
            .expect("Output path not given.")));
}

fn symbol(matches: &clap::ArgMatches) {
    if matches.is_present("cpu") {
        return symbol_cpu(matches);
    }

    let platform =
        match platform::get_platforms() {
            Ok(x) if x.len() == 0 => return symbol_cpu(matches),
            Err(_) => return symbol_cpu(matches),
            Ok(x) => 
                x
                    .into_iter()
                    .nth(matches.value_of("platform-id").unwrap().parse::<usize>().unwrap())
                    .expect("Can not get the platform.")
        };
            

    let device_id =
        matches.value_of("device-id").unwrap().parse::<usize>().unwrap();

    let devices =
        platform
        .get_devices(DEVICE_TYPE)
        .unwrap();

    if devices.len() == 0 {
        return symbol_cpu(matches);
    }

    let context = 
        crate::context::Context::new(
            devices
                .into_iter()
                .nth(device_id)
                .unwrap());

    let (image, width, height) =
        context.load_png(matches.value_of("INPUT").expect("No input png given."));

    let mut edge = 
        memory::Buffer::<u8>::create(
            &context.opencl_context,
            0,
            width * height,
            std::ptr::null_mut()
        ).unwrap();

    let wait_for_edge_detect =
        context.edge_detect(
            &image, 
            &mut edge, 
            width, 
            height,
            &[]);

    let (stride, search_radius) = 
        get_stride_and_search_radius(&matches);

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
        context.read_buffer_to_cpu(
            &gpu_sdf, 
            &mut result_sdf.pixels, 
            &[wait_for_sdf_generate]);

    wait_for_read_buffer.wait().unwrap();

    result_sdf.save_png(&Path::new(
        matches
            .value_of("OUTPUT")
            .expect("Output path not given.")));
}

