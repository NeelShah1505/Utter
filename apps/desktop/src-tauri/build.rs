// build.rs — required by Tauri to embed resources and generate glue code.
fn main() {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        use std::env;

        let out_dir = env::var("OUT_DIR").unwrap();
        
        let status = Command::new("swiftc")
            .args(&[
                "-c", "-O",
                "-target", "arm64-apple-macos13.0",
                "helpers/utter_transcribe.swift",
                "-o", &format!("{}/utter_transcribe.o", out_dir)
            ])
            .status()
            .expect("Failed to compile Swift helper");
            
        assert!(status.success(), "Swift compilation failed");
        
        let status = Command::new("ar")
            .args(&[
                "crs",
                &format!("{}/libutter_transcribe.a", out_dir),
                &format!("{}/utter_transcribe.o", out_dir)
            ])
            .status()
            .expect("Failed to archive Swift helper");
            
        assert!(status.success(), "Swift archiving failed");

        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static=utter_transcribe");
        println!("cargo:rustc-link-search=/usr/lib/swift");
        println!("cargo:rustc-link-lib=swiftCore");
        println!("cargo:rustc-link-lib=swiftFoundation");
        println!("cargo:rustc-link-lib=framework=Speech");
        println!("cargo:rustc-link-lib=framework=AVFoundation");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rerun-if-changed=helpers/utter_transcribe.swift");
    }
    
    tauri_build::build()
}
