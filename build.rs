fn main() {
    let target = std::env::var("TARGET").expect("TARGET was not set.");

    if target.contains("windows") && target.contains("msvc") {
        if target.contains("x86_64") {
            println!("cargo:rustc-link-search=C:/Program Files (x86)/IntelSWTools/system_studio_2020/OpenCL/sdk/lib/x64");
        } else if target.contains("i686") {
            println!("cargo:rustc-link-search=C:/Program Files (x86)/IntelSWTools/system_studio_2020/OpenCL/sdk/lib/x86");
        } else {
            panic!("Not supported!")
        }
    } else {
        panic!("Not supported!");
    }
}