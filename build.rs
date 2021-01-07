use std::{
    collections::BTreeMap,
    convert::TryFrom,
    io::{Read, Write},
    path::PathBuf,
};

const SHIM: &'static str = r#"
mod _editor_shim {
    pub type Program = impl core::future::Future<Output = ()>;

    #[allow(dead_code)]
    pub unsafe fn program() -> Program {
        async move { panic!() }
    }
}
"#;

fn main() {
    let manifest_dir = PathBuf::try_from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    let mut programs = Vec::new();
    {
        let programs_dir = manifest_dir.join("src").join("programs");
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(programs_dir.join("mod.rs"))
            .unwrap();
        let mut declarations = format!("// THIS FILE IS AUTO-GENERATED, DO NOT EDIT \n\n");
        for path in std::fs::read_dir(programs_dir).unwrap() {
            let name = path
                .unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();
            if name == "mod" {
                continue;
            }
            programs.push(name.clone());
            declarations.push_str(&format!(
                "mod {name};
#[cfg(feature = {:?})]
pub use {name}::{{{name} as program, Program}};\n",
                name = name
            ));
        }
        declarations.push_str(&format!(
            "\n#[cfg(not(any(
  {cfgs}
)))]{}
#[cfg(not(any(
  {cfgs}
)))]
pub use _editor_shim::*;
",
            SHIM,
            cfgs = programs
                .iter()
                .map(|program| { format!("feature = {:?}", program) })
                .collect::<Vec<_>>()
                .join(", "),
        ));
        f.write_all(declarations.as_bytes()).unwrap();
    }
    let mut buf = String::new();
    std::fs::File::open(manifest_dir.join("Cargo.toml"))
        .unwrap()
        .read_to_string(&mut buf)
        .unwrap();
    let mut manifest: cargo_toml::Manifest = toml::from_str(&buf).unwrap();
    manifest.features = BTreeMap::new();
    for program in programs {
        manifest.features.insert(program, vec![]);
    }
    let manifest = toml::Value::try_from(manifest).unwrap();
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(manifest_dir.join("Cargo.toml"))
        .unwrap()
        .write_all(manifest.to_string().as_bytes())
        .unwrap();
}
