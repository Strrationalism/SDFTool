#![feature(path_try_exists)]
use std::{fs::*, process::Command};

fn setup_opencl_sdk() {
    if !try_exists("./OpenCL-SDK/build").unwrap() {
        create_dir("./OpenCL-SDK/build").unwrap();
    }

    if !try_exists("./OpenCL-SDK/build/build.ninja").unwrap() {
        let status = 
            Command::new("cmake")
                .arg("..")
                .arg("-G")
                .arg("Ninja")
                .current_dir("./OpenCL-SDK/build")
                .status()
                .unwrap();

        assert!(status.success());
    }

    if !try_exists("./OpenCL-SDK/build/external/OpenCL-ICD-Loader").unwrap() {
        let status =
            Command::new("ninja")
                .current_dir("./OpenCL-SDK/build")
                .status()
                .unwrap();
        
        assert!(status.success());
    }
}

fn main() {
    setup_opencl_sdk();
    println!("cargo:rustc-link-search=./OpenCL-SDK/build/external/OpenCL-ICD-Loader/");
}