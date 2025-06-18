fn main() {
    // Skip all Windows resource compilation
    #[cfg(target_os = "windows")]
    {
        // Set environment to skip resource embedding
        std::env::set_var("TAURI_SKIP_DEVSERVER_CHECK", "true");
        std::env::set_var("TAURI_SKIP_BUNDLE_CHECK", "true");
        
        // Don't generate any Windows resources at all
        println!("cargo:rustc-cfg=skip_windows_resource");
        return;
    }
    
    #[cfg(not(target_os = "windows"))]
    tauri_build::build()
}
