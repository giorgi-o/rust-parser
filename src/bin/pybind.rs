use std::{fs::OpenOptions, io::Write as _, path::PathBuf, process::Command};

use temp_dir::TempDir;

/// Not to be confused with the C++ "pybind11" library.

fn main() {
    // 0. create temporary folder
    let temp_dir = TempDir::with_prefix("pybind").expect("Failed to create temp dir");

    // 1. create cargo project in folder
    const MODULE_NAME: &str = "pybind";
    Command::new("cargo")
        .args(&["new", "--lib", MODULE_NAME])
        .current_dir(temp_dir.path())
        .status()
        .expect("Failed to create cargo project");

    let proj_dir = temp_dir.path().join(MODULE_NAME);

    // 2. follow instructions in https://www.maturin.rs/tutorial
    // 2.1 add config to Cargo.toml
    write_to(
        proj_dir.join("Cargo.toml"),
        true,
        format!(
            r#"

[lib]
name = "{MODULE_NAME}"
# "cdylib" is necessary to produce a shared library for Python to import from.
crate-type = ["cdylib"]

[dependencies.pyo3]
version = "0.22.2"
# "abi3-py38" tells pyo3 (and maturin) to build using the stable ABI with minimum Python version 3.8
features = ["abi3-py38"]
"#
        ),
    );

    // 2.2 create pyproject.toml
    write_to(
        proj_dir.join("pyproject.toml"),
        false,
        r#"
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[tool.maturin]
# "extension-module" tells pyo3 we want to build an extension module (skips linking against libpython.so)
features = ["pyo3/extension-module"]
"#,
    );

    // 2.3 create src/lib.rs
    write_to(
        proj_dir.join("src").join("lib.rs"),
        false,
        format!(
            r#"
        use pyo3::prelude::*;

#[pymodule]
fn {MODULE_NAME}(m: &Bound<'_, PyModule>) -> PyResult<()> {{
    m.add_function(wrap_pyfunction!(greet, m)?)?;

    Ok(())
    }}

#[pyfunction]
fn greet(name: &str) -> PyResult<String> {{
    Ok(format!("Hello, {{}}!", name))
}}
"#
        ),
    );

    // 3. create python venv and install maturin
    // 3.1 create venv
    Command::new("python")
        .args(&["-m", "venv", ".venv"])
        .current_dir(&proj_dir)
        .status()
        .expect("Failed to create python venv");

    // 3.2 install maturin
    let venv_scripts_path = proj_dir.join(".venv").join("Scripts");
    let pip_path = venv_scripts_path.join("pip.exe");
    Command::new(pip_path)
        .args(&["install", "-U", "pip", "maturin"])
        .current_dir(&proj_dir)
        .status()
        .expect("Failed to install maturin");

    // 4. build the project
    let maturin_path = venv_scripts_path.join("maturin.exe");
    Command::new(maturin_path)
        .args(&["build"])
        .current_dir(&proj_dir)
        .status()
        .expect("Failed to build project");

    // 4. copy the shared library and wheel to the current directory
    // todo: make this work on non-windows
    let wheel_path = proj_dir
        .join("target")
        .join("wheels")
        .read_dir()
        .expect("Failed to read wheels/ dir")
        .next()
        .expect("No wheels found in wheels/ dir")
        .expect("Failed to get wheel path")
        .path();

    let shared_lib_path = proj_dir
        .join("target")
        .join("debug")
        .read_dir()
        .expect("Failed to read debug/ dir")
        .find(|entry| {
            entry
                .as_ref()
                .expect("Failed to get entry")
                .file_name()
                .to_string_lossy()
                .ends_with(".dll")
        })
        .expect("No shared library found in debug/ dir")
        .expect("Failed to get shared library path")
        .path();

    for path in &[&wheel_path, &shared_lib_path] {
        let destination = path
            .file_name()
            .expect("Failed to get file name")
            .to_string_lossy()
            .to_string();

        std::fs::copy(path, &destination).expect("Failed to copy file");
        println!("Copied {:?} to current directory", destination);
    }

    // 4.1 rename .dll file to .pyd (so that it can be imported in Python)
    std::fs::rename(
        shared_lib_path
            .file_name()
            .expect("Failed to get file name"),
        "pybind.pyd",
    )
    .expect("Failed to rename shared library");
}

pub fn write_to(file: PathBuf, append: bool, content: impl AsRef<str>) {
    OpenOptions::new()
        .write(true)
        .append(append)
        .create(!append)
        .open(file)
        .expect("Failed to open file")
        .write_all(content.as_ref().as_bytes())
        .expect("Failed to write to file");
}
