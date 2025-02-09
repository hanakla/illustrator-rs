extern crate bindgen;

use glob;
use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.hpp");

    println!("{}", env::var("AISDK_ROOT").unwrap_or("".to_string()));
    if env::var("AISDK_ROOT").is_err() {
        println!("cargo:rustc-cfg=builtin_bindings");
        return;
    }

    let ai_sdk_path = &env::var("AISDK_ROOT").expect("AISDK_ROOT is not set");
    let search_paths = vec![
        Path::new(ai_sdk_path).join("illustratorapi"),
        Path::new(ai_sdk_path).join("samplecode").join("common"),
    ];
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut include_dirs = Vec::new();
    search_paths.iter().for_each(|path| {
        glob::glob_with(
            format!("{}/**", path.display()).as_str(),
            glob::MatchOptions {
                case_sensitive: false,
                require_literal_separator: false,
                require_literal_leading_dot: false,
            },
        )
        .unwrap()
        .filter(|entry| entry.is_ok())
        .for_each(|entry| {
            include_dirs.push(entry.unwrap().display().to_string());
        });
    });

    let mut ai_bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        // .allowlist_type("AI.*")
        // .allowlist_type("ActionParamType")
        // .allowlist_type("ai::.*")
        // .allowlist_type("AI::.*")
        .allowlist_type("AI.*")
        .allowlist_type("ActionParamType")
        .allowlist_type("ai::.*")
        .allowlist_type("AI::.*")
        .allowlist_type("AS.*")
        .allowlist_type("AT.*")
        .allowlist_type("kSP.*")
        .allowlist_type("kAI.*")
        .allowlist_type(".*Err.*")
        .allowlist_type("ai_sys::.*")
        .allowlist_type("k?AI.*Suite.*")
        .allowlist_type("SP.*Suite.*")
        .allowlist_type("SP.*Message.*")
        .allowlist_type("Suites")
        .allowlist_type(".*Plugin.*")
        .allowlist_type("P[A-Z]_InData")
        .allowlist_type("pr::.*")
        .allowlist_type("PiPL::.*")
        .allowlist_function("SP.*Suite.*")
        .allowlist_function("ai::.*")
        .allowlist_function("unicode_string_from_utf8")
        .allowlist_function("std_string_to_c_stf")
        // .allowlist_function(".*Plugin.*")
        // .allowlist_function("Fixup.*")
        .allowlist_function("kSP.*")
        .allowlist_function("kAI.*")
        .allowlist_var("kSP.*")
        .allowlist_var("kAI.*")
        .clang_arg("-std=c++14")
        .clang_args(&["-x", "c++"])
        .layout_tests(false);

    for entry in &include_dirs {
        ai_bindings = ai_bindings.clang_arg(format!("-I{}", entry));
    }

    if cfg!(target_os = "macos") {
        ai_bindings = ai_bindings
            //.clang_arg("-I/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/System/Library/Frameworks/CoreFoundation.framework/Versions/A/Headers/")
            //.clang_arg("-I/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/System/Library/Frameworks/CoreServices.framework/Versions/A/Headers/")
            //.clang_arg("-I/Library/Developer/CommandLineTools/usr/include/c++/v1/")
            .clang_arg(
                // FIXME: This will bitrot when clang updates or on really old macos instances
                "-I/Library/Developer/CommandLineTools/usr/lib/clang/12.0.0/include/stdint.h",
            )
            .clang_arg(
                "-F/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/System/Library/Frameworks/",
            );
    }

    ai_bindings
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("done");
}
