#!/bin/bash

# Часть проекта MoonWalk с открытым исходным кодом.
# Лицензия EPL 2.0, подробнее в файле LICENSE. UpdateDeveloper, 2025

set -e

if [[ "$1" == "--all" ]]; then
    echo "Building for ARM64 (v8a) + ARMv7..."
    TARGETS="-t aarch64-linux-android -t armv7-linux-androideabi"
else
    echo "Building for ARM64 (v8a) only..."
    TARGETS="-t aarch64-linux-android"
fi

cargo ndk $TARGETS --platform 30 build --release -p example

echo "Done!"
echo "Artifacts located in"
echo "   - target/aarch64-linux-android/release/libexample.so"
if [[ "$1" == "--all" ]]; then
    echo "   - target/armv7-linux-androideabi/release/libexample.so"
fi