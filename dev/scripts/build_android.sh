#!/bin/bash

# Часть проекта MoonWalk с открытым исходным кодом.
# Лицензия EPL 2.0, подробнее в файле LICENSE. UpdateDeveloper, 2025

set -e

TARGET_ARM64="true"
TARGET_ARM32="false"
TARGET_X86="false"

for arg in "$@"
do
    case $arg in
        --all)
            TARGET_ARM64="true"
            TARGET_ARM32="true"
            TARGET_X86="true"
            ;;
        --arm32)
            TARGET_ARM32="true"
            ;;
        --chromeos)
            TARGET_X86="true"
            ;;
        *)
            # Игнорируем неизвестные флаги
            ;;
    esac
done

NDK_TARGETS=""

if [ "$TARGET_ARM64" = "true" ]; then
    NDK_TARGETS="$NDK_TARGETS -t aarch64-linux-android"
    echo "Enabled: ARM64 (v8a) -> Modern Phones"
fi

if [ "$TARGET_ARM32" = "true" ]; then
    NDK_TARGETS="$NDK_TARGETS -t armv7-linux-androideabi"
    echo "Enabled: ARMv7 (Legacy) -> Old Phones"
fi

if [ "$TARGET_X86" = "true" ]; then
    NDK_TARGETS="$NDK_TARGETS -t x86_64-linux-android"
    echo "✅ Enabled: x86_64 -> ChromeOS / Emulator"
fi

echo "Building Android libraries..."

# Запускаем сборку
# shellcheck disable=SC2086
cargo ndk $NDK_TARGETS --platform 30 build --release -p example

echo "Done! Artifacts located in:"
if [ "$TARGET_ARM64" = "true" ]; then echo "   - target/aarch64-linux-android/release/libexample.so"; fi
if [ "$TARGET_ARM32" = "true" ]; then echo "   - target/armv7-linux-androideabi/release/libexample.so"; fi
if [ "$TARGET_X86" = "true" ]; then echo "   - target/x86_64-linux-android/release/libexample.so"; fi