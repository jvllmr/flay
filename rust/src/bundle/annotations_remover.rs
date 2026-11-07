use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use pyo3::{pyclass, pymethods};
use ruff_python_ast::visitor::{Visitor, walk_stmt as walk_visitor_stmt};
use ruff_python_ast::{Expr, Stmt, StmtAnnAssign, StmtAssign, StmtClassDef, StmtFunctionDef};

use crate::common::ast::{
    generate_source,
    providers::fully_qualified_name_provider::FullyQualifiedNameProvider,
    transformer::{Transformer, walk_body, walk_stmt},
};
use crate::common::cache::load_module;
use crate::common::fs_cache;

/// Removes type annotations from a module. Classes that belong to an ecosystem
/// which reads annotations at runtime, like dataclasses or pydantic, keep their
/// annotations.
#[pyclass]
pub struct AnnotationsRemover {
    annotation_ecosystem_names: HashSet<String>,
    cache_dir: Option<PathBuf>,
    names_provider: FullyQualifiedNameProvider,
    preserved_classes: Vec<bool>,
    /// names of classes whose annotations are preserved, so subclasses of
    /// preserved classes are preserved as well
    preserved_class_names: HashSet<String>,
}

/// Collects all class definitions of a module, including nested ones
#[derive(Default)]
struct ClassCollector {
    classes: Vec<StmtClassDef>,
}

impl<'a> Visitor<'a> for ClassCollector {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let Stmt::ClassDef(class_def) = stmt {
            self.classes.push(class_def.clone());
        }
        walk_visitor_stmt(self, stmt);
    }
}

#[pymethods]
impl AnnotationsRemover {
    #[new]
    fn new(annotation_ecosystem_names: HashSet<String>, cache_dir: Option<PathBuf>) -> Self {
        AnnotationsRemover {
            annotation_ecosystem_names,
            cache_dir,
            names_provider: FullyQualifiedNameProvider::new("", &PathBuf::new()),
            preserved_classes: Vec::new(),
            preserved_class_names: HashSet::new(),
        }
    }

    /// Scan all given modules and record every class that preserves its
    /// annotations (ecosystem classes and their subclasses, transitively).
    /// Must be called once before remove_annotations_in_file so subclass
    /// propagation works regardless of file processing order.
    fn precompute_preserved_classes(&mut self, modules: Vec<(String, PathBuf)>) {
        let mut changed = true;
        while changed {
            changed = false;
            for (module_spec, source_path) in &modules {
                let Ok(entry) = load_module(source_path) else {
                    continue;
                };
                self.names_provider = FullyQualifiedNameProvider::new(module_spec, source_path);
                let module = entry.parsed.syntax();
                for stmt in &module.body {
                    self.names_provider.visit_stmt(stmt);
                }
                let mut collector = ClassCollector::default();
                for stmt in &module.body {
                    collector.visit_stmt(stmt);
                }
                for class_def in &collector.classes {
                    if self.class_preserves_annotations(class_def) {
                        for fqn in self
                            .names_provider
                            .get_stmt_fully_qualified_name(&Stmt::ClassDef(class_def.clone()))
                        {
                            if self.preserved_class_names.insert(fqn) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Remove type annotations from a module file and write the result to
    /// another file. The result is cached on disk keyed by the content hash
    /// of the source, so unchanged files are not even parsed again on repeat
    /// runs.
    fn remove_annotations_in_file(
        &mut self,
        source_path: PathBuf,
        target_path: PathBuf,
        module_spec: String,
    ) -> Result<(), std::io::Error> {
        let raw_source = fs::read(&source_path)?;
        let mut ecosystems: Vec<&str> = self
            .annotation_ecosystem_names
            .iter()
            .map(|name| name.as_str())
            .collect();
        ecosystems.sort_unstable();
        let cache_key = fs_cache::hash_parts(&[
            &raw_source,
            ecosystems.join("\n").as_bytes(),
            module_spec.as_bytes(),
        ]);
        if let Some(cached) = fs_cache::get_counted(self.cache_dir.as_deref(), &cache_key) {
            fs::write(&target_path, cached)?;
            return Ok(());
        }
        // fresh name resolution per module; provider state must not leak
        // between the modules of a bundle. the module spec is needed so
        // relative imports resolve to the real package paths.
        // preserved_class_names is kept across files so that subclassing a
        // preserved class propagates (see precompute_preserved_classes)
        self.names_provider = FullyQualifiedNameProvider::new(&module_spec, &source_path);
        self.preserved_classes.clear();
        let entry = load_module(&source_path)?;
        let parsed = entry.parsed.clone();
        let module = parsed.syntax();
        let new_body = self.visit_body(&module.body);
        let new_source = generate_source(&new_body, &parsed, &entry.source);
        fs::write(&target_path, &new_source)?;
        fs_cache::put(self.cache_dir.as_deref(), &cache_key, new_source.as_bytes());
        Ok(())
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
    /// Annotated metadata arguments are evaluated at runtime and may carry
    /// side effects (e.g. clonf.CliOption()) or be read back via
    /// __annotations__, so Annotated annotations are never removed
    fn uses_annotated(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Name(_) | Expr::Attribute(_) => self
                .names_provider
                .get_expr_fully_qualified_name(expr)
                .iter()
                .any(|fqn| fqn == "typing.Annotated" || fqn == "typing_extensions.Annotated"),
            Expr::StringLiteral(literal) => literal.value.to_str().contains("Annotated"),
            Expr::Subscript(subscript) => {
                self.uses_annotated(&subscript.value) || self.uses_annotated(&subscript.slice)
            }
            Expr::Tuple(tuple) => tuple.elts.iter().any(|elt| self.uses_annotated(elt)),
            Expr::List(list) => list.elts.iter().any(|elt| self.uses_annotated(elt)),
            Expr::BinOp(bin_op) => {
                self.uses_annotated(&bin_op.left) || self.uses_annotated(&bin_op.right)
            }
            Expr::Call(call) => {
                self.uses_annotated(&call.func)
                    || call
                        .arguments
                        .args
                        .iter()
                        .any(|arg| self.uses_annotated(arg))
                    || call
                        .arguments
                        .keywords
                        .iter()
                        .any(|kw| self.uses_annotated(&kw.value))
            }
            _ => false,
        }
    }

    /// Ecosystems like clonf read function annotations at runtime
    /// (clonf_click inspects __annotations__ of the decorated function)
    fn function_preserves_annotations(&self, func_def: &StmtFunctionDef) -> bool {
        func_def.decorator_list.iter().any(|decorator| {
            self.names_provider
                .get_expr_fully_qualified_name(&decorator.expression)
                .iter()
                .any(|fqn| self.annotation_ecosystem_names.contains(fqn))
        })
    }

    fn visit_function_def(&mut self, mut func_def: StmtFunctionDef) -> Stmt {
        let annotated = func_def
            .returns
            .as_deref()
            .is_some_and(|expr| self.uses_annotated(expr))
            || func_def.parameters.iter().any(|parameter| {
                parameter
                    .annotation()
                    .is_some_and(|expr| self.uses_annotated(expr))
            });
        if !self.function_preserves_annotations(&func_def) && !annotated {
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
        // Annotated metadata is evaluated at runtime and may carry side
        // effects, so the annotation is kept
        if self.uses_annotated(&ann_assign.annotation) {
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
