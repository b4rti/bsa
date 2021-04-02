use std::path::{Path, PathBuf};
use std::{error, fs, io, path};
use structopt::StructOpt;

mod bsa;
mod cp1252;

fn ls(file: &Path) -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
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

fn cat(bsa_file: &Path, path: &str) -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
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
                            &mut io::stdout().lock(),
                        )?;
                        return Ok(());
                    }
                }
            }
        }
    }
    eprintln!(
        "File {} does not exist in {}",
        path,
        bsa_file.to_string_lossy()
    );
    Ok(())
}

fn extract(
    bsa_file: &Path,
    into: Option<&Path>,
) -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
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
                    io::copy(&mut file.clone().read_contents(&mut bsa)?, &mut output_file)?;
                }
            }
        }
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
    let args = Cli::from_args();
    match args {
        Cli::Ls { file } => ls(&file)?,
        Cli::Cat { file, path } => cat(&file, &path)?,
        Cli::Extract { file, into } => {
            if let Some(into) = into {
                extract(&file, Some(&into))?
            } else {
                extract(&file, None)?
            }
        }
    }
    Ok(())
}

#[derive(StructOpt, Debug)]
enum Cli {
    Ls {
        /// Input file
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    Cat {
        /// Input file
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        /// Path to file in the bsa
        path: String,
    },
    Extract {
        /// Input file
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(parse(from_os_str), long = "into")]
        into: Option<PathBuf>,
    },
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
