use wixpkgdep;

fn main() {
    if let Some(path) = wixpkgdep::get_program_files_dir() {
        println!("{path}");
    }
}
