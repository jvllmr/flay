use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use pyo3::{pyclass, pymethods};
use ruff_python_ast::{
    Decorator, Expr, ExprAttribute, ExprCompare, Stmt,
    visitor::{Visitor, walk_expr, walk_stmt},
};
use thin_vec::ThinVec;

use crate::common::ast::{
    full_name::get_full_name_for_expr, get_import_from_absolute_module_spec,
    providers::fully_qualified_name_provider::FullyQualifiedNameProvider,
};
use crate::common::cache::load_module;
use crate::common::fs_cache;
use crate::constants::PYTHON_MINOR_VERSION;

use super::nodes_remover::NodesRemover;

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
    // stays in rust; it is handed over to the nodes remover without crossing
    // the python boundary
    references_counts: HashMap<String, usize>,
    #[pyo3(get)]
    new_references_count: usize,
    always_bump_context: bool,
    source_path: PathBuf,
    import_aliases: HashMap<String, String>,
    safe_decorators: HashSet<String>,
    preserve_symbols: HashSet<String>,
    cache_dir: Option<PathBuf>,
}

#[pymethods]
impl ReferencesCounter {
    #[new]
    fn new(
        preserve_symbols: HashSet<String>,
        import_aliases: HashMap<String, String>,
        safe_decorators: HashSet<String>,
        cache_dir: Option<PathBuf>,
    ) -> Self {
        let mut references_counts: HashMap<String, usize> = HashMap::new();
        for symbol in &preserve_symbols {
            references_counts.insert(symbol.clone(), 1);
        }
        ReferencesCounter {
            module_spec: String::new(),
            names_provider: FullyQualifiedNameProvider::new("", &PathBuf::from("")),
            references_counts,
            always_bump_context: false,
            new_references_count: 0,
            source_path: PathBuf::new(),
            import_aliases: import_aliases,
            safe_decorators: safe_decorators,
            preserve_symbols,
            cache_dir,
        }
    }

    fn reset_counter(&mut self) {
        self.new_references_count = 0;
    }

    /// Move the counted references into a nodes remover without transferring
    /// the counts to python
    fn into_nodes_remover(
        &mut self,
        known_modules: HashSet<String>,
        cache_dir: Option<PathBuf>,
    ) -> NodesRemover {
        log::debug!(
            target: "flay.treeshake.package",
            "Counted references: {:?}",
            self.references_counts
        );
        NodesRemover::new(
            std::mem::take(&mut self.references_counts),
            known_modules,
            cache_dir,
        )
    }

    /// Load the final reference counts from the persistent cache when nothing
    /// they depend on changed. Returns true on a hit.
    fn load_cached_counts(&mut self, modules: Vec<(String, PathBuf)>) -> bool {
        let Some(key) = counts_cache_key(
            &modules,
            &self.import_aliases,
            &self.preserve_symbols,
            &self.safe_decorators,
        ) else {
            return false;
        };
        let Some(blob) = fs_cache::get_counted(self.cache_dir.as_deref(), &key) else {
            return false;
        };
        match fs_cache::parse_str_map(&blob) {
            Some(counts) => {
                self.references_counts = counts;
                true
            }
            None => false,
        }
    }

