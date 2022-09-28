# Installing dependencies
## Windows
### SteamVR
* Install SteamVR through Steam

## Linux
### Monado
* On Ubuntu: `sudo apt install libopenxr-loader1 libopenxr-dev libopenxr1-monado`
* On Arch: `yay -S monado openxr`

### SteamVR (currently not working)
* NOTE: The following currently only compiles the program. This does not work yet
* Install SteamVR through Steam
* On Ubuntu: `sudo apt install libopenxr-loader1 libopenxr-dev`

# Running
* For VR: `cargo run --release -- vr` 
* For desktop: `cargo run --release` 

# TODO
- [ ] Use multiview rendering
- [ ] Display a floating cube (on a new branch `cube`)
- [ ] Create a library abstracting away the platform-dependent parts of this
