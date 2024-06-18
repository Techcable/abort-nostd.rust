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
    let target_families = load_cargo_cfg_var("target_family");
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
    let trap_impl_name = if RUST_VERSION.is_nightly() {
        "core-intrinsics"
    } else if target_families.contains(&"wasm".into()) {
        "wasm"
    } else if supported_arch {
        "assembly"
    } else {
        "fallback"
    };
    emit_check_cfg(
        "trap_impl",
        Some(vec!["core-intrinsics", "assembly", "wasm", "fallback"]),
    );
    println!("cargo:rustc-cfg=trap_impl=\"{}\"", trap_impl_name);
    let abort_impl_name = if has_cargo_feature("std") {
        "std"
    } else if has_cargo_feature("libc") {
        "libc"
    } else {
        "fallback"
    };
    emit_check_cfg("abort_impl", Some(vec!["std", "libc", "fallback"]));
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
    std::env::var_os(format!("CARGO_FEATURE_{name}")).is_some()
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
    println!("cargo:rustc-check-cfg=cfg({name}{values_spec})");
}
