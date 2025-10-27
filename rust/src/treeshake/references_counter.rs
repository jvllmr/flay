use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use pyo3::{pyclass, pymethods};
use ruff_python_ast::{
    Decorator, Expr, ExprAttribute, ExprCompare, Stmt,
    visitor::{Visitor, walk_expr, walk_stmt},
};

use crate::common::ast::{
    get_import_from_absolute_module_spec, parse_python_source,
    providers::fully_qualified_name_provider::FullyQualifiedNameProvider,
};

pub trait ReferencesHolder {
    fn get_references_counts(&self) -> &HashMap<String, usize>;
    fn get_names_provider(&self) -> &FullyQualifiedNameProvider;
    fn get_source_path(&self) -> &PathBuf;
    fn get_module_spec(&self) -> &String;

    fn module_spec_has_references(&self) -> bool {
        let references_counts = self.get_references_counts();
        let module_spec = self.get_module_spec();
        for (key, count) in references_counts {
            if key.starts_with(module_spec) && *count > 0 {
                return true;
            }
        }
        false
    }

    fn has_references_for_str(&self, str_: &str) -> bool {
        let references_counts = self.get_references_counts();
        // TODO: ??? this looks wrong; someone with more rust xp please help
        return references_counts.get(str_).unwrap_or(&(0 as usize)) > &0;
    }

    fn has_references_for_expr(&self, expr: &Expr) -> bool {
        let names_provider = self.get_names_provider();
        let fqns = names_provider.get_expr_fully_qualified_name(expr);
        for fqn in fqns {
            if self.has_references_for_str(&fqn) {
                return true;
            }
        }
        false
    }

    fn has_references_for_stmt(&self, stmt: &Stmt) -> bool {
        let names_provider = self.get_names_provider();

        let fqns: Vec<String> = match stmt {
            Stmt::ImportFrom(import_from) => {
                if import_from.names.len() == 1 && import_from.names[0].name.as_str() == "*" {
                    if let Ok(module_specs) = get_import_from_absolute_module_spec(
                        import_from,
                        &self
                            .get_names_provider()
                            .get_imports_provider()
                            .get_parent_package(),
                        true,
                    ) {
                        module_specs
                    } else {
                        Vec::new()
                    }
                } else {
                    names_provider.get_stmt_fully_qualified_name(stmt)
                }
            }
            _ => names_provider.get_stmt_fully_qualified_name(stmt),
        };

        for fqn in fqns {
            if self.has_references_for_str(&fqn) {
                return true;
            }
        }
        false
    }

    fn is_global_scope(&self) -> bool {
        self.get_names_provider().name_context.len() == 0
    }
}

#[pyclass]
pub struct ReferencesCounter {
    names_provider: FullyQualifiedNameProvider,
    module_spec: String,
    #[pyo3(get, set)]
    references_counts: HashMap<String, usize>,
    #[pyo3(get, set)]
    new_references_count: usize,
    always_bump_context: bool,
    source_path: PathBuf,
    import_aliases: HashMap<String, String>,
    safe_decorators: HashSet<String>,
}

#[pymethods]
impl ReferencesCounter {
    #[new]
    fn new(
        references_counts: HashMap<String, usize>,
        import_aliases: HashMap<String, String>,
        safe_decorators: HashSet<String>,
    ) -> Self {
        ReferencesCounter {
            module_spec: String::new(),
            names_provider: FullyQualifiedNameProvider::new("", &PathBuf::from("")),
            references_counts,
            always_bump_context: false,
            new_references_count: 0,
            source_path: PathBuf::new(),
            import_aliases: import_aliases,
            safe_decorators: safe_decorators,
        }
    }

    fn reset_counter(&mut self) {
        self.new_references_count = 0;
    }

    fn visit_module(
        &mut self,
        module_spec: String,
        source_path: PathBuf,
    ) -> Result<(), std::io::Error> {
        self.always_bump_context = false;
        self.module_spec = module_spec;
        self.source_path = source_path;
        self.names_provider =
            FullyQualifiedNameProvider::new(&self.module_spec, self.get_source_path());

        let file_content = fs::read_to_string(&self.source_path)?;
        let module = parse_python_source(&file_content).unwrap().expect_module();
        for stmt in &module.body {
            self.visit_stmt(stmt);
        }
        Ok(())
    }
}

impl ReferencesCounter {
    fn increase(&mut self, fqn: &str) {
        let old_references_count = self.references_counts.get(fqn);

        match old_references_count {
            Some(count) => {
                self.references_counts.insert(fqn.to_string(), count + 1);
            }
            None => {
                self.references_counts.insert(fqn.to_string(), 1);
                self.new_references_count += 1;
            }
        }
        if let Some(alias) = self.import_aliases.get(fqn).cloned() {
            self.increase(&alias);
        }
    }

