use std::{env, error, fs, io::Read};

mod bsa;

fn foo() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    let mut file = fs::File::open(&args[1])?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    let bsa = bsa::Bsa::read(&data)?;
    println!("{:#?}", bsa);
    Ok(())
}

fn main() {
    match foo() {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
