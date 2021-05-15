use std::path::{Path, PathBuf};
use std::process::{Stdio};

use anyhow::Result;
use indoc::formatdoc;
use tracing::{info, debug};

use tokio::fs::{create_dir_all, create_dir, remove_dir_all};
use tokio::fs::{write, rename};
use tokio::process::Command;


static OUTPUT: &str = "./out";


pub async fn compile_code(name: &str, code: &str) -> Result<PathBuf> {
    let path = format!("./generate/{}", name);
    info!("generating code with target @ {target}", target=&path);

    create_dir_all(&path).await?;
    create_dir_all(format!("{}/src", &path)).await?;
    debug!("cargo directories made successfully");

    let toml = generate_toml(name);
    write(format!("{}/Cargo.toml", &path), &toml).await?;
    debug!("cargo toml made successfully");

    write(format!("{}/src/lib.rs", &path), code).await?;
    debug!("code injection made successfully");

    info!("beginning compilation");
    let target = Path::new(&path);
    compile(target.canonicalize()?).await?;
    info!("compilation complete");

    ensure_out().await?;

    let output = format!("{}/target/release/{}.dll", &path, name);
    let resulting_output = format!("{}/{}.dll", OUTPUT, name);
    info!("extracting built dll @ {path}", path=&output);
    rename(&output, &resulting_output).await?;
    info!("extraction complete, dll @ {path}", path=&resulting_output);

    remove_dir_all(&path).await?;
    info!("generation folder removed");

    let result = Path::new(&resulting_output).canonicalize()?;
    Ok(result)
}


async fn ensure_out() -> Result<()> {
    debug!("ensuring output directory exists");
    let path = Path::new(OUTPUT);
    if path.exists() {
        debug!("directory already exists, skipping");
        return Ok(())
    }

    debug!("directory does not exist, creating directory");
    create_dir(path).await?;
    debug!("directory created");

    Ok(())
}


async fn compile(target: PathBuf) -> Result<()> {
    let code = Command::new("cargo")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .current_dir(target)
        .args(&["build", "--release"])
        .spawn()?
        .wait()
        .await?;

    info!(
        "cargo finished with exit code {}, success = {}",
        code.code().unwrap_or(0),
        code.success(),
    );

    Ok(())
}


fn generate_toml(name: &str) -> String {
    formatdoc!(r#"
    [package]
    name = "{name}"
    version = "0.1.0"
    authors = ["quark-generate"]
    edition = "2018"

    [lib]
    name = "{name}"
    crate-type = ["dylib"]

    [dependencies]
    anyhow = "1"
    rkyv = "0.6.4"
    "#, name=name)
}


#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber;
    use super::*;

    #[tokio::test]
    async fn test_new() -> Result<()> {
        tracing_subscriber::fmt()
            // Configure formatting settings
            .with_max_level(Level::DEBUG)
            .with_target(false)
            .with_timer(tracing_subscriber::fmt::time::time())
            .with_level(true)
            // Set the collector as the default.
            .init();

        compile_code("ahh", "fn main() { println!(\"hello\") }").await?;

        Ok(())
    }
}