    /// Store the final reference counts in the persistent cache
    fn store_counts_cache(&self, modules: Vec<(String, PathBuf)>) {
        if let Some(key) = counts_cache_key(
            &modules,
            &self.import_aliases,
            &self.preserve_symbols,
            &self.safe_decorators,
        ) {
            fs_cache::put(
                self.cache_dir.as_deref(),
                &key,
                &fs_cache::serialize_str_map(&self.references_counts),
            );
        }
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

        let entry = load_module(&self.source_path)?;
        let module = entry.parsed.syntax();
        self.names_provider.seed_module_names(&module.body);
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

    fn has_unsafe_decorator(&mut self, decorators: &ThinVec<Decorator>) -> bool {
        for decorator in decorators {
            if !self.is_safe_decorator(decorator) {
                return true;
            }
        }
        false
    }

    /// A string annotation like `Optional["SomeType"]` holds a forward
    /// reference; the referenced name must stay alive
    fn bump_string_annotation_references(&mut self, annotation: &Expr) {
        let mut names: Vec<String> = Vec::new();
        collect_string_annotation_reference_names(annotation, &mut names);
        for name in names {
            for fqn in self.names_provider.resolve_fully_qualified_name(&name) {
                self.increase(&fqn);
            }
        }
    }
}

/// Collect names referenced from string forward references inside an
/// annotation. Unquoted names are referenced by the regular walk and must
/// not be collected here.
fn collect_string_annotation_reference_names(expr: &Expr, names: &mut Vec<String>) {
    match expr {
        Expr::StringLiteral(literal) => {
            if let Ok(parsed) = ruff_python_parser::parse(
                literal.value.to_str(),
                ruff_python_parser::ParseOptions::from(ruff_python_parser::Mode::Expression),
            ) {
                if let ruff_python_ast::Mod::Expression(inner) = parsed.syntax() {
                    collect_annotation_reference_names(&inner.body, names);
                }
            }
        }
        Expr::Subscript(subscript) => {
            collect_string_annotation_reference_names(&subscript.value, names);
            collect_string_annotation_reference_names(&subscript.slice, names);
        }
        Expr::Tuple(tuple) => {
            for element in &tuple.elts {
                collect_string_annotation_reference_names(element, names);
            }
        }
        Expr::List(list) => {
            for element in &list.elts {
                collect_string_annotation_reference_names(element, names);
            }
        }
        Expr::BinOp(bin_op) => {
            collect_string_annotation_reference_names(&bin_op.left, names);
            collect_string_annotation_reference_names(&bin_op.right, names);
        }
        _ => {}
    }
}

/// Collect all names of a parsed forward reference expression, following
/// nested string annotations recursively
fn collect_annotation_reference_names(expr: &Expr, names: &mut Vec<String>) {
    match expr {
        Expr::StringLiteral(literal) => {
            if let Ok(parsed) = ruff_python_parser::parse(
                literal.value.to_str(),
                ruff_python_parser::ParseOptions::from(ruff_python_parser::Mode::Expression),
            ) {
                if let ruff_python_ast::Mod::Expression(inner) = parsed.syntax() {
                    collect_annotation_reference_names(&inner.body, names);
                }
            }
        }
        Expr::Subscript(subscript) => {
            collect_annotation_reference_names(&subscript.value, names);
            collect_annotation_reference_names(&subscript.slice, names);
        }
        Expr::Tuple(tuple) => {
            for element in &tuple.elts {
                collect_annotation_reference_names(element, names);
            }
        }
        Expr::List(list) => {
            for element in &list.elts {
                collect_annotation_reference_names(element, names);
            }
        }
        Expr::BinOp(bin_op) => {
            collect_annotation_reference_names(&bin_op.left, names);
            collect_annotation_reference_names(&bin_op.right, names);
        }
        Expr::Name(_) | Expr::Attribute(_) => {
            names.extend(get_full_name_for_expr(expr));
        }
        _ => {}
    }
}

fn is_if_name_main(expr: &Expr) -> bool {
    if let Expr::Compare(ExprCompare {
        ops: cmp_ops,
        operands,
        ..
    }) = expr
    {
        // NOTE: make more readable?
        if cmp_ops.len() == 1
            && cmp_ops[0].is_eq()
            && operands.len() == 2
            && ((operands[0]
                .as_name_expr()
                .is_some_and(|name| name.id.as_str() == "__name__")
                && operands[1]
                    .as_string_literal_expr()
                    .is_some_and(|c| c.value == *"__main__"))
                || (operands[0]
                    .as_name_expr()
                    .is_some_and(|name| name.id.as_str() == "__name__")
                    && operands[1]
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
            Stmt::AnnAssign(ann_assign) => {
                self.bump_string_annotation_references(&ann_assign.annotation);
                if self.has_references_for_stmt(&stmt) {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                }
            }
            Stmt::AugAssign(_) => {
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
                // forward references in string annotations reference real names
                for parameter_with_default in func_def
                    .parameters
                    .posonlyargs
                    .iter()
                    .chain(func_def.parameters.args.iter())
                    .chain(func_def.parameters.kwonlyargs.iter())
                {
                    if let Some(annotation) = &parameter_with_default.parameter.annotation {
                        self.bump_string_annotation_references(annotation);
                    }
                }
                if let Some(vararg) = &func_def.parameters.vararg {
                    if let Some(annotation) = &vararg.annotation {
                        self.bump_string_annotation_references(annotation);
                    }
                }
                if let Some(kwarg) = &func_def.parameters.kwarg {
                    if let Some(annotation) = &kwarg.annotation {
                        self.bump_string_annotation_references(annotation);
                    }
                }
                if let Some(returns) = &func_def.returns {
                    self.bump_string_annotation_references(returns);
                }
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
                // default values and annotations are evaluated in the
                // enclosing scope, before the parameter names exist; visit
                // them before the scope is entered so the parameters do not
                // shadow their names
                for default in func_def
                    .parameters
                    .iter_non_variadic_params()
                    .filter_map(|parameter| parameter.default.as_deref())
                {
                    self.visit_expr(default);
                }
                for parameter in func_def.parameters.iter().map(|p| p.as_parameter()) {
                    if let Some(annotation) = &parameter.annotation {
                        self.visit_annotation(annotation);
                    }
                }
            }

            Stmt::If(if_block) => {
                if self.is_global_scope() && is_if_name_main(&if_block.test) {
                    self.maybe_increase_stmt(&stmt);
                    self.always_bump_context = true;
                } else if self.is_global_scope() && self.module_spec_has_references() {
                    // keep the bump context for the body walk below: the nodes
                    // remover keeps whole if blocks, so definitions inside
                    // them (e.g. classes with base classes) must stay referenced
                    self.always_bump_context = true;
                    self.visit_expr(&if_block.test);
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
                    // a name rebound at module level does not re-export the import
                    let module_fqn = format!("{}.{}", self.module_spec, defined_name);
                    let is_module_local = self
                        .names_provider
                        .get_locality_provider()
                        .is_module_local(defined_name);
                    for fqn in self.names_provider.resolve_fully_qualified_name(
                        &self.names_provider.resolve_qualified_name(&defined_name),
                    ) {
                        if fqn == module_fqn && is_module_local {
                            continue;
                        }
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
                    // a name rebound at module level does not re-export the import
                    let module_fqn = format!("{}.{}", self.module_spec, defined_name);
                    let is_module_local = self
                        .names_provider
                        .get_locality_provider()
                        .is_module_local(defined_name);
                    for fqn in self.names_provider.resolve_fully_qualified_name(
                        &self.names_provider.resolve_qualified_name(&defined_name),
                    ) {
                        if fqn == module_fqn && is_module_local {
                            continue;
                        }
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

/// Aggregate cache key covering every input of the reference counting
/// fixpoint: the ordered (file name, module spec, content hash) of each
/// visited module plus all configuration. The reference counter results are
/// not a pure function of a single module (they depend on the global counts),
/// so a change in any one file — a leaf included — changes its content hash
/// and busts the whole key, which then propagates to every module above it.
fn counts_cache_key(
    modules: &[(String, PathBuf)],
    import_aliases: &HashMap<String, String>,
    preserve_symbols: &HashSet<String>,
    safe_decorators: &HashSet<String>,
) -> Option<String> {
    let mut sorted_aliases: Vec<String> = import_aliases
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    sorted_aliases.sort_unstable();
    let mut sorted_preserve: Vec<&str> = preserve_symbols.iter().map(|s| s.as_str()).collect();
    sorted_preserve.sort_unstable();
    let mut sorted_safe: Vec<&str> = safe_decorators.iter().map(|s| s.as_str()).collect();
    sorted_safe.sort_unstable();

    let mut parts: Vec<Vec<u8>> = vec![
        PYTHON_MINOR_VERSION.to_string().into_bytes(),
        sorted_aliases.join("\n").into_bytes(),
        sorted_preserve.join("\n").into_bytes(),
        sorted_safe.join("\n").into_bytes(),
    ];
    for (module_spec, path) in modules {
        let file_name = path.file_name()?.to_str()?.as_bytes().to_vec();
        let content = std::fs::read(path).ok()?;
        let content_hash = fs_cache::hash_parts(&[&content]);
        parts.push(file_name);
        parts.push(module_spec.clone().into_bytes());
        parts.push(content_hash.into_bytes());
    }
    let part_refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
    Some(fs_cache::hash_parts(&part_refs))
}
