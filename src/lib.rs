use windows::core::w;

mod registry;

pub fn get_program_files_dir() -> Option<String> {
    let path = w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion");
    let name = w!("ProgramFilesDir");

    let key = registry::Key::open(registry::HKEY_LOCAL_MACHINE, &path).ok()?;
    if let Ok(Some(value)) = key.string_value(&name) {
        unsafe {
            return Some(String::from_utf16_lossy(value.as_wide()));
        }
    }

    None
}
