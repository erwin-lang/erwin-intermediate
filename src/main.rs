use std::{env::args, path::Path};

use crate::{
    arena::Arena, assembler::AssemblerAMD64, checker::Checker, error::Error, importer::Importer,
};

mod arena;
mod assembler;
mod checker;
mod error;
mod importer;
mod parser;
mod structure;

fn main() -> Result<(), Error> {
    let args = args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(Error::Custom("Usage: eil <file>".to_string()));
    }

    let main_module = Path::new(&args[1]).canonicalize()?;
    let arena = Box::leak(Box::new(Arena::new()));

    let importer = Importer::new(arena, &main_module);
    let program = importer.resolve()?;

    let checker = Checker::new(arena, &program);
    let (symbols, resolutions) = checker.check()?;

    let assembler = AssemblerAMD64::new(arena, &program, &symbols, &resolutions);
    let assembly_output = assembler.assemble()?;

    print!("{:#?}", assembly_output);

    Ok(())
}
