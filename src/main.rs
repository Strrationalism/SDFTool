mod context;

use opencl3::*;

fn print_help() {
    println!("SDFTool");
    println!("by Strrationalism Studio 2021");
    println!();
    println!("Usage:");
    println!("    sdftool symbol <image> <output-image>");
    println!("    sdftool font <ttf/otf> <charset> <output-dir>");
    println!();
    println!("Usable OpenCL platforms:");

    let platforms = 
        platform::get_platforms()
            .expect("Can not get opencl platforms.");

    for platform in platforms {
        println!("    {}", platform.name().expect("Can not get platform."));


        let devices = 
            platform.get_devices(device::CL_DEVICE_TYPE_ALL)
                .expect("Can not get devices.");
                
        for device in devices {
            let device = device::Device::new(device);
            println!("        {}({})", 
                device.name().unwrap(), 
                device::device_type_text(
                    device.dev_type().unwrap()));
        }
    }
    
    println!();
}

fn main() {
    use std::fs::File;

    match std::env::args().nth(1) {
        Some(x) if x == "symbol" =>{
            let _output_file = std::env::args().nth(3).expect("Output file not given.");
            if std::env::args().nth(4) != None {
                panic!("Too much args given.")
            }
            let input_png = 
                png::Decoder::new(
                    File::open(
                        std::env::args()
                            .nth(2)
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

            let _image_bytes = &buf[..frame_info.buffer_size()];

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
            
        }
        Some(x) if x == "font" => panic!("Not impl!!!"),
        _ => print_help()
    }
}