    fn maybe_increase_stmt_selective<F>(&mut self, stmt: &Stmt, predicate: F)
    where
        F: Fn(&str) -> bool,
    {
        for fqn in self.names_provider.get_stmt_fully_qualified_name(stmt) {
            if predicate(&fqn) {
                self.increase(&fqn);
            }
        }
    }

    fn maybe_increase_stmt(&mut self, stmt: &Stmt) {
        self.maybe_increase_stmt_selective(stmt, |_| true);
    }

    fn maybe_increase_expr(&mut self, expr: &Expr) {
        for fqn in self.names_provider.get_expr_fully_qualified_name(expr) {
            self.increase(&fqn);
        }
    }

    fn make_known(&mut self, fqn: &str) {
        if !self.references_counts.contains_key(fqn) {
            self.references_counts.insert(fqn.to_owned(), 0);
            self.new_references_count += 1;
        }
    }

    fn make_known_stmt(&mut self, stmt: &Stmt) {
        for fqn in self.names_provider.get_stmt_fully_qualified_name(stmt) {
            self.make_known(&fqn);
        }
    }

    fn is_safe_decorator(&mut self, decorator: &Decorator) -> bool {
        for fqn in self
            .names_provider
            .get_expr_fully_qualified_name(&decorator.expression)
        {
            if self.safe_decorators.contains(&fqn) {
                return true;
            }
        }
        false
    }

    fn has_unsafe_decorator(&mut self, decorators: &Vec<Decorator>) -> bool {
        for decorator in decorators {
            if !self.is_safe_decorator(decorator) {
                return true;
            }
        }
        false
    }
}

fn is_if_name_main(expr: &Expr) -> bool {
    if let Expr::Compare(ExprCompare {
        ops: cmp_ops,
        left,
        comparators,
        ..
    }) = expr
    {
        // NOTE: make more readable?
        if cmp_ops.len() == 1
            && cmp_ops[0].is_eq()
            && comparators.len() == 1
            && ((left
                .as_name_expr()
                .is_some_and(|name| name.id.as_str() == "__name__")
                && comparators[0]
                    .as_string_literal_expr()
                    .is_some_and(|c| c.value == *"__main__"))
                || (comparators[0]
                    .as_name_expr()
                    .is_some_and(|name| name.id.as_str() == "__name__")
                    && left
                        .as_string_literal_expr()
                        .is_some_and(|c| c.value == *"__main__")))
        {
            return true;
        }
    }

    return false;
}

impl ReferencesHolder for ReferencesCounter {
    fn get_module_spec(&self) -> &String {
        &self.module_spec
    }

    fn get_source_path(&self) -> &PathBuf {
        &self.source_path
    }
    fn get_names_provider(&self) -> &FullyQualifiedNameProvider {
        &self.names_provider
    }

    fn get_references_counts(&self) -> &HashMap<String, usize> {
        &self.references_counts
    }
}

