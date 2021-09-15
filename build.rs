#![feature(path_try_exists)]
use std::{fs::*, process::Command};

fn setup_opencl_sdk_for_msvc() {
    if !try_exists("./OpenCL-SDK/build").unwrap() {
        create_dir("./OpenCL-SDK/build").unwrap();
    }

    if !try_exists("./OpenCL-SDK/build/Makefile").unwrap() {
        let build_tool = "NMake Makefiles" ;
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


    let build_tool = "nmake";
    let status = 
        Command::new(build_tool)
            .current_dir("./OpenCL-SDK/build")
            .status()
            .expect("Cargo build command must run in Visual Studio prompt or NMake not installed.");

    assert!(status.success());
    
}

fn main() {
    let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let target = std::env::var("TARGET").unwrap();
    if target.contains("msvc") { setup_opencl_sdk_for_msvc(); }
    println!("cargo:rustc-link-search={}/OpenCL-SDK/build/external/OpenCL-ICD-Loader/", project_dir);
}