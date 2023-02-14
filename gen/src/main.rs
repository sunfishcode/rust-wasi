use std::ffi::OsStr;
use std::io;
use std::path::Path;
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

    for proposal in [
        "wasi-clocks",
        "wasi-filesystem",
        "wasi-io",
        "wasi-logging",
        "wasi-poll",
        "wasi-random",
        //"wasi-sockets", // temporarily
    ] {
        let success = Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg("--quiet")
            .arg(format!("https://github.com/WebAssembly/{proposal}"))
            .current_dir(tmp.path())
            .status()?
            .success();
        if !success {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "git clone invocation failed",
            ));
        }
        if proposal == "wasi-sockets" {
            for entry in std::fs::read_dir(tmp.path().join(proposal))? {
                let entry = entry?;
                if entry.file_name() == "world.wit" || entry.file_name() == "world.wit.md" {
                    continue;
                }
                if entry.path().extension() != Some(OsStr::new("wit")) {
                    continue;
                }
                std::fs::copy(entry.path(), Path::new("../wit").join(entry.file_name()))?;
            }
        } else {
            for entry in std::fs::read_dir(tmp.path().join(proposal).join("wit"))? {
                let entry = entry?;
                if entry.file_name() == "world.wit" || entry.file_name() == "world.wit.md" {
                    continue;
                }
                if entry.path().extension() != Some(OsStr::new("wit"))
                    && entry.path().extension() != Some(OsStr::new("md"))
                {
                    continue;
                }
                if entry.file_type()?.is_dir() {
                    todo!("subdirectories");
                } else {
                    std::fs::copy(entry.path(), Path::new("../wit").join(entry.file_name()))?;
                }
            }
        }
    }


    let success = Command::new(tmp.path().join("bin/wit-bindgen"))
    .current_dir("../src")
    .arg("rust")
    .arg("--rustfmt")
    .arg("--std-feature")
    .arg("../wit")
    .status()?.success();
    if !success {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("wit-bindgen invocation failed")
        ));
    }

    Ok(())
}
