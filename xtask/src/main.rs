use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("dist") => dist()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
dist            builds application and man pages
"
    )
}

fn dist() -> Result<(), DynError> {
    let _ = fs::remove_dir_all(dist_dir());
    fs::create_dir_all(dist_dir())?;

    dist_binary("1")?;
    dist_binary("2")?;

    Ok(())
}

fn dist_binary(dest: &str) -> Result<(), DynError> {
    env::set_var("NAME", "Perf name");
    env::set_var("PARAMS", "param7781,param2,param7");
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .current_dir(project_root())
        .args(&["build", "--release", "--config", "rustc-env='SOME_ENV=VALUE'"])
        .status()?;

    if !status.success() {
        Err("cargo build failed")?;
    }

    let src = project_root().join("target/release/libsplit_plug.so");

    fs::copy(&src, dist_dir().join("hello-world".to_string() + dest + ".clap"))?;

    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn dist_dir() -> PathBuf {
    project_root().join("target/dist")
}
