//! Generates bindings from the `LeapC.h` of the Ultraleap software on this machine.
//!
//! The header is proprietary, so neither it nor the generated code is committed. Symbols are
//! resolved when the library is loaded at runtime, so no link step is involved.

use std::env;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
const DEFAULT_SDK_ROOT: &str = "/Applications/Ultraleap Hand Tracking.app/Contents/LeapSDK";
#[cfg(target_os = "windows")]
const DEFAULT_SDK_ROOT: &str = r"C:\Program Files\Ultraleap\LeapSDK";
#[cfg(target_os = "linux")]
const DEFAULT_SDK_ROOT: &str = "/usr/share/doc/ultraleap-hand-tracking-service";

fn find_header() -> Option<PathBuf> {
    let from_env = env::var_os("LEAPSDK_LIB_PATH").map(PathBuf::from);
    from_env
        .iter()
        .flat_map(|lib| lib.parent().map(|root| root.join("include/LeapC.h")))
        .chain(std::iter::once(
            PathBuf::from(DEFAULT_SDK_ROOT).join("include/LeapC.h"),
        ))
        .find(|path| path.is_file())
}

fn main() {
    println!("cargo:rerun-if-env-changed=LEAPSDK_LIB_PATH");
    println!("cargo:rerun-if-changed=build.rs");

    let header = find_header().unwrap_or_else(|| {
        panic!(
            "LeapC.h was not found. Install the Ultraleap Hand Tracking software, or set \
             LEAPSDK_LIB_PATH to the SDK lib directory. To build without LeapC (mock mode only), \
             use `--no-default-features`."
        )
    });
    println!("cargo:rerun-if-changed={}", header.display());

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .allowlist_function("Leap.*")
        .allowlist_type("_?(LEAP|eLeap).*")
        .default_enum_style(bindgen::EnumVariation::Consts)
        .prepend_enum_name(false)
        .layout_tests(true)
        .derive_default(false)
        .dynamic_library_name("LeapCLibrary")
        // A newer or older SDK may lack some symbols: those become errors on use, not on load.
        .dynamic_link_require_all(false)
        .generate()
        .expect("could not generate bindings from LeapC.h");

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    bindings
        .write_to_file(out.join("bindings.rs"))
        .expect("could not write the generated bindings");
}
