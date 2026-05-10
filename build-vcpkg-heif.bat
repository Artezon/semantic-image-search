@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
set VCPKG_ROOT=%CD%\src-tauri\target\vcpkg
cargo install cargo-vcpkg
cargo vcpkg -v build --manifest-path src-tauri/Cargo.toml
