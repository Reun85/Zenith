fn main() {
    // Detect which profile is being used and define the corresponding flag
    let profile = std::env::var("PROFILE").unwrap();

    if profile == "debug" {
        println!("cargo:rustc-cfg=build_type=\"debug\"");
        println!("cargo:rustc-cfg=debug");
    } else if profile == "release" {
        println!("cargo:rustc-cfg=build_type=\"release\"");
        println!("cargo:rustc-cfg=debug");
    } else if profile == "dist" {
        println!("cargo:rustc-cfg=build_type=\"dist\"");
    }
}
