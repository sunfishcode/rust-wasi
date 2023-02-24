use std::io;
use std::process::Command;

fn main() -> io::Result<()> {
    let tmp = tempfile::tempdir()?;

    let success = Command::new("cargo")
        .arg("install")
        .arg("--locked")
        .arg("--quiet")
        // Use the latest wit-bindgen, which has better no-std support.
        //.arg("--version")
        //.arg("0.3.0")
        .arg("--git")
        .arg("https://github.com/bytecodealliance/wit-bindgen")
        .arg("--root")
        .arg(tmp.path())
        .arg("wit-bindgen-cli")
        .status()?
        .success();
    if !success {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "cargo install invocation failed",
        ));
    }

    let success = Command::new(tmp.path().join("bin/wit-bindgen"))
    .current_dir("../src")
    .arg("rust")
    .arg("--rustfmt")
    .arg("--std-feature")
    .arg("../wasi-cli/wit")
    .status()?.success();
    if !success {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("wit-bindgen invocation failed")
        ));
    }

    Ok(())
}