impl Visitor<'_> for ReferencesCounter {
    fn visit_stmt(&mut self, stmt: &ruff_python_ast::Stmt) {
        // everything in __main__.py should be preserved
        if self
            .source_path
            .file_name()
            .is_some_and(|file_name| file_name == "__main__.py")
        {
            self.maybe_increase_stmt(&stmt);
        }
        let can_reset_context = !self.always_bump_context;
        if self.always_bump_context {
            self.maybe_increase_stmt(&stmt);
        }
        self.make_known_stmt(&stmt);

        match &stmt {
            Stmt::AnnAssign(_) | Stmt::AugAssign(_) => {
                if self.has_references_for_stmt(&stmt) {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
            }
            Stmt::Assign(stmt_assign) => {
                let mut should_bump_stmt_assign = false;
                if self.is_global_scope() && self.module_spec_has_references() {
                    for target in &stmt_assign.targets {
                        let mut search_target = target;
                        let mut found_deepest_attribute: Option<ExprAttribute> = None;
                        while let Expr::Attribute(attr) = search_target {
                            search_target = &attr.value;
                            found_deepest_attribute = Some(attr.to_owned());
                        }

                        if let Some(deepest_attribute) = found_deepest_attribute {
                            if self.has_references_for_expr(&deepest_attribute.value) {
                                should_bump_stmt_assign = true;
                            }
                        }
                    }
                }
                if self.has_references_for_stmt(&stmt) || should_bump_stmt_assign {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
            }
            Stmt::ClassDef(class_def) => {
                if self.has_unsafe_decorator(&class_def.decorator_list)
                    || self.has_references_for_stmt(&stmt)
                {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
                // visit decorators, bases and keywords before they are prefixed with scope
                for decorator in &class_def.decorator_list {
                    self.visit_decorator(decorator);
                }
                for base in class_def.bases() {
                    self.visit_expr(base);
                }
                for keyword in class_def.keywords() {
                    self.visit_keyword(keyword);
                }
            }
            Stmt::For(_) => {
                if self.is_global_scope() && self.module_spec_has_references() {
                    self.always_bump_context = true;
                }
            }
            Stmt::FunctionDef(func_def) => {
                if self.has_unsafe_decorator(&func_def.decorator_list)
                    || self.has_references_for_stmt(&stmt)
                {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                    // visit decorators before they are prefixed with scope
                    for decorator in &func_def.decorator_list {
                        self.visit_decorator(&decorator);
                    }
                }
                // respect pep562 by preserving __getattr__ and __dir__ on module level
                if self.is_global_scope()
                    && self.module_spec_has_references()
                    && (func_def.name.as_str() == "__dir__"
                        || func_def.name.as_str() == "__getattr__")
                {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
            }

            Stmt::If(if_block) => {
                if self.is_global_scope() && is_if_name_main(&if_block.test) {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                } else if self.is_global_scope() && self.module_spec_has_references() {
                    self.always_bump_context = true;
                    self.visit_expr(&if_block.test);
                    self.always_bump_context = false;
                }
            }
            Stmt::Import(stmt_import) => {
                // check if one of the names defined by this import was imported somewhere else
                // if yes, bump reference of this import
                for alias in &stmt_import.names {
                    let defined_name = if let Some(alias_value) = &alias.asname {
                        alias_value
                    } else {
                        &alias.name
                    };
                    for fqn in self.names_provider.resolve_fully_qualified_name(
                        &self.names_provider.resolve_qualified_name(&defined_name),
                    ) {
                        if self.has_references_for_str(&fqn) {
                            self.maybe_increase_stmt_selective(&stmt, |n| n == alias.name.as_str());
                        }
                    }
                }
            }

            Stmt::ImportFrom(stmt_import_from) => {
                // check if one of the names defined by this import was imported somewhere else
                // if yes, bump reference of this import
                for alias in &stmt_import_from.names {
                    let defined_name = if let Some(alias_value) = &alias.asname {
                        alias_value
                    } else {
                        &alias.name
                    };
                    for fqn in self.names_provider.resolve_fully_qualified_name(
                        &self.names_provider.resolve_qualified_name(&defined_name),
                    ) {
                        if !fqn.starts_with("__builtin__") && self.has_references_for_str(&fqn) {
                            self.maybe_increase_stmt_selective(&stmt, |n| {
                                n.ends_with(alias.name.as_str())
                            });
                        }
                    }
                }
                // for a top-level star import we find out what names were imported from there
                // and check if that name has active references
                // if yes, bump the original name
                if self.is_global_scope()
                    && stmt_import_from.names.len() == 1
                    && stmt_import_from.names[0].name.as_str() == "*"
                {
                    if let Ok(module_specs) = get_import_from_absolute_module_spec(
                        &stmt_import_from,
                        &self
                            .names_provider
                            .get_imports_provider()
                            .get_parent_package(),
                        true,
                    ) {
                        for module_spec in module_specs {
                            let mut new_names: Vec<String> = Vec::new();
                            let mut bump_names: Vec<String> = Vec::new();
                            for (reference, _) in &self.references_counts {
                                if reference.len() > module_spec.len()
                                    // && !reference.ends_with("*")
                                    && reference.starts_with(&module_spec)
                                {
                                    let imported_name =
                                        (&reference[module_spec.len() + 1..]).to_owned();
                                    let exported_name =
                                        format!("{}.{}", self.module_spec, imported_name);

                                    if self
                                        .references_counts
                                        .get(&exported_name)
                                        .is_some_and(|n| *n > 0)
                                    {
                                        bump_names.push(reference.to_owned());
                                    }
                                    new_names.push(exported_name);
                                }
                            }

                            for name in new_names {
                                self.make_known(&name);
                            }
                            for name in bump_names {
                                self.increase(&name);
                            }
                        }
                    }
                }
            }
            _ => {}
        };

        let scope = self.names_provider.enter_scope(&stmt);
        self.names_provider.visit_stmt(stmt);
        walk_stmt(self, stmt);
        if can_reset_context {
            self.always_bump_context = false;
        }
        self.names_provider.exit_scope(scope);
    }

    fn visit_expr(&mut self, expr: &ruff_python_ast::Expr) {
        // everything in __main__.py should be preserved
        if self
            .source_path
            .file_name()
            .is_some_and(|file_name| file_name == "__main__.py")
        {
            self.maybe_increase_expr(&expr);
        }
        let can_reset_context = !self.always_bump_context;
        if self.always_bump_context {
            self.maybe_increase_expr(&expr);
        };

        match expr {
            Expr::Call(_) => {
                if self.is_global_scope() && self.module_spec_has_references() {
                    self.maybe_increase_expr(&expr);
                    self.always_bump_context = true;
                }
            }
            _ => {}
        }

        walk_expr(self, expr);
        if can_reset_context {
            self.always_bump_context = false;
        }
    }
}
