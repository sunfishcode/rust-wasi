use std::io;
use std::process::Command;

fn main() -> io::Result<()> {
    let tmp = tempfile::tempdir()?;

    // Install the wit-bindgen tool.
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

    // Generate the main CLI bindings.
    let success = Command::new(tmp.path().join("bin/wit-bindgen"))
        .current_dir("../src")
        .arg("rust")
        .arg("--rustfmt")
        .arg("--std-feature")
        .arg("--macro-export")
        .arg("../wasi-cli/wit")
        .status()?
        .success();
    if !success {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("wit-bindgen invocation failed"),
        ));
    }

    // Generate the main cli-reactor bindings.
    let success = Command::new(tmp.path().join("bin/wit-bindgen"))
        .current_dir("../src")
        .arg("rust")
        .arg("--rustfmt")
        .arg("--std-feature")
        .arg("--macro-export")
        .arg("--skip")
        .arg("command")
        .arg("../wasi-cli-reactor/wit")
        .status()?
        .success();
    if !success {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("wit-bindgen invocation failed"),
        ));
    }

    // Generate the specialized CLI bindings used by the polyfill.
    let success = Command::new(tmp.path().join("bin/wit-bindgen"))
        .current_dir("../src")
        .arg("rust")
        .arg("--rustfmt")
        .arg("--std-feature")
        // The polyfill gets passed strings in raw-byte form and needs to be able
        // to pass them into APIs expecting `string`s. The Canonical ABI will
        // diagnose invalid Unicode issues.
        .arg("--raw-strings")
        // The generated definition of these fuctions will pull in the allocator, so we
        // are defining them manually in the polyfill.
        .arg("--skip")
        .arg("command")
        .arg("--skip")
        .arg("preopens")
        .arg("--skip")
        .arg("get-environment")
        // Write the bindings to an alternate location.
        .arg("--out-dir")
        .arg("polyfill")
        .arg("../wasi-cli/wit")
        .status()?
        .success();
    if !success {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("wit-bindgen invocation failed"),
        ));
    }

    // Generate the specialized cli-reactor bindings used by the polyfill.
    let success = Command::new(tmp.path().join("bin/wit-bindgen"))
        .current_dir("../src")
        .arg("rust")
        .arg("--rustfmt")
        .arg("--std-feature")
        // The polyfill gets passed strings in raw-byte form and needs to be able
        // to pass them into APIs expecting `string`s. The Canonical ABI will
        // diagnose invalid Unicode issues.
        .arg("--raw-strings")
        // The generated definition of these fuctions will pull in the allocator, so we
        // are defining them manually in the polyfill.
        .arg("--skip")
        .arg("preopens")
        .arg("--skip")
        .arg("get-environment")
        .arg("--skip")
        .arg("command")
        // Write the bindings to an alternate location.
        .arg("--out-dir")
        .arg("polyfill")
        .arg("../wasi-cli-reactor/wit")
        .status()?
        .success();
    if !success {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("wit-bindgen invocation failed"),
        ));
    }

    Ok(())
}
