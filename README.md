# SDFTool
Signed distance field font and image command line tool based on OpenCL.

## Build

### Windows

Run `cargo build --release` in Visual Studio developer x64 command prompt.

### macOS/Linux

Run `cargo build --release` in terminal.

## Install

### Windows
Run `cargo install --git https://github.com/Strrationalism/SDFTool.git sdftool` in Visual Studio developer x64 command prompt.

### macOS/Linux
Run `cargo install --git https://github.com/Strrationalism/SDFTool.git sdftool` in terminal.

## Usage

### Show OpenCL devices
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

### Create SDF font atlas from TTF/OTF
```
sdftool-font
Create the sdf font

USAGE:
    sdftool font [FLAGS] [OPTIONS] <INPUT> <OUTDIR>

FLAGS:
    -h, --help          Prints help information
        --no-ascii      Do not generate ascii charset
        --schinese-1    Generate common standard chinese table 1
        --schinese-2    Generate common standard chinese table 2
        --schinese-3    Generate common standard chinese table 3
    -V, --version       Prints version information

OPTIONS:
        --margin-x <margin-x>              Margin X on every sdf character in pixels [default: 0]
        --margin-y <margin-y>              Margin Y on every sdf character in pixels [default: 0]
        --origin-scale <origin-scale>      Basic font scale before downsample [default: 384]
        --padding-x <padding-x>            Padding X on every basic character in pixels [default: 24]
        --padding-y <padding-y>            Padding Y on every basic character in pixels [default: 24]
        --page-height <page-height>        Single page height in pixels [default: 1024]
        --page-width <page-width>          Single page width in pixels [default: 1024]
    -r, --search-radius <search-radius>    Set the radius for edge searching [default: 24]
    -s, --stride <stride>                  Set the downsample stride size (1 will not downsample) [default: 8]

ARGS:
    <INPUT>     Input ttf/otf file
    <OUTDIR>    Output path
    ```
