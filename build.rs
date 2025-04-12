fn main() {
    // This build script helps with dynamic linking for Bevy
    
    // Only rerun this build script if the build.rs file changes
    println!("cargo:rerun-if-changed=build.rs");
    
    // On Windows, ensure we use the dynamic CRT
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmt.lib");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmtd.lib");
    }
}
