#![doc = include_str!("../README.md")]

// Included bindings are generated from After Effects SDK dated May 2023

#[cfg(all(target_os = "windows", builtin_bindings))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings_win.rs"));

#[cfg(all(any(target_os = "macos", target_os = "linux"), builtin_bindings))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings_macos.rs"));

#[cfg(not(builtin_bindings))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
