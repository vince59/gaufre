
use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("usage: gaufre <fichier.gfr> [out.wat]");
    let out_path = env::args().nth(2);
    let default_out = Path::new(&path).with_extension("wat").to_string_lossy().into_owned();
    let out = out_path.unwrap_or(default_out);
    let wat = "test";
    fs::write(&out, wat)?;
    let src = fs::read_to_string(&path)?;
    eprintln!("{}",src);
    Ok(())
}
