#![feature(path_try_exists)]
use std::{fs::*, process::Command};

fn setup_opencl_sdk(target: &str) {
    if !try_exists("./OpenCL-SDK/LICENSE").unwrap() {
        let status =
            Command::new("git")
                .arg("clone")
                .arg("https://github.com/KhronosGroup/OpenCL-SDK.git")
                .status()
                .unwrap();
                
        assert!(status.success());
    }

    let opencl_submodules =
        try_exists("./OpenCL-SDK/external/OpenCL-ICD-Loader/LICENSE").unwrap()
        && try_exists("./OpenCL-SDK/external/OpenCL-Headers/LICENSE").unwrap()
        && try_exists("./OpenCL-SDK/external/OpenCL-CLHPP/LICENSE").unwrap();

    if !opencl_submodules {
        let status =
            Command::new("git")
                .arg("submodule")
                .arg("update")
                .arg("--init")
                .current_dir("./OpenCL-SDK")
                .status()
                .unwrap();

        assert!(status.success());
    }

    if !try_exists("./OpenCL-SDK/build").unwrap() {
        create_dir("./OpenCL-SDK/build").unwrap();
    }

    if !try_exists("./OpenCL-SDK/build/CMakeCache.txt").unwrap() {
        let g =
            if target.contains("pc-windows-msvc") {
                "NMake Makefiles"
            } else {
                "Unix Makefiles"
            };

        let status = 
            Command::new("cmake")
                .arg("..")
                .arg("-G")
                .arg(g)
                .current_dir("./OpenCL-SDK/build")
                .status()
                .unwrap();

        assert!(status.success());
    }

    if !try_exists("./OpenCL-SDK/build/external/OpenCL-ICD-Loader").unwrap() {
        let status =
            if target.contains("pc-windows-msvc") {
                Command::new("nmake")
                    .current_dir("./OpenCL-SDK/build")
                    .status()
                    .unwrap()
            } else {
                Command::new("make")
                    .current_dir("./OpenCL-SDK/build")
                    .status()
                    .unwrap()
            };
        
        assert!(status.success());
    }
}

fn main() {
    let target = std::env::var("TARGET").expect("TARGET was not set.");

    setup_opencl_sdk(&target);

    if target.contains("pc-windows-msvc") {
        println!("cargo:rustc-link-search=./OpenCL-SDK/build/external/OpenCL-ICD-Loader/");
    } else {
        panic!("Not supported!");
    }
}