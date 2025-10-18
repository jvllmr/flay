use std::collections::{HashMap, HashSet};

use ruff_python_ast::{Stmt, StmtImport, StmtImportFrom};

use crate::common::ast::{finders::find_dynamic_import, get_import_from_absolute_module_spec};

type TActiveImports = HashMap<String, String>;
type TActiveStarImports = HashSet<String>;

pub struct ImportTrackingProviderScope {
    pub active_imports: Option<TActiveImports>,
    pub active_star_imports: Option<TActiveStarImports>,
}

pub struct ImportsTrackingProvider {
    pub active_imports: TActiveImports,
    pub active_star_imports: TActiveStarImports,
    parent_package: String,
    importlib_module_alias: Option<String>,
}

impl ImportsTrackingProvider {
    pub fn new(parent_package: &str) -> Self {
        ImportsTrackingProvider {
            active_imports: HashMap::new(),
            active_star_imports: HashSet::new(),
            parent_package: parent_package.to_string(),
            importlib_module_alias: None,
        }
    }

    pub fn enter_scope(&self, stmt: &Stmt) -> ImportTrackingProviderScope {
        match stmt {
            Stmt::ClassDef(_) | Stmt::FunctionDef(_) => ImportTrackingProviderScope {
                active_imports: Some(self.active_imports.clone()),
                active_star_imports: Some(self.active_star_imports.clone()),
            },
            _ => ImportTrackingProviderScope {
                active_imports: None,
                active_star_imports: None,
            },
        }
    }

    pub fn exit_scope(&mut self, scope: ImportTrackingProviderScope) {
        if let Some(active_imports) = scope.active_imports {
            self.active_imports.clear();
            self.active_imports.extend(active_imports);
        }
        if let Some(active_star_imports) = scope.active_star_imports {
            self.active_star_imports.clear();
            self.active_star_imports.extend(active_star_imports);
        }
    }

    fn visit_import_from(&mut self, import_from: &StmtImportFrom) {
        if import_from.level == 0
            && import_from
                .module
                .as_ref()
                .is_some_and(|x| x == "importlib")
        {
            self.importlib_module_alias = Some("importlib".to_string())
        }

        let module_specs =
            match get_import_from_absolute_module_spec(import_from, &self.parent_package, false) {
                Ok(spec) => spec,
                Err(_) => vec![],
            };

        for module_spec in &module_specs {
            for name in &import_from.names {
                if name.name.as_str() == "*" {
                    self.active_star_imports.insert(module_spec.to_string());
                } else if let Some(asname) = &name.asname {
                    self.active_imports
                        .insert(asname.to_string(), format!("{}.{}", module_spec, name.name));
                } else {
                    self.active_imports.insert(
                        name.name.to_string(),
                        format!("{}.{}", module_spec, name.name),
                    );
                }
            }
        }
    }

    fn visit_import(&mut self, import: &StmtImport) {
        for name in &import.names {
            if let Some(asname) = &name.asname {
                self.active_imports
                    .insert(asname.to_string(), name.name.to_string());
            } else {
                self.active_imports
                    .insert(name.name.to_string(), name.name.to_string());
            }
            if name.name.as_str() == "importlib" {
                self.importlib_module_alias = name
                    .asname
                    .as_ref()
                    .or(Some(&name.name))
                    .map(|x| x.to_string())
            }
        }
    }

    pub fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Import(import) => {
                self.visit_import(import);
            }
            Stmt::ImportFrom(import_from) => {
                self.visit_import_from(&import_from);
            }
            _ => {
                for (full_name, module_spec) in
                    find_dynamic_import(stmt, self.importlib_module_alias.as_ref())
                {
                    self.active_imports.insert(full_name, module_spec);
                }
            }
        }
    }
}
