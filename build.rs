use rustversion_detect::RUST_VERSION;

pub fn main() {
    // version detection
    emit_check_cfg("has_cfg_panic", None);
    if RUST_VERSION.is_since_minor_version(1, 60) {
        println!("cargo:rustc-cfg=has_cfg_panic");
    }
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
