use std::{collections::HashSet, fs::read_to_string, path::Path};

use logos::Logos;

use crate::{
    arena::Arena,
    error::Error,
    parser::Parser,
    structure::{
        ast::{Instruction, InstructionKind},
        token::Token,
    },
};

pub(crate) struct Importer<'a> {
    arena: &'a Arena<'a>,
    main_module: &'a Path,

    program: Vec<Instruction<'a>>,
    parsed: HashSet<&'a Path>,
}

impl<'a> Importer<'a> {
    pub(crate) fn new(arena: &'a Arena<'a>, main_module: &'a Path) -> Self {
        Self {
            arena,
            main_module,

            program: Vec::new(),
            parsed: HashSet::new(),
        }
    }

    pub(crate) fn resolve(mut self) -> Result<Vec<Instruction<'a>>, Error> {
        self.resolve_imports(self.main_module, None)?;

        Ok(self.program)
    }

    fn resolve_imports(
        &mut self,
        current_module: &'a Path,
        current_prefix: Option<&str>,
    ) -> Result<(), Error> {
        if self.parsed.contains(current_module) {
            return Ok(());
        }

        self.parsed.insert(current_module);

        let source_code = self
            .arena
            .dyn_alloc(read_to_string(current_module)?)
            .as_str();
        let instructions = Parser::new(self.arena, Token::lexer(source_code)).parse()?;

        for mut inst in instructions {
            if let InstructionKind::Import { path, alias } = inst.kind {
                self.resolve_imports(path, Some(alias))?;
                continue;
            }

            if let Some(prefix) = current_prefix {
                inst.add_module_prefix(self.arena, prefix);
            }

            self.program.push(inst);
        }

        Ok(())
    }
}
