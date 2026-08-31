// LUMIO_ABI_HASH / LUMIO_BUILD_ID are embedded into the root API table at compile time
// via option_env!. Cargo does not track option_env! reads for change detection by default,
// so a dev-build that only changes these env values would silently reuse a stale binary
// (observed MS-00002: new ABI hash in build-info.json, old hash embedded in the DLL).
// Declaring the env dependencies here forces a recompile whenever the identity changes.
fn main() {
    println!("cargo:rerun-if-env-changed=LUMIO_ABI_HASH");
    println!("cargo:rerun-if-env-changed=LUMIO_BUILD_ID");
}
