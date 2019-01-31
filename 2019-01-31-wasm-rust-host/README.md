# Wasm Rust host example

This repository contains an example on how to execute Wasm modules from Rust host.

* [wasmi](https://github.com/paritytech/wasmi) is used as an interpreter.


## Run

```sh
# Install wasm toolchain 
rustup target add wasm32-unknown-unknown

# Build Wasm module
make build

# Execute Wasm module
make run
```


## Presentation

Find presentation file in `./presentation.pdf`
