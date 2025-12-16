use std::io;
use std::process::Command;

fn main() -> io::Result<()> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--all")
        .status()?;

    #[cfg(windows)]
    {
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = workspace_root.ancestors().nth(2).unwrap().to_path_buf();

        let installers_dir = workspace_root.join("installers");
        let nsis_dir = installers_dir.join("NSIS");

        // Try common NSIS installation paths, then fall back to PATH
        let nsis_paths = [
            r"C:\Program Files (x86)\NSIS\makensis.exe",
            r"C:\Program Files\NSIS\makensis.exe",
            "makensis.exe", // Fall back to PATH
        ];

        let makensis = nsis_paths
            .iter()
            .find(|path| {
                if **path == "makensis.exe" {
                    // For PATH lookup, always try it as last resort
                    true
                } else {
                    std::path::Path::new(*path).exists()
                }
            })
            .expect("makensis.exe not found. Please install NSIS.");

        let status = Command::new(*makensis)
            .current_dir(&nsis_dir)
            .arg("installer.nsi")
            .status()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "makensis failed to build installer",
            ));
        }

        std::fs::create_dir_all(workspace_root.join("target").join("dist"))?;

        std::fs::copy(
            nsis_dir.join("Equicord Installer.exe"),
            workspace_root
                .join("target")
                .join("release")
                .join("Equicord Installer.exe"),
        )?;
    }

    Ok(())
}
