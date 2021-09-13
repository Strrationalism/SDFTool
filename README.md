# SDFTool
Signed distance field font and image command line tool.

## Build

### Windows

1. Install the [OpenCL SDK from Intel](https://software.intel.com/content/www/cn/zh/develop/tools/opencl-sdk.html?wapkw=OpenCL%20SDK) to default path.
2. `cargo build --release`

## Usage

### Show OpenCL Devices
```
sdftool cl-devices
```

### Create SDF image for PNG symbol
```
USAGE:
    sdftool symbol [OPTIONS] <INPUT> <OUTPUT>

OPTIONS:
        --device-id <device-id>         Select the device to use [default: 0]
        --platform-id <platform-id>     Select the platform to use [default: 0]
    -r, --search-radius <search-radius> Set the radius for edge searching [default: 128]
    -s, --stride <stride>               Set the downsample stride size (1 will not downsample) [default: 4]

ARGS:
    <INPUT>     Symbol image in PNG format
    <OUTPUT>    Output path for SDF image in PNG format
```
