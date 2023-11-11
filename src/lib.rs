use windows::core::w;

mod registry;

pub fn get_program_files_dir() -> Option<String> {
    let key = registry::Key::open(
        registry::HKEY_LOCAL_MACHINE,
        w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion"),
    )
    .ok()?;
    if let Some(registry::Value::String(s)) = key.value(w!("ProgramFilesDir")) {
        return Some(s);
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
        for v in k.values().unwrap() {
            match v {
                registry::Value::Binary(data) => println!("{:?}", data),
                registry::Value::DWord(num) => println!("{}", num),
                registry::Value::MultiString(arr) => println!("{:?}", arr),
                registry::Value::QWord(num) => println!("{}", num),
                registry::Value::String(s) => println!("{}", s),
            }
        }
    }
}
