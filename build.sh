#!/bin/bash

# Compile and run with Cargo
cargo run --release

# Run ppm2png with img.ppm and img.png as arguments
./ppm2png img.ppm img.png

# Remove img.ppm
rm -rf img.ppm
