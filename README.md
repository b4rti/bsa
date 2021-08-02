# bsa

A Rust library and CLI tool for working with Bethesda Software Archives (BSA files).

```bash
$ cargo install bsa
$ bsa ls 'Skyrim - Patch.bsa'
$ bsa extract 'Skyrim - Patch.bsa'
```

## CLI Usage:

```
USAGE:
    bsa <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    cat         Output a file from a BSA
    extract     Extract all files from a BSA
    help        Prints this message or the help of the given subcommand(s)
    ls          List files in a BSA
    validate    Validate BSA files
```

## Library Usage:

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
bsa = "0.1"
```

Then use the library like this:

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut bsa = bsa::open("file.bsa")?;
    for folder in bsa.folders() {
        for file in folder.files() {
            println!("File {:?} in folder {:?}", file.name(), folder.name());
            let contents = file.read_to_vec(&mut bsa)?;
            println!("{:?}", &contents);
        }
    }
    Ok(())
}
```
