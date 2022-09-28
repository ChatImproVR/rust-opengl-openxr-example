# Installing dependencies
## Linux
### SteamVR
* Install SteamVR through Steam
* On Ubuntu: `sudo apt install libopenxr-loader1 libopenxr-dev`

### Monado
* On Ubuntu: `sudo apt install libopenxr-loader1 libopenxr-dev libopenxr1-monado`
* On Arch: `yay -S monado openxr`

## Windows
### SteamVR
* Install SteamVR through Steam

# Running
* For VR: `cargo run --release -- vr` 
* For desktop: `cargo run --release` 
