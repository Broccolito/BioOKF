fn main() {
    if std::env::var("PROFILE").as_deref() == Ok("debug") {
        let bin_dir = std::path::Path::new("bin");
        let _ = std::fs::create_dir_all(bin_dir);
        for name in ["bokf", "bokf-mcp"] {
            let path = bin_dir.join(name);
            if !path.exists() {
                let script = format!(
                    "#!/bin/sh\nDIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec \"$DIR/../../target/debug/{name}\" \"$@\"\n"
                );
                if std::fs::write(&path, script).is_ok() {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ =
                            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
                    }
                }
            }
        }
    }
    tauri_build::build()
}
