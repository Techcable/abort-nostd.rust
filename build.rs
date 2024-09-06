use rustversion_detect::RUST_VERSION;

pub fn main() {
    // version detection
    emit_check_cfg("has_cfg_panic", None);
    if RUST_VERSION.is_since_minor_version(1, 60) {
        println!("cargo:rustc-cfg=has_cfg_panic");
    }
    emit_check_cfg("has_doc_cfg", None);
    if RUST_VERSION.is_nightly() {
        println!("cargo:rustc-cfg=has_doc_cfg");
    }
    let target_arch = {
        let mut values = load_cargo_cfg_var("target_arch");
        assert_eq!(
            values.len(),
            1,
            "Must have one and only one `cfg!(target_arch)`"
        );
        values.remove(0)
    };
    // can't use matchs! due to MSRV
    #[allow(clippy::match_like_matches_macro)]
    let supported_arch = match &*target_arch {
        "x86_64" | "x86" | "arm" | "aarch64" => true,
        _ => false,
    };
    let trap_impl_name = if target_arch == "wasm32" && RUST_VERSION.is_since_minor_version(1, 37) {
        // Use core::arch::wasm32::unreachable() intrinsic
        //
        // Requires `simd_wasm32` feature for the module (stable 1.33),
        // and the `unreachable_wasm32` feature for the function (stable 1.37)
        "wasm32-intrinsic"
    } else if target_arch == "wasm64" && RUST_VERSION.is_nightly() {
        // Use core::arch::wasm64::unreachable() intrinsic
        //
        // Requires `simd_wasm64` feature for the module (unstable, issue #90599)
        emit_warning(&"The `wasm64` architecture is currently untested (issue #3)");
        "wasm64-intrinsic"
    } else if RUST_VERSION.is_nightly() {
        // The `core::intrinsics` module requires nightly.
        // It is an "internal" feature that will never be directly stabilized.
        "core-intrinsics"
    } else if supported_arch && RUST_VERSION.is_since_minor_version(1, 59) {
        "assembly"
    } else {
        "fallback"
    };
    emit_check_cfg(
        "trap_impl",
        Some(vec![
            "core-intrinsics",
            "assembly",
            "wasm32-intrinsic",
            "wasm64-intrinsic",
            "fallback",
        ]),
    );
    println!("cargo:rustc-cfg=trap_impl=\"{}\"", trap_impl_name);
    let abort_impl_name = if has_cargo_feature("std") {
        "std"
    } else if has_cargo_feature("libc") {
        "libc"
    } else if has_cargo_feature("abort-via-trap") && trap_impl_name != "fallback" {
        "trap"
    } else {
        "fallback"
    };
    emit_check_cfg("abort_impl", Some(vec!["std", "libc", "trap", "fallback"]));
    println!("cargo:rustc-cfg=abort_impl=\"{}\"", abort_impl_name);
    // never need to be re-run
    println!("cargo:rerun-if-changed=build.rs");
}

fn load_cargo_cfg_var(name: &'static str) -> Vec<String> {
    let env_var = format!("CARGO_CFG_{}", name.to_uppercase());
    match std::env::var(&env_var) {
        Ok(val) => val.split(",").map(String::from).collect(),
        Err(std::env::VarError::NotUnicode(_)) => panic!("Var not unicode: {:?}", env_var),
        Err(std::env::VarError::NotPresent) => vec![],
    }
}

fn has_cargo_feature(name: &str) -> bool {
    let name = name.replace('-', "_").to_uppercase();
    std::env::var_os(format!("CARGO_FEATURE_{}", name)).is_some()
}

fn emit_check_cfg(name: &'static str, values: Option<Vec<&str>>) {
    let mut values_spec = String::new();
    if let Some(values) = values {
        values_spec.push_str(", values(");
        for (index, value) in values.into_iter().enumerate() {
            if index > 0 {
                values_spec.push_str(", ");
            }
            values_spec.push('"');
            values_spec.push_str(value);
            values_spec.push('"');
        }
        values_spec.push(')');
    }
    println!("cargo:rustc-check-cfg=cfg({}{})", name, values_spec);
}

fn emit_warning(msg: &dyn std::fmt::Display) {
    println!("cargo:warning={}", msg);
}
