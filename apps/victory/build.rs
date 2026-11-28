fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os == "windows" && target_env == "msvc" {
        // Export the DirectX 12 Agility SDK version and relative path as data
        // symbols on `victory.exe` so Windows 10 can load an app-local
        // `D3D12\D3D12Core.dll` with Shader Model 6.6 support.
        println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKVersion,DATA");
        println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKPath,DATA");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
