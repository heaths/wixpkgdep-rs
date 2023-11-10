use windows::core::w;

mod registry;

pub fn get_program_files_dir() -> Option<String> {
    let key = registry::Key::open(
        registry::HKEY_LOCAL_MACHINE,
        w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion"),
    )
    .ok()?;
    if let Ok(Some(value)) = key.string_value(w!("ProgramFilesDir")) {
        unsafe {
            return Some(String::from_utf16_lossy(value.as_wide()));
        }
    }

    None
}

pub fn enum_installers() {
    let key = registry::Key::open(
        registry::HKEY_CURRENT_USER,
        w!("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    )
    .unwrap();
    for k in key.keys().unwrap() {
        if let Ok(Some(display_name)) = k.string_value(w!("DisplayName")) {
            unsafe {
                let display_name = display_name.to_string().unwrap();
                println!("{display_name}");
            }
        }
    }
}
