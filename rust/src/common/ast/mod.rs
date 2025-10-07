pub mod checkers;
pub mod finders;
pub mod full_name;
pub mod providers;
pub mod transformer;
use pyo3::{
    PyResult,
    exceptions::{PyImportError, PyValueError},
};
use ruff_python_ast::{AtomicNodeIndex, Mod, ModModule, Stmt, StmtImportFrom, StmtPass};
use ruff_python_codegen::{Generator, Stylist};
use ruff_python_parser::{Mode, ParseError, ParseOptions, Parsed, parse};
use ruff_text_size::TextRange;

use crate::common::module_spec::get_parent_package;

pub fn parse_python_source(python_source: &str) -> Result<Mod, ParseError> {
    Ok(parse(python_source, ParseOptions::from(Mode::Module))?
        .syntax()
        .to_owned())
}

pub fn generate_source(
    body: &Vec<Stmt>,
    parsed: Parsed<ModModule>,
    original_source: &str,
) -> String {
    let stylist = Stylist::from_tokens(parsed.tokens(), original_source);
    let mut generator: Generator = (&stylist).into();
    generator.unparse_suite(body);
    let new_source = generator.stmt(&Stmt::Pass(StmtPass {
        range: TextRange::default(),
        node_index: AtomicNodeIndex::dummy(),
    }));
    return match new_source.strip_suffix("pass") {
        Some(s) => s.to_owned(),
        None => new_source,
    };
}

// does the same as libcst's resolve_name
fn resolve_name(name: &str, package: &str, level: &usize) -> PyResult<String> {
    if *level == 0 {
        return Ok(name.to_string());
    }

    let mut bits: Vec<&str> = package.rsplitn(*level, ".").collect();
    bits.reverse();

    if bits.len() < *level {
        return Err(PyImportError::new_err(format!(
            "attempted relative import beyond top-level package {} {}",
            &package, &level
        )));
    }
    let base = bits[0].to_string();
    if name.len() > 0 {
        return Ok(base + "." + name);
    }

    return Ok(base);
}

pub fn get_import_from_absolute_module_spec(
    node: &StmtImportFrom,
    parent_package: &str,
    greedy: bool,
) -> PyResult<Vec<String>> {
    let mut fixed_parent_package = parent_package.to_string();
    if fixed_parent_package.ends_with(".__main__") {
        fixed_parent_package = fixed_parent_package.replace(".__main__", "");
    }
    if node.module.is_none() && node.level == 0 {
        return Err(PyValueError::new_err(
            "No absolute module spec could be found for node",
        ));
    }
    if let Some(module) = &node.module {
        return Ok(vec![resolve_name(
            module,
            &fixed_parent_package,
            &usize::try_from(node.level).unwrap(),
        )?]);
    }

    if node.level > 0 {
        let mut target_package = fixed_parent_package.clone();
        for _ in 1..node.level {
            target_package = get_parent_package(&target_package);
        }
        let mut result = vec![target_package.clone()];
        if greedy {
            for name in &node.names {
                let name_str = name.name.to_string();
                if name_str != "*" {
                    result.push(format!("{}.{}", target_package, name_str));
                }
            }
        }
        return Ok(result);
    }

    Err(PyValueError::new_err("Don't know how to handle node"))
}
