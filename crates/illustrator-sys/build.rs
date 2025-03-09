extern crate bindgen;

use bindgen::builder;
use glob::{glob_with, glob};
use std::{
    env, io::Write, path::{Path, PathBuf}
};

use std::process::Command;
use std::collections::HashMap;
use std::io::{self};
use wildcard::Wildcard;
use regex::Regex;


fn main() {
    println!("cargo:rerun-if-changed=wrapper.hpp");

    println!("{}", env::var("AISDK_ROOT").unwrap_or("".to_string()));
    if env::var("AISDK_ROOT").is_err() {
        println!("cargo:rustc-cfg=builtin_bindings");
    }

    let ai_sdk_path = &env::var("AISDK_ROOT").expect("AISDK_ROOT is not set");
    let search_paths = vec![
        Path::new(ai_sdk_path).join("illustratorapi"),
        Path::new(ai_sdk_path).join("samplecode").join("common"),
    ];
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut include_dirs = Vec::new();
    search_paths.iter().for_each(|path| {
        glob_with(
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

        // .allowlist_file(arg)
        .allowlist_function("SP.*Suite.*")
        .allowlist_function("ai::.*")
        .allowlist_function("kSP.*")
        .allowlist_function("kAI.*")

        .allowlist_var("sAI.*")
        .allowlist_var("sSP.*")
        .allowlist_var("kSP.*")
        .allowlist_var("kAI.*")
        .allowlist_var("k.*Err")
        .allowlist_var("AIAPI_VERSION")
        .allowlist_var("kAIUserSuiteVersion")

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

    let consts = extract_constants_from_clang(&include_dirs).unwrap();

    // Append consts to bindings.rs
    let bindings = &mut std::fs::OpenOptions::new()
        .append(true)
        .open(out_path.join("bindings.rs"))
        .unwrap();

    bindings.write_all(consts.as_bytes()).unwrap();

    // let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    // let platform = env::var("PLATFORM").unwrap();
    //
    // let Some(entry) = glob(format!("../../target/{}/build/illustrator-rs-*/out/bindings.rs", profile).as_str()).unwrap().next() else { todo!(); };
    // let source_path = entry.unwrap();
    // let destination_path = PathBuf::from(format!("./bindings_{}.rs", platform));
    //
    // std::fs::copy(source_path, destination_path).expect("Failed to copy bindings to bindings_macos.rs");

    println!("done");
}

fn extract_constants_from_clang(include_dirs: &Vec<String>) -> io::Result<String> {
  let mut cmd = Command::new("clang");
  cmd.arg("-dM").arg("-E").arg("wrapper.hpp");

  for dir in include_dirs {
      cmd.arg("-I").arg(dir);
  }

  let output = cmd.output()?;

  if !output.status.success() {
      eprintln!("clang failed with status: {}", output.status);
      return Err(io::Error::new(io::ErrorKind::Other, "clang failed"));
  }

  let stdout = String::from_utf8_lossy(&output.stdout);
  let mut defines = HashMap::new();

  let filter_patterns = vec![
      Wildcard::new("kAI*".as_bytes()).unwrap(),
  ];

  for line in stdout.lines() {
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() == 3 {
          defines.insert(parts[1].to_string(), parts[2].to_string());
      }
  }

  fn expand_macro(name: &str, defines: &HashMap<String, String>) -> Option<String> {
      let Some(value) = defines.get(name) else {
        panic!("Macro not found: {}", name);
      };

      if value.contains("(") && value.contains(")") {
          let args = value.trim_matches(|c| c == '(' || c == ')');

          if let Ok(arg_value) = args.parse::<i32>() {
              return Some(arg_value.to_string());
          } else if defines.contains_key(args) {
              return expand_macro(args, defines);
          }
      }

      Some(value.clone())
  }

  let mut constants = String::new();
  constants.push_str(r#"fn AIAPI_VERSION(i32 v) -> i32 {v + 1000}"#);
  constants.push_str(r#"macro_rules! to_u32_char {
    ($input:expr) => {{
        let mut result = 0u32;
        for (i, c) in $input.chars().enumerate() {
            result |= (c as u32) << (8 * i); // 順番にシフトして結合
        }
        result
    }};
}"#);

  fn to_rust_value(key: String, value: String, defines: &HashMap<String, String>) -> String {
    let macro_pattern = Regex::new(r"^\w+\(.*\)$").unwrap();
    let unbrace_regex = Regex::new(r"^\((.*)\)$").unwrap();

    if let Ok(val) = value.parse::<i32>() {
      return format!(
          "pub const {}: i32 = {};\n",
          key, val
      );
    }
    else if value.ends_with("f") {
        let val = &value[..value.len() - 1];
        return format!(
            "pub const {}: f32 = {};\n",
            key, val.strip_suffix("f").unwrap()
        );
    }
    else if value.starts_with("\"") && value.ends_with("\"") {
        let val = &value[1..value.len() - 1];
        let byte_array: Vec<u8> = val.as_bytes().to_vec();
        return format!(
            "pub const {}: &[u8; {}] = b\"{}\\0\";\n",
            key, byte_array.len() + 1, val
        );
    } else if value.starts_with("'") && value.ends_with("'") {
        let val = &value[1..value.len() - 1];
        return format!(
            "pub const {}: u32 = to_u32_char!('{}');\n",
            key, val
        );
    }
    else if macro_pattern.is_match(&value) {
        let expanded_value = expand_macro(&key, &defines);
        if let Some(expanded) = expanded_value {
            return format!(
                "pub const {}: i32 = {};\n",
                key, expanded
            );
        } else {
            panic!("Failed to expand macro: {}", key);
        }
    }
    else if unbrace_regex.is_match(&value) {
        let unbraced = unbrace_regex.replace(&value, "$1");
        return to_rust_value(key, unbraced.to_string(), defines)
    } else {
        println!("elsing {}: {}", key, value);
        let value = defines.get(&value).unwrap();
        return to_rust_value(key, value.clone(), defines)
    }
  }

  println!("hashmap: {:?}", defines);
  for (key, value) in &defines {
      if !filter_patterns.iter().any(|w| w.is_match(key.as_bytes())) {
          continue;
      }

      constants.push_str(&to_rust_value(key.clone(), value.clone(), &defines));
  }

  println!("AIAPI_VERSION: {:?}", defines.get("AIAPI_VERSION"));

  if !constants.is_empty() {
      println!("{}", constants);
  } else {
      println!("No matching constants found.");
  }

  Ok(constants)
}
