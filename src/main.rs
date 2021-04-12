use std::{error, fs, io, path, process};

mod bsa;
mod cp1252;
mod hash;

fn setup_logger(verbose: bool) {
    let level = if verbose {
        log::LevelFilter::max()
    } else {
        log::LevelFilter::Off
    };
    pretty_env_logger::formatted_builder()
        .filter(None, level)
        .init();
}

fn ls(file: &path::Path) -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
    let bsa = bsa::open(file)?;
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

fn cat(
    bsa_file: &path::Path,
    path: &str,
) -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
    let path = if path.find('/').is_some() {
        path.replace('/', "\\")
    } else {
        path.to_string()
    };
    let mut bsa = bsa::open(bsa_file)?;
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
    bsa_file: &path::Path,
    into: Option<&path::Path>,
) -> Result<(), Box<dyn error::Error + Send + Sync + 'static>> {
    let mut bsa = bsa::open(bsa_file)?;
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
    let args = <Cli as structopt::StructOpt>::from_args();
    match args {
        Cli::Ls { file, verbose } => {
            setup_logger(verbose);
            ls(&file)?
        }
        Cli::Cat {
            file,
            path,
            verbose,
        } => {
            setup_logger(verbose);
            cat(&file, &path)?
        }
        Cli::Extract {
            file,
            into,
            verbose,
        } => {
            setup_logger(verbose);
            if let Some(into) = into {
                extract(&file, Some(&into))?
            } else {
                extract(&file, None)?
            }
        }
    }
    Ok(())
}

#[derive(structopt::StructOpt, Debug)]
enum Cli {
    /// List files in a BSA
    Ls {
        /// Input file
        #[structopt(parse(from_os_str))]
        file: path::PathBuf,
        /// Enable verbose output
        #[structopt(short, long)]
        verbose: bool,
    },
    /// Output a file from a BSA
    Cat {
        /// Input file
        #[structopt(parse(from_os_str))]
        file: path::PathBuf,
        /// Path to file in the BSA
        path: String,
        /// Enable verbose output
        #[structopt(short, long)]
        verbose: bool,
    },
    /// Extract all files from a BSA
    Extract {
        /// Input file
        #[structopt(parse(from_os_str))]
        file: path::PathBuf,
        /// Directory to extract into
        #[structopt(parse(from_os_str), long)]
        into: Option<path::PathBuf>,
        /// Enable verbose output
        #[structopt(short, long)]
        verbose: bool,
    },
}

fn print_error_chain(mut err: &dyn error::Error) {
    eprint!("{}", err);
    while let Some(inner) = err.source() {
        eprint!(": {}", inner);
        err = inner;
    }
    eprintln!();
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => {
            print_error_chain(err.as_ref());
            process::exit(1);
        }
    }
}
