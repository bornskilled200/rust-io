# Simple PMS5003 web server

## Install 
https://github.com/RigacciOrg/AirPi


## Building
Cross compiling from windows to raspberry pi (only 3 is tested)
```
rustup target add armv7-unknown-linux-gnueabihf
# https://gnutoolchains.com/raspberry/

RUSTFLAGS="-C linker=arm-linux-gnueabihf-gcc" cargo build --release --target armv7-unknown-linux-gnueabihf
```

You can also just run this on raspberry pi
```
cargo build --release
./target/release/main
```