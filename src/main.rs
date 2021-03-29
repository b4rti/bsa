use std::{env, error, ffi, fs, io};

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
    for folder in bsa.folders_mut() {
        if folder.name().is_some() {
            let folder_name = folder.name().unwrap().to_owned().clone();
            for file in folder.files_mut() {
                if let Some(file_name) = file.name() {
                    let combined_name = format!("{}\\{}", folder_name, file_name);
                    if path == combined_name {
                        io::copy(file.contents(), &mut io::stdout().lock())?;
                        return Ok(());
                    }
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
}

fn run() -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() < 2 {
        print_help();
        return Ok(());
    }
    match args[1].to_str() {
        Some("ls") => ls(&args[2])?,
        Some("cat") => cat(&args[2], args[3].to_str().unwrap())?,
        _ => print_help()
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
