//! Prefers AOT: parse each `templates/*.dejavu` to IR JSON under OUT_DIR.
//! Dyn fallback embeds sources for `render_source` when the `aot` feature is off.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let templates_dir = manifest_dir.join("templates");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let aot_dir = out_dir.join("aot");
    fs::create_dir_all(&aot_dir).expect("create OUT_DIR/aot");

    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=build.rs");

    let mut entries = fs::read_dir(&templates_dir)
        .expect("read templates/")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| ext == "dejavu")
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.path());

    let aot = env::var("CARGO_FEATURE_AOT").is_ok();
    let dyn_only = !aot && env::var("CARGO_FEATURE_DYN").is_ok();
    if !aot && !dyn_only {
        panic!("vos-generator: enable feature `aot` (default) or `dyn`");
    }
    let mut match_arms = String::new();
    let mut name_list = Vec::new();

    for entry in &entries {
        let path = entry.path();
        let stem = template_stem(&path);
        name_list.push(stem.clone());
        println!("cargo:rerun-if-changed={}", path.display());

        let source = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("read {}: {e}", path.display());
        });

        if aot {
            let doc = dejavu::Dejavu::parse(&source).unwrap_or_else(|e| {
                panic!("AOT parse {}: {e:?}", path.display());
            });
            let ir_path = aot_dir.join(format!("{stem}.ir.json"));
            let json = serde_json::to_string_pretty(&doc).expect("serialize IR");
            fs::write(&ir_path, json).expect("write IR JSON");
            match_arms.push_str(&format!(
                "        {stem:?} => render_aot_ir(include_str!(concat!(env!(\"OUT_DIR\"), \"/aot/{stem}.ir.json\")), ctx),\n"
            ));
        } else {
            let src_path = aot_dir.join(format!("{stem}.src.dejavu"));
            fs::write(&src_path, &source).expect("write dyn source copy");
            match_arms.push_str(&format!(
                "        {stem:?} => render_dyn_source(include_str!(concat!(env!(\"OUT_DIR\"), \"/aot/{stem}.src.dejavu\")), ctx),\n"
            ));
        }
    }

    let registry = format!(
        r#"/// Template names registered from `templates/`.
pub const TEMPLATE_NAMES: &[&str] = &{names:?};

pub(crate) fn render_registered(name: &str, ctx: &serde_json::Value) -> crate::Result<String> {{
    match name {{
{arms}        other => Err(crate::Error::UnknownTemplate(other.to_owned())),
    }}
}}
"#,
        names = name_list,
        arms = match_arms,
    );
    fs::write(out_dir.join("aot_registry.rs"), registry).expect("write aot_registry.rs");
}

/// `rust_struct.rs.dejavu` → `rust_struct`
fn template_stem(path: &Path) -> String {
    let file = path
        .file_name()
        .and_then(|s| s.to_str())
        .expect("utf-8 template name");
    file.strip_suffix(".dejavu")
        .unwrap_or(file)
        .trim_end_matches(".rs")
        .trim_end_matches(".ts")
        .trim_end_matches(".js")
        .to_owned()
}
