# Rust OpenGL OpenXR example
Minimal example code to display a triangle on both desktop and in OpenXR using OpenGL in Rust. Also provides `session_create_info()` to perform appropriate platform-specific magic to get `SessionCreateInfo`. One should use caution when using this library, as it was really just hacked together using guesswork and `std::mem::transmute()`. PRs welcome!

# Installing dependencies
## Windows
### SteamVR
* Install SteamVR through Steam

## Linux
### Monado
* On Ubuntu: `sudo apt install libopenxr-loader1 libopenxr-dev libopenxr1-monado`
* On Arch: `yay -S monado openxr`

### SteamVR (currently not working on Linux)
* NOTE: The following currently only compiles the program. This does not work yet
* Install SteamVR through Steam
* On Ubuntu: `sudo apt install libopenxr-loader1 libopenxr-dev`

# Running
`cargo run --release --example triangle`
Run with the `--vr` flag to use OpenXR

# TODO
- [x] Create a library abstracting away the platform-dependent parts of this
- [ ] Use multiview rendering
- [ ] Display a floating cube (on a new branch `cube`)

