mod lexer;
mod parser;
mod grammar;

use lexer::Lexer;
use parser::Parser;
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

fn resolve_rel(base_file: &Path, rel: &str) -> PathBuf {
    let base_dir = base_file.parent().unwrap_or_else(|| Path::new("."));
    base_dir.join(rel)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_path = PathBuf::from(
        env::args()
            .nth(1)
            .expect("usage: gaufre <root.gfr> [out.wat]"),
    );
    let out_path = env::args().nth(2);

    // 1) main program parsing : imports + fn main { ... }
    let src_root = fs::read_to_string(&root_path)?;
    let lx_root = Lexer::with_file(root_path.to_string_lossy(), &src_root);
    let mut p = Parser::new(lx_root)?;
    let (imports, root_prog) = p.parse_main_program()?; // Program { stmts }

    // 2) Load every import (no import in these files)
    let mut imported_stmts = Vec::new();
    let mut seen = HashSet::new(); 
    for rel in imports {
        let full = resolve_rel(&root_path, &rel); // build import full path from rel path
        if !seen.insert(full.clone()) { // remove import duplicates 
            continue;
        }
        let src = fs::read_to_string(&full)?;
        let lx = Lexer::with_file(full.to_string_lossy(), &src); // new lexer for the import
        let mut p = Parser::new(lx)?;
        let mut part = p.parse_sub_programs()?; // parse import 
        imported_stmts.append(&mut part);
    }

    // 3) WAT code generation
    let wat = "test";

    let default_out = root_path.with_extension("wat");
    let out = out_path.unwrap_or_else(|| default_out.to_string_lossy().into_owned());
    fs::write(&out, wat)?;
    Ok(())
}
