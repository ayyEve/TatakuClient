use std::io::Write;

const INCLUDE_DIRS: &[&str] = &[
    "menus",
    "dialogs"
];
const OUTPUT: &str = "src/lib.rs";

fn main() {
    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(OUTPUT)
        .expect("error opening output file");

    for i in INCLUDE_DIRS {
        read_folder(i, 0, &mut output);
    }

    output.flush().expect("error writing to output file");
}

fn read_folder(
    path: impl AsRef<std::path::Path>,
    mut indent: usize,
    output: &mut std::fs::File
) {
    let path = path.as_ref();
    let name = path.file_name().unwrap().to_string_lossy();
    let tabs = "    ".repeat(indent);

    output.write_all(format!("{tabs}pub mod {name} {{\n").as_bytes()).expect("error writing to output file");

    indent += 1;
    for f in std::fs::read_dir(path).expect("error reading dir").filter_map(Result::ok) {
        let path = f.path();

        if path.is_dir() {
            read_folder(path, indent + 1, output);
            continue
        }

        let name = path.file_stem().unwrap();
        let const_name = name.to_ascii_uppercase();
        let const_name_display = const_name.to_string_lossy();
        let path = f.path().canonicalize().expect("error canonicalizing path");
        let path_display = path.display();

        let line = format!("pub const {const_name_display}:&[u8] = include_bytes!(\"{path_display}\");");
        
        let tabs = "    ".repeat(indent);
        output
            .write_all(format!("{tabs}{line}\n").as_bytes())
            .expect("error writing to output file");
    }
    
    output
        .write_all(format!("{tabs}}}\n").as_bytes())
        .expect("error writing to output file");
}
