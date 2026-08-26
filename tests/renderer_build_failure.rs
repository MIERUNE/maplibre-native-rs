//! A renderer construction failure in the native graphics stack must surface
//! as `RendererBuildError`, not `std::terminate`.
//!
//! This is its own integration-test binary because the EGL environment is
//! poisoned process-wide before the first native call; sharing a process with
//! working-renderer tests would break them.

// The failure injection targets glvnd's EGL vendor discovery, which only
// exists on the Linux EGL (opengl) backend.
#![cfg(all(target_os = "linux", feature = "opengl"))]

use std::num::NonZeroU32;

use maplibre_native::ImageRendererBuilder;

#[test]
fn a_failed_egl_initialization_is_an_error_not_a_process_abort() {
    // With no resolvable vendor library, EGL display initialization fails and
    // MapLibre Native throws from headless-frontend construction — the same
    // boundary a broken production display environment fails through
    // (observed as `eglCreateContext() returned error 0x3001` followed by
    // `std::terminate` before construction became fallible).
    std::env::set_var(
        "__EGL_VENDOR_LIBRARY_FILENAMES",
        "/nonexistent/egl_vendor.json",
    );

    let error = ImageRendererBuilder::new()
        .with_size(NonZeroU32::new(64).unwrap(), NonZeroU32::new(64).unwrap())
        .with_pixel_ratio(1.0)
        .try_build_static_renderer()
        .err()
        .expect("EGL cannot initialize in this environment, so construction must fail");

    // The native exception's text must survive the crossing so operators can
    // tell a graphics-stack failure from every other construction error.
    assert!(
        !error.to_string().is_empty(),
        "the error must carry the native exception message"
    );
}
