use std::collections::HashSet;

use pyo3::{pyclass, pymethods};
use ruff_python_ast::{Stmt, StmtAnnAssign, StmtAssign, StmtClassDef, StmtFunctionDef};
use ruff_python_parser::parse_module;

use crate::common::ast::{
    generate_source,
    providers::fully_qualified_name_provider::FullyQualifiedNameProvider,
    transformer::{Transformer, walk_body, walk_stmt},
};

/// Removes type annotations from a module. Classes that belong to an ecosystem
/// which reads annotations at runtime, like dataclasses or pydantic, keep their
/// annotations.
#[pyclass]
pub struct AnnotationsRemover {
    annotation_ecosystem_names: HashSet<String>,
    names_provider: FullyQualifiedNameProvider,
    preserved_classes: Vec<bool>,
    /// names of classes whose annotations are preserved, so subclasses of
    /// preserved classes are preserved as well
    preserved_class_names: HashSet<String>,
}

#[pymethods]
impl AnnotationsRemover {
    #[new]
    fn new(annotation_ecosystem_names: HashSet<String>) -> Self {
        AnnotationsRemover {
            annotation_ecosystem_names,
            names_provider: FullyQualifiedNameProvider::new("", &std::path::PathBuf::new()),
            preserved_classes: Vec::new(),
            preserved_class_names: HashSet::new(),
        }
    }

    fn remove_annotations(&mut self, module_source: String) -> String {
        // fresh name resolution per module; provider state must not leak
        // between the modules of a bundle
        self.names_provider = FullyQualifiedNameProvider::new("", &std::path::PathBuf::new());
        self.preserved_classes.clear();
        self.preserved_class_names.clear();
        let parsed = parse_module(&module_source).unwrap();
        let module = parsed.syntax();
        let new_body = self.visit_body(&module.body);
        generate_source(&new_body, parsed, &module_source)
    }
}

impl AnnotationsRemover {
    fn class_preserves_annotations(&self, class_def: &StmtClassDef) -> bool {
        for decorator in &class_def.decorator_list {
            for fqn in self
                .names_provider
                .get_expr_fully_qualified_name(&decorator.expression)
            {
                if self.annotation_ecosystem_names.contains(&fqn)
                    || self.preserved_class_names.contains(&fqn)
                {
                    return true;
                }
            }
        }
        for base in class_def.bases() {
            for fqn in self.names_provider.get_expr_fully_qualified_name(base) {
                if self.annotation_ecosystem_names.contains(&fqn)
                    || self.preserved_class_names.contains(&fqn)
                {
                    return true;
                }
            }
        }
        return false;
    }
}

impl Transformer for AnnotationsRemover {
    fn visit_body(&mut self, body: &[Stmt]) -> thin_vec::ThinVec<Stmt> {
        let new_body = walk_body(self, body);
        if new_body.is_empty() && !body.is_empty() {
            // removing bare annotations can empty a class or function body;
            // an empty body is invalid python
            let mut with_pass = new_body;
            with_pass.push(Stmt::Pass(ruff_python_ast::StmtPass {
                range: ruff_text_size::TextRange::default(),
                node_index: ruff_python_ast::AtomicNodeIndex::default(),
            }));
            return with_pass;
        }
        return new_body;
    }

    fn visit_stmt(&mut self, stmt: Stmt) -> Option<Stmt> {
        self.names_provider.visit_stmt(&stmt);
        match stmt {
            Stmt::FunctionDef(func_def) => Some(self.visit_function_def(func_def)),
            Stmt::ClassDef(class_def) => {
                let preserves_annotations = self.class_preserves_annotations(&class_def);
                self.preserved_classes.push(preserves_annotations);
                if preserves_annotations {
                    for fqn in self
                        .names_provider
                        .get_stmt_fully_qualified_name(&Stmt::ClassDef(class_def.clone()))
                    {
                        self.preserved_class_names.insert(fqn);
                    }
                }
                let new_stmt = walk_stmt(self, Stmt::ClassDef(class_def));
                self.preserved_classes.pop();
                new_stmt
            }
            Stmt::AnnAssign(ann_assign) => self.visit_ann_assign(ann_assign),
            _ => walk_stmt(self, stmt),
        }
    }
}

impl AnnotationsRemover {
    fn visit_function_def(&mut self, mut func_def: StmtFunctionDef) -> Stmt {
        func_def.returns = None;
        let parameters = &mut func_def.parameters;
        for parameter_with_default in parameters
            .posonlyargs
            .iter_mut()
            .chain(parameters.args.iter_mut())
            .chain(parameters.kwonlyargs.iter_mut())
        {
            parameter_with_default.parameter.annotation = None;
        }
        if let Some(vararg) = &mut parameters.vararg {
            vararg.annotation = None;
        }
        if let Some(kwarg) = &mut parameters.kwarg {
            kwarg.annotation = None;
        }
        walk_stmt(self, Stmt::FunctionDef(func_def)).unwrap_or(Stmt::Pass(
            ruff_python_ast::StmtPass {
                range: ruff_text_size::TextRange::default(),
                node_index: ruff_python_ast::AtomicNodeIndex::default(),
            },
        ))
    }

    fn visit_ann_assign(&mut self, ann_assign: StmtAnnAssign) -> Option<Stmt> {
        // ecosystems like dataclasses read class level annotations at runtime
        if *self.preserved_classes.last().unwrap_or(&false) {
            return walk_stmt(self, Stmt::AnnAssign(ann_assign));
        }
        match ann_assign.value {
            // `x: int = 1` keeps its binding, the annotation is dropped
            Some(value) => walk_stmt(
                self,
                Stmt::Assign(StmtAssign {
                    range: ann_assign.range,
                    node_index: ann_assign.node_index,
                    targets: vec![*ann_assign.target],
                    value,
                }),
            ),
            // `x: int` without a value is a pure annotation, remove it entirely
            None => None,
        }
    }
}
