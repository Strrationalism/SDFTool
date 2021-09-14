#![feature(path_try_exists)]
use std::{fs::*, process::Command};

fn setup_opencl_sdk(target: &str) {
    if !try_exists("./OpenCL-SDK/build").unwrap() {
        create_dir("./OpenCL-SDK/build").unwrap();
    }

    if !try_exists("./OpenCL-SDK/build/Makefile").unwrap() {
        let build_tool = 
            if target.contains("msvc") {
                "NMake Makefiles" 
            } else { 
                "Unix Makefiles" 
            };

        let status = 
            Command::new("cmake")
                .arg("..")
                .arg("-G")
                .arg(build_tool)
                .current_dir("./OpenCL-SDK/build")
                .status()
                .expect("Cargo build command must run in Visual Studio prompt or CMake not installed.");

        assert!(status.success());
    }


    let build_tool = 
        if target.contains("msvc") {
            "nmake"
        } else {
            "make"
        };

    let status = 
        Command::new(build_tool)
            .current_dir("./OpenCL-SDK/build")
            .status()
            .expect("Cargo build command must run in Visual Studio prompt or NMake/Make not installed.");

    assert!(status.success());
    
}

fn main() {
    let target = std::env::var("TARGET").unwrap();
    setup_opencl_sdk(&target);
    println!("cargo:rustc-link-search=native=./OpenCL-SDK/build/external/OpenCL-ICD-Loader/");
}