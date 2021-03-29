use std::{env, error, ffi, fs, io, path};

mod bsa;

fn ls(file: &ffi::OsStr) -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
    let file = fs::File::open(file)?;
    let bsa = bsa::Bsa::read(file)?;
    for folder in bsa.folders() {
        if let Some(folder_name) = folder.name() {
            for file in folder.files() {
                if let Some(file_name) = file.name() {
                    println!("{}\\{}", folder_name, file_name);
                }
            }
        }
    }
    Ok(())
}

fn cat(bsa_file: &ffi::OsStr, path: &str) -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
    let path = if path.find('/').is_some() {
        path.replace('/', "\\")
    } else {
        path.to_string()
    };
    let file = fs::File::open(bsa_file)?;
    let mut bsa = bsa::Bsa::read(file)?;
    for folder in bsa.folders() {
        if folder.name().is_some() {
            let folder_name = folder.name().unwrap();
            for file in folder.files() {
                if let Some(file_name) = file.name() {
                    let combined_name = format!("{}\\{}", folder_name, file_name);
                    if path == combined_name {
                        io::copy(
                            &mut file.clone().read_contents(&mut bsa)?,
                            &mut io::stdout().lock())?;
                        return Ok(());
                    }
                }
            }
        }
    }
    eprintln!("File {} does not exist in {}", path, bsa_file.to_string_lossy());
    Ok(())
}

fn extract(bsa_file: &ffi::OsStr, into: Option<&ffi::OsStr>) -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
    let file = fs::File::open(bsa_file)?;
    let mut bsa = bsa::Bsa::read(file)?;
    for folder in bsa.folders() {
        if folder.name().is_some() {
            let folder_name = folder.name().unwrap();
            let mut concat_folder = if let Some(into) = into {
                path::PathBuf::from(into)
            } else {
                path::PathBuf::new()
            };
            for folder_part in folder_name.split('\\') {
                concat_folder.push(folder_part);
            }
            fs::create_dir_all(&concat_folder)?;
            for file in folder.files() {
                if let Some(file_name) = file.name() {
                    let mut file_path = concat_folder.clone();
                    file_path.push(file_name);
                    let mut output_file = fs::File::create(&file_path)?;
                    println!("Creating {:?}", &file_path);
                    io::copy(
                        &mut file.clone().read_contents(&mut bsa)?,
                        &mut output_file)?;
                }
            }
        }
    }
    Ok(())
}

fn print_help() {
    eprintln!("Usage:");
    eprintln!("  bsa ls <file.bsa>");
    eprintln!("  bsa cat <file.bsa> <path>");
    eprintln!("  bsa extract <file.bsa> [into/this/path]");
}

fn run() -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() < 2 {
        print_help();
        return Ok(());
    }
    let string_args: Vec<String> = env::args().collect();
    let str_args: Vec<&str> = string_args.iter().map(|s| s.as_str()).collect();
    match str_args.as_slice() {
        ["ls", _] => ls(&args[2])?,
        ["cat", _, _] => cat(&args[2], args[3].to_str().unwrap())?,
        ["extract", _] => extract(&args[2], None)?,
        ["extract", _, _] => extract(&args[2], Some(&args[3]))?,
        _ => print_help(),
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
