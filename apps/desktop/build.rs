// Build script for ZipLock Desktop Application
// Handles platform-specific build configurations, including Windows icon embedding

use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");

    match target_os.as_str() {
        "windows" => configure_windows(),
        "macos" => configure_macos(),
        "linux" => configure_linux(),
        _ => println!("cargo:warning=Unknown target OS: {}", target_os),
    }
}

/// Configure Windows-specific build settings
fn configure_windows() {
    println!("cargo:warning=Configuring Windows build...");

    // Embed Windows icon resource
    embed_windows_icon();

    // Check if building for production
    let is_production =
        env::var("ZIPLOCK_PRODUCTION").is_ok() || env::var("CARGO_FEATURE_PRODUCTION").is_ok();

    if is_production {
        println!("cargo:warning=Building Windows application for production mode");
        println!(
            "cargo:warning=Console subsystem will be hidden via #[windows_subsystem] attribute"
        );
        println!("cargo:warning=Event logging enabled for production builds");
    } else {
        println!("cargo:warning=Building Windows application for development mode");
        println!("cargo:warning=Console logging available for debugging");
    }

    // Enable static linking of runtime libraries
    if env::var("RUSTFLAGS")
        .unwrap_or_default()
        .contains("crt-static")
    {
        println!("cargo:warning=Static CRT linking enabled");
    }
}

/// Configure macOS-specific build settings
fn configure_macos() {
    println!("cargo:warning=Configuring macOS build...");

    // Set up macOS-specific linker flags if needed
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
}

/// Configure Linux-specific build settings
fn configure_linux() {
    println!("cargo:warning=Configuring Linux build...");

    // Set up Linux-specific configurations
    // Icon embedding for Linux is handled differently (desktop files, etc.)
    println!("cargo:rerun-if-changed=resources/icons/");
}

/// Embed Windows icon resource in the executable
fn embed_windows_icon() {
    // Look for resource configuration file first
    let possible_rc_paths = [
        "resources/windows/ziplock.rc",
        "../../packaging/windows/resources/ziplock.rc",
    ];

    let mut rc_path = None;
    for path in &possible_rc_paths {
        let full_path = PathBuf::from(path);
        if full_path.exists() {
            rc_path = Some(full_path);
            println!("cargo:rerun-if-changed={}", path);
            break;
        }
    }

    if let Some(rc_file) = rc_path {
        println!(
            "cargo:warning=Found resource configuration: {}",
            rc_file.display()
        );
        if let Err(e) = embed_resource_file(&rc_file) {
            println!("cargo:warning=Resource file embedding failed: {}", e);
            fallback_icon_embedding();
        } else {
            println!("cargo:warning=Successfully embedded resources from .rc file");
        }
    } else {
        println!("cargo:warning=No .rc file found, trying individual icon embedding");
        fallback_icon_embedding();
    }
}

/// Embed Windows resource file using embed-resource crate
fn embed_resource_file(rc_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", rc_path.display());

    // Use embed-resource to compile the .rc file
    embed_resource::compile(rc_path, embed_resource::NONE);
    Ok(())
}

/// Fallback icon embedding when no .rc file is found
fn fallback_icon_embedding() {
    // Look for icon files in multiple possible locations
    let possible_icon_paths = [
        "../../packaging/windows/resources/ziplock.ico",
        "resources/icons/ziplock.ico",
        "../../assets/icons/ziplock.ico",
    ];

    let mut icon_path = None;
    for path in &possible_icon_paths {
        let full_path = PathBuf::from(path);
        if full_path.exists() {
            icon_path = Some(full_path);
            println!("cargo:warning=Found icon at: {}", path);
            println!("cargo:rerun-if-changed={}", path);
            break;
        }
    }

    match icon_path {
        Some(icon_file) => {
            // Try to embed the icon using embed-resource
            if let Err(e) = embed_icon_with_embed_resource(&icon_file) {
                println!("cargo:warning=embed-resource failed: {}", e);

                // Fallback to winres
                if let Err(e2) = embed_icon_with_winres(&icon_file) {
                    println!("cargo:warning=winres also failed: {}", e2);
                    println!("cargo:warning=Icon embedding failed, executable will use default Windows icon");
                } else {
                    println!("cargo:warning=Successfully embedded icon using winres");
                }
            } else {
                println!("cargo:warning=Successfully embedded icon using embed-resource");
            }
        }
        None => {
            println!("cargo:warning=No icon file found, creating basic resource file");
            create_basic_windows_resource();
        }
    }
}

/// Embed icon using embed-resource crate
fn embed_icon_with_embed_resource(icon_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary .rc file
    let out_dir = env::var("OUT_DIR")?;
    let rc_file = PathBuf::from(out_dir).join("ziplock.rc");

    let rc_content = format!(
        r#"
// ZipLock Windows Resource File
#include <windows.h>

// Version Information
1 VERSIONINFO
FILEVERSION 1,0,0,0
PRODUCTVERSION 1,0,0,0
FILEFLAGSMASK VS_FFI_FILEFLAGSMASK
FILEFLAGS 0
FILEOS VOS__WINDOWS32
FILETYPE VFT_APP
FILESUBTYPE VFT2_UNKNOWN
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904E4"
        BEGIN
            VALUE "CompanyName", "ZipLock Project"
            VALUE "FileDescription", "ZipLock Password Manager"
            VALUE "FileVersion", "1.0.0.0"
            VALUE "InternalName", "ziplock"
            VALUE "LegalCopyright", "© 2024 ZipLock Project"
            VALUE "OriginalFilename", "ziplock.exe"
            VALUE "ProductName", "ZipLock"
            VALUE "ProductVersion", "1.0.0.0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1252
    END
END

// Application Icon
1 ICON "{}"
"#,
        icon_path.display()
    );

    std::fs::write(&rc_file, rc_content)?;

    // Use embed-resource to compile the .rc file
    embed_resource::compile(&rc_file, embed_resource::NONE);

    println!("cargo:rerun-if-changed={}", rc_file.display());
    Ok(())
}

/// Embed icon using winres crate as fallback
fn embed_icon_with_winres(icon_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon_with_id(icon_path.to_str().unwrap(), "1");
        res.set("FileDescription", "ZipLock Password Manager");
        res.set("ProductName", "ZipLock");
        res.set("CompanyName", "ZipLock Project");
        res.set("LegalCopyright", "© 2024 ZipLock Project");
        res.set("FileVersion", "1.0.0.0");
        res.set("ProductVersion", "1.0.0.0");
        res.compile()?;
    }
    Ok(())
}

/// Create basic Windows resource without icon
fn create_basic_windows_resource() {
    #[cfg(target_os = "windows")]
    {
        if let Ok(mut res) = std::panic::catch_unwind(|| {
            let mut res = winres::WindowsResource::new();
            res.set("FileDescription", "ZipLock Password Manager");
            res.set("ProductName", "ZipLock");
            res.set("CompanyName", "ZipLock Project");
            res.set("LegalCopyright", "© 2024 ZipLock Project");
            res.set("FileVersion", "1.0.0.0");
            res.set("ProductVersion", "1.0.0.0");
            res
        }) {
            if let Err(e) = res.compile() {
                println!("cargo:warning=Failed to create basic resource: {}", e);
            }
        }
    }
}

/// Check if we're building for Windows
#[allow(dead_code)]
fn is_windows_target() -> bool {
    env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows"
}

/// Check if we're in a CI environment
#[allow(dead_code)]
fn is_ci_build() -> bool {
    env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok()
}
