#![feature(path_try_exists)]
use std::{fs::*, process::Command};

fn setup_opencl_sdk(target: &str) {
    if target.contains("apple-darwin") { return; }

    if !try_exists("./OpenCL-SDK/build").unwrap() {
        create_dir("./OpenCL-SDK/build").unwrap();
    }

    let (make_tool_cmake_id, make_tool, shared_lib) =
        if target.contains("pc-windows-msvc") {
            ("NMake Makefiles", "nmake", "true")
        } else {
            ("Unix Makefiles", "make", "false")
        };

    if !try_exists("./OpenCL-SDK/build/Makefile").unwrap() {
        let status = 
            Command::new("cmake")
                .arg("..")
                .arg("-G")
                .arg(make_tool_cmake_id)
                .arg(format!("-DBUILD_SHARED_LIBS={}", shared_lib))
                .current_dir("./OpenCL-SDK/build")
                .status()
                .expect("Cargo build command must run in Visual Studio prompt or CMake not installed.");

        assert!(status.success());
    }


    let status = 
        Command::new(make_tool)
            .current_dir("./OpenCL-SDK/build")
            .status()
            .expect("Cargo build command must run in Visual Studio prompt or NMake not installed.");

    assert!(status.success());
    
    if target.contains("pc-windows-msvc") {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        std::fs::copy(
            "./OpenCL-SDK/build/external/OpenCL-ICD-Loader/OpenCL.dll", 
            out_dir + "/../../../OpenCL.dll").unwrap();
    }
}

fn main() {
    let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let target = std::env::var("TARGET").unwrap();
    
    setup_opencl_sdk(&target);
    
    println!("cargo:rustc-link-search={}/OpenCL-SDK/build/external/OpenCL-ICD-Loader/", project_dir);
}