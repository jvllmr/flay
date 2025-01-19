use rustpython_ast::{
    Alias, Arg, ArgWithDefault, Arguments, ExceptHandler, ExceptHandlerExceptHandler, Expr,
    Keyword, MatchCase, Pattern, PatternMatchAs, PatternMatchClass, PatternMatchMapping,
    PatternMatchOr, PatternMatchSequence, PatternMatchSingleton, PatternMatchStar,
    PatternMatchValue, Stmt, StmtAnnAssign, StmtAssert, StmtAssign, StmtAsyncFor,
    StmtAsyncFunctionDef, StmtAsyncWith, StmtAugAssign, StmtBreak, StmtClassDef, StmtContinue,
    StmtDelete, StmtExpr, StmtFor, StmtFunctionDef, StmtGlobal, StmtIf, StmtImport, StmtImportFrom,
    StmtMatch, StmtNonlocal, StmtPass, StmtRaise, StmtReturn, StmtTry, StmtTryStar, StmtTypeAlias,
    StmtWhile, StmtWith, TypeParam, TypeParamParamSpec, TypeParamTypeVar, TypeParamTypeVarTuple,
    WithItem,
};

fn box_expr_option(expr: Option<Expr>) -> Option<Box<Expr>> {
    expr.map(|value| Box::new(value))
}
#[allow(unused_mut)]
pub trait Transformer {
    fn visit_stmt_vec(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt> {
        let mut new_stmts: Vec<Stmt> = Vec::new();

        for stmt in stmts {
            if let Some(new_stmt) = self.visit_stmt(stmt) {
                new_stmts.push(new_stmt);
            }
        }

        return new_stmts;
    }

    fn visit_stmt(&mut self, mut stmt: Stmt) -> Option<Stmt> {
        self.generic_visit_stmt(stmt)
    }

    fn generic_visit_stmt(&mut self, mut stmt: Stmt) -> Option<Stmt> {
        match stmt {
            Stmt::Delete(del) => self
                .visit_stmt_delete(del)
                .map(|new_stmt| Stmt::Delete(new_stmt)),
            Stmt::Assert(assert) => self
                .visit_stmt_assert(assert)
                .map(|new_stmt| Stmt::Assert(new_stmt)),
            Stmt::AnnAssign(ann_assign) => self
                .visit_stmt_ann_assign(ann_assign)
                .map(|new_stmt| Stmt::AnnAssign(new_stmt)),
            Stmt::For(for_) => self
                .visit_stmt_for(for_)
                .map(|new_stmt| Stmt::For(new_stmt)),
            Stmt::AsyncFor(async_for) => self
                .visit_stmt_async_for(async_for)
                .map(|new_stmt| Stmt::AsyncFor(new_stmt)),
            Stmt::FunctionDef(func) => self
                .visit_stmt_function_def(func)
                .map(|new_stmt| Stmt::FunctionDef(new_stmt)),
            Stmt::AsyncFunctionDef(async_func) => self
                .visit_stmt_async_function_def(async_func)
                .map(|new_stmt| Stmt::AsyncFunctionDef(new_stmt)),
            Stmt::AsyncWith(async_with) => self
                .visit_stmt_async_with(async_with)
                .map(|new_stmt| Stmt::AsyncWith(new_stmt)),
            Stmt::With(with) => self
                .visit_stmt_with(with)
                .map(|new_stmt| Stmt::With(new_stmt)),
            Stmt::Break(break_) => self
                .visit_stmt_break(break_)
                .map(|new_stmt| Stmt::Break(new_stmt)),
            Stmt::Pass(pass) => self
                .visit_stmt_pass(pass)
                .map(|new_stmt| Stmt::Pass(new_stmt)),
            Stmt::Continue(continue_) => self
                .visit_stmt_continue(continue_)
                .map(|new_stmt| Stmt::Continue(new_stmt)),
            Stmt::Return(return_) => self
                .visit_stmt_return(return_)
                .map(|new_stmt| Stmt::Return(new_stmt)),
            Stmt::Raise(raise) => self
                .visit_stmt_raise(raise)
                .map(|new_stmt| Stmt::Raise(new_stmt)),
            Stmt::ClassDef(stmt_class_def) => self
                .visit_stmt_class_def(stmt_class_def)
                .map(|new_stmt| Stmt::ClassDef(new_stmt)),
            Stmt::Assign(stmt_assign) => self
                .visit_stmt_assign(stmt_assign)
                .map(|new_stmt| Stmt::Assign(new_stmt)),
            Stmt::TypeAlias(stmt_type_alias) => self
                .visit_stmt_type_alias(stmt_type_alias)
                .map(|new_stmt| Stmt::TypeAlias(new_stmt)),
            Stmt::AugAssign(stmt_aug_assign) => self
                .visit_stmt_aug_assign(stmt_aug_assign)
                .map(|new_stmt| Stmt::AugAssign(new_stmt)),
            Stmt::While(stmt_while) => self
                .visit_stmt_while(stmt_while)
                .map(|new_stmt| Stmt::While(new_stmt)),
            Stmt::If(stmt_if) => self
                .visit_stmt_if(stmt_if)
                .map(|new_stmt| Stmt::If(new_stmt)),
            Stmt::Match(stmt_match) => self
                .visit_stmt_match(stmt_match)
                .map(|new_stmt| Stmt::Match(new_stmt)),
            Stmt::Try(stmt_try) => self
                .visit_stmt_try(stmt_try)
                .map(|new_stmt| Stmt::Try(new_stmt)),
            Stmt::TryStar(stmt_try_star) => self
                .visit_stmt_try_star(stmt_try_star)
                .map(|new_stmt| Stmt::TryStar(new_stmt)),
            Stmt::Import(stmt_import) => self
                .visit_stmt_import(stmt_import)
                .map(|new_stmt| Stmt::Import(new_stmt)),
            Stmt::ImportFrom(stmt_import_from) => self
                .visit_stmt_import_from(stmt_import_from)
                .map(|new_stmt| Stmt::ImportFrom(new_stmt)),
            Stmt::Global(stmt_global) => self
                .visit_stmt_global(stmt_global)
                .map(|new_stmt| Stmt::Global(new_stmt)),
            Stmt::Nonlocal(stmt_nonlocal) => self
                .visit_stmt_nonlocal(stmt_nonlocal)
                .map(|new_stmt| Stmt::Nonlocal(new_stmt)),
            Stmt::Expr(stmt_expr) => self
                .visit_stmt_expr(stmt_expr)
                .map(|new_stmt| Stmt::Expr(new_stmt)),
        }
    }

    fn generic_visit_keyword_vec(&mut self, mut keywords: Vec<Keyword>) -> Vec<Keyword> {
        let mut new_keywords = Vec::new();

        for keyword in keywords {
            if let Some(new_keyword) = self.visit_keyword(keyword) {
                new_keywords.push(new_keyword);
            }
        }
        new_keywords
    }

    fn visit_keyword(&mut self, mut keyword: Keyword) -> Option<Keyword> {
        self.generic_visit_keyword(keyword)
    }

    fn generic_visit_keyword(&mut self, mut keyword: Keyword) -> Option<Keyword> {
        keyword.value = self
            .visit_expr(keyword.value)
            .expect("Cannot remove value from keyword");

        Some(keyword)
    }

    fn visit_stmt_class_def(&mut self, mut stmt: StmtClassDef) -> Option<StmtClassDef> {
        self.generic_visit_stmt_class_def(stmt)
    }

    fn generic_visit_stmt_class_def(&mut self, mut stmt: StmtClassDef) -> Option<StmtClassDef> {
        stmt.decorator_list = self.visit_expr_vec(stmt.decorator_list);

        stmt.type_params = self.generic_visit_type_param_vec(stmt.type_params);
        stmt.bases = self.visit_expr_vec(stmt.bases);
        stmt.keywords = self.generic_visit_keyword_vec(stmt.keywords);
        stmt.body = self.visit_stmt_vec(stmt.body);

        if stmt.body.len() == 0 {
            panic!("Cannot remove body from class def")
        }

        Some(stmt)
    }

    fn visit_stmt_assign(&mut self, mut stmt: StmtAssign) -> Option<StmtAssign> {
        self.generic_visit_stmt_assign(stmt)
    }

    fn generic_visit_stmt_assign(&mut self, mut stmt: StmtAssign) -> Option<StmtAssign> {
        stmt.targets = self.visit_expr_vec(stmt.targets);
        if stmt.targets.len() == 0 {
            panic!("Cannot remove all targets from assignment")
        }
        stmt.value = Box::new(
            self.visit_expr(*stmt.value)
                .expect("Cannot remove value from assignment"),
        );

        Some(stmt)
    }

    fn visit_stmt_type_alias(&mut self, mut stmt: StmtTypeAlias) -> Option<StmtTypeAlias> {
        self.generic_visit_stmt_type_alias(stmt)
    }

    fn generic_visit_stmt_type_alias(&mut self, mut stmt: StmtTypeAlias) -> Option<StmtTypeAlias> {
        stmt.name = Box::new(
            self.visit_expr(*stmt.name)
                .expect("Cannot remove name from type alias"),
        );
        stmt.type_params = self.generic_visit_type_param_vec(stmt.type_params);
        stmt.value = Box::new(
            self.visit_expr(*stmt.value)
                .expect("Cannot remove value from type alias"),
        );

        Some(stmt)
    }

    fn visit_stmt_aug_assign(&mut self, mut stmt: StmtAugAssign) -> Option<StmtAugAssign> {
        self.generic_visit_stmt_aug_assign(stmt)
    }

    fn generic_visit_stmt_aug_assign(&mut self, mut stmt: StmtAugAssign) -> Option<StmtAugAssign> {
        stmt.value = Box::new(
            self.visit_expr(*stmt.value)
                .expect("Cannot remove value from augmented assignment"),
        );
        stmt.target = Box::new(
            self.visit_expr(*stmt.target)
                .expect("Cannot remove target from augmented assignment"),
        );

        Some(stmt)
    }

    fn visit_stmt_while(&mut self, mut stmt: StmtWhile) -> Option<StmtWhile> {
        self.generic_visit_stmt_while(stmt)
    }

    fn generic_visit_stmt_while(&mut self, mut stmt: StmtWhile) -> Option<StmtWhile> {
        stmt.test = Box::new(
            self.visit_expr(*stmt.test)
                .expect("Cannot remove test from while statement"),
        );
        stmt.body = self.visit_stmt_vec(stmt.body);
        stmt.orelse = self.visit_stmt_vec(stmt.orelse);

        if stmt.body.len() == 0 && stmt.orelse.len() == 0 {
            return None;
        }

        Some(stmt)
    }

    fn visit_stmt_if(&mut self, mut stmt: StmtIf) -> Option<StmtIf> {
        self.generic_visit_stmt_if(stmt)
    }

    fn generic_visit_stmt_if(&mut self, mut stmt: StmtIf) -> Option<StmtIf> {
        stmt.test = Box::new(
            self.visit_expr(*stmt.test)
                .expect("Cannot remove test from if statement"),
        );
        stmt.body = self.visit_stmt_vec(stmt.body);
        stmt.orelse = self.visit_stmt_vec(stmt.orelse);

        if stmt.body.len() == 0 && stmt.orelse.len() == 0 {
            return None;
        }

        Some(stmt)
    }

    fn visit_pattern_match_or(&mut self, mut pattern: PatternMatchOr) -> Option<PatternMatchOr> {
        self.generic_visit_pattern_match_or(pattern)
    }

    fn generic_visit_pattern_match_or(
        &mut self,
        mut pattern: PatternMatchOr,
    ) -> Option<PatternMatchOr> {
        pattern.patterns = self.generic_visit_pattern_vec(pattern.patterns);
        if pattern.patterns.len() == 0 {
            return None;
        }

        Some(pattern)
    }

    fn visit_pattern_match_as(&mut self, mut pattern: PatternMatchAs) -> Option<PatternMatchAs> {
        self.generic_visit_pattern_match_as(pattern)
    }

    fn generic_visit_pattern_match_as(
        &mut self,
        mut pattern: PatternMatchAs,
    ) -> Option<PatternMatchAs> {
        if let Some(inner_pattern) = pattern.pattern {
            pattern.pattern = self
                .visit_pattern(*inner_pattern)
                .map(|new_pattern| Box::new(new_pattern));
        }

        Some(pattern)
    }

    fn visit_pattern_match_mapping(
        &mut self,
        mut pattern: PatternMatchMapping,
    ) -> Option<PatternMatchMapping> {
        self.generic_visit_pattern_match_mapping(pattern)
    }

    fn generic_visit_pattern_match_mapping(
        &mut self,
        mut pattern: PatternMatchMapping,
    ) -> Option<PatternMatchMapping> {
        pattern.keys = self.visit_expr_vec(pattern.keys);
        pattern.patterns = self.generic_visit_pattern_vec(pattern.patterns);
        Some(pattern)
    }

    fn visit_pattern_match_star(
        &mut self,
        mut pattern: PatternMatchStar,
    ) -> Option<PatternMatchStar> {
        self.generic_visit_pattern_match_star(pattern)
    }

    fn generic_visit_pattern_match_star(
        &mut self,
        mut pattern: PatternMatchStar,
    ) -> Option<PatternMatchStar> {
        Some(pattern)
    }

    fn visit_pattern_match_class(
        &mut self,
        mut pattern: PatternMatchClass,
    ) -> Option<PatternMatchClass> {
        self.generic_visit_pattern_match_class(pattern)
    }

    fn generic_visit_pattern_match_class(
        &mut self,
        mut pattern: PatternMatchClass,
    ) -> Option<PatternMatchClass> {
        pattern.cls = Box::new(
            self.visit_expr(*pattern.cls)
                .expect("Cannot remove class from pattern match class"),
        );
        pattern.patterns = self.generic_visit_pattern_vec(pattern.patterns);
        pattern.kwd_patterns = self.generic_visit_pattern_vec(pattern.kwd_patterns);
        Some(pattern)
    }

    fn visit_pattern_match_sequence(
        &mut self,
        mut pattern: PatternMatchSequence,
    ) -> Option<PatternMatchSequence> {
        self.generic_visit_pattern_match_sequence(pattern)
    }

    fn generic_visit_pattern_match_sequence(
        &mut self,
        mut pattern: PatternMatchSequence,
    ) -> Option<PatternMatchSequence> {
        pattern.patterns = self.generic_visit_pattern_vec(pattern.patterns);
        if pattern.patterns.len() == 0 {
            return None;
        }

        Some(pattern)
    }

    fn visit_pattern_match_singleton(
        &mut self,
        mut pattern: PatternMatchSingleton,
    ) -> Option<PatternMatchSingleton> {
        self.generic_visit_pattern_match_singleton(pattern)
    }

    fn generic_visit_pattern_match_singleton(
        &mut self,
        mut pattern: PatternMatchSingleton,
    ) -> Option<PatternMatchSingleton> {
        Some(pattern)
    }

    fn visit_pattern_match_value(
        &mut self,
        mut pattern: PatternMatchValue,
    ) -> Option<PatternMatchValue> {
        self.generic_visit_pattern_match_value(pattern)
    }

    fn generic_visit_pattern_match_value(
        &mut self,
        mut pattern: PatternMatchValue,
    ) -> Option<PatternMatchValue> {
        pattern.value = Box::new(
            self.visit_expr(*pattern.value)
                .expect("Cannot remove value from pattern match value"),
        );
        Some(pattern)
    }

    fn generic_visit_pattern_vec(&mut self, patterns: Vec<Pattern>) -> Vec<Pattern> {
        let mut new_patterns: Vec<Pattern> = Vec::new();
        for pattern in patterns {
            if let Some(new_pattern) = self.visit_pattern(pattern) {
                new_patterns.push(new_pattern);
            }
        }

        new_patterns
    }

    fn visit_pattern(&mut self, pattern: Pattern) -> Option<Pattern> {
        self.generic_visit_pattern(pattern)
    }

    fn generic_visit_pattern(&mut self, pattern: Pattern) -> Option<Pattern> {
        match pattern {
            Pattern::MatchValue(pattern_match_value) => self
                .visit_pattern_match_value(pattern_match_value)
                .map(|new_pattern| Pattern::MatchValue(new_pattern)),
            Pattern::MatchSingleton(pattern_match_singleton) => self
                .visit_pattern_match_singleton(pattern_match_singleton)
                .map(|new_pattern| Pattern::MatchSingleton(new_pattern)),
            Pattern::MatchSequence(pattern_match_sequence) => self
                .visit_pattern_match_sequence(pattern_match_sequence)
                .map(|new_pattern| Pattern::MatchSequence(new_pattern)),
            Pattern::MatchMapping(pattern_match_mapping) => self
                .visit_pattern_match_mapping(pattern_match_mapping)
                .map(|new_pattern| Pattern::MatchMapping(new_pattern)),
            Pattern::MatchClass(pattern_match_class) => self
                .visit_pattern_match_class(pattern_match_class)
                .map(|new_pattern| Pattern::MatchClass(new_pattern)),
            Pattern::MatchStar(pattern_match_star) => self
                .visit_pattern_match_star(pattern_match_star)
                .map(|new_pattern| Pattern::MatchStar(new_pattern)),
            Pattern::MatchAs(pattern_match_as) => self
                .visit_pattern_match_as(pattern_match_as)
                .map(|new_pattern| Pattern::MatchAs(new_pattern)),
            Pattern::MatchOr(pattern_match_or) => self
                .visit_pattern_match_or(pattern_match_or)
                .map(|new_pattern| Pattern::MatchOr(new_pattern)),
        }
    }

    fn generic_visit_match_case_vec(&mut self, cases: Vec<MatchCase>) -> Vec<MatchCase> {
        let mut new_cases: Vec<MatchCase> = Vec::new();
        for case in cases {
            if let Some(new_case) = self.visit_match_case(case) {
                new_cases.push(new_case);
            }
        }

        new_cases
    }

    fn visit_match_case(&mut self, mut case: MatchCase) -> Option<MatchCase> {
        self.generic_visit_match_case(case)
    }

    fn generic_visit_match_case(&mut self, mut case: MatchCase) -> Option<MatchCase> {
        if let Some(guard) = case.guard {
            case.guard = box_expr_option(self.visit_expr(*guard));
        }

        case.body = self.visit_stmt_vec(case.body);
        if case.body.len() == 0 {
            return None;
        }
        Some(case)
    }

    fn visit_stmt_match(&mut self, mut stmt: StmtMatch) -> Option<StmtMatch> {
        self.generic_visit_stmt_match(stmt)
    }

    fn generic_visit_stmt_match(&mut self, mut stmt: StmtMatch) -> Option<StmtMatch> {
        stmt.subject = Box::new(
            self.visit_expr(*stmt.subject)
                .expect("Cannot remove subject from match statement"),
        );
        stmt.cases = self.generic_visit_match_case_vec(stmt.cases);
        if stmt.cases.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn generic_visit_except_handler_vec(
        &mut self,
        handlers: Vec<ExceptHandler>,
    ) -> Vec<ExceptHandler> {
        let mut new_handlers: Vec<ExceptHandler> = Vec::new();

        for handler in handlers {
            if let Some(new_handler) = self.visit_except_handler(handler) {
                new_handlers.push(new_handler);
            }
        }

        new_handlers
    }

    fn visit_except_handler(&mut self, mut handler: ExceptHandler) -> Option<ExceptHandler> {
        self.generic_visit_except_handler(handler)
    }

    fn generic_visit_except_handler(
        &mut self,
        mut handler: ExceptHandler,
    ) -> Option<ExceptHandler> {
        match handler {
            ExceptHandler::ExceptHandler(except_handler) => self
                .visit_except_handler_except_handler(except_handler)
                .map(|new_except_handler| ExceptHandler::ExceptHandler(new_except_handler)),
        }
    }

    fn visit_except_handler_except_handler(
        &mut self,
        mut except_handler: ExceptHandlerExceptHandler,
    ) -> Option<ExceptHandlerExceptHandler> {
        self.generic_visit_except_handler_except_handler(except_handler)
    }

    fn generic_visit_except_handler_except_handler(
        &mut self,
        mut except_handler: ExceptHandlerExceptHandler,
    ) -> Option<ExceptHandlerExceptHandler> {
        except_handler.body = self.visit_stmt_vec(except_handler.body);
        if except_handler.body.len() == 0 {
            return None;
        }
        Some(except_handler)
    }

    fn visit_stmt_try(&mut self, mut stmt: StmtTry) -> Option<StmtTry> {
        self.generic_visit_stmt_try(stmt)
    }

    fn generic_visit_stmt_try(&mut self, mut stmt: StmtTry) -> Option<StmtTry> {
        stmt.body = self.visit_stmt_vec(stmt.body);
        stmt.finalbody = self.visit_stmt_vec(stmt.finalbody);
        stmt.handlers = self.generic_visit_except_handler_vec(stmt.handlers);
        stmt.orelse = self.visit_stmt_vec(stmt.orelse);

        if stmt.body.len() == 0 {
            return None;
        }

        Some(stmt)
    }

    fn visit_stmt_try_star(&mut self, mut stmt: StmtTryStar) -> Option<StmtTryStar> {
        self.generic_visit_stmt_try_star(stmt)
    }

    fn generic_visit_stmt_try_star(&mut self, mut stmt: StmtTryStar) -> Option<StmtTryStar> {
        stmt.body = self.visit_stmt_vec(stmt.body);
        stmt.finalbody = self.visit_stmt_vec(stmt.finalbody);
        stmt.handlers = self.generic_visit_except_handler_vec(stmt.handlers);
        stmt.orelse = self.visit_stmt_vec(stmt.orelse);

        if stmt.body.len() == 0 {
            return None;
        }

        Some(stmt)
    }

    fn generic_visit_alias_vec(&mut self, aliases: Vec<Alias>) -> Vec<Alias> {
        let mut new_aliases: Vec<Alias> = Vec::new();

        for alias in aliases {
            if let Some(new_alias) = self.visit_alias(alias) {
                new_aliases.push(new_alias);
            }
        }

        return new_aliases;
    }

    fn visit_alias(&mut self, mut alias: Alias) -> Option<Alias> {
        self.generic_visit_alias(alias)
    }

    fn generic_visit_alias(&mut self, mut alias: Alias) -> Option<Alias> {
        Some(alias)
    }

    fn visit_stmt_import(&mut self, mut stmt: StmtImport) -> Option<StmtImport> {
        self.generic_visit_stmt_import(stmt)
    }

    fn generic_visit_stmt_import(&mut self, mut stmt: StmtImport) -> Option<StmtImport> {
        stmt.names = self.generic_visit_alias_vec(stmt.names);

        if stmt.names.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_stmt_import_from(&mut self, mut stmt: StmtImportFrom) -> Option<StmtImportFrom> {
        self.generic_visit_stmt_import_from(stmt)
    }

    fn generic_visit_stmt_import_from(
        &mut self,
        mut stmt: StmtImportFrom,
    ) -> Option<StmtImportFrom> {
        stmt.names = self.generic_visit_alias_vec(stmt.names);

        if stmt.names.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_stmt_global(&mut self, mut stmt: StmtGlobal) -> Option<StmtGlobal> {
        self.generic_visit_stmt_global(stmt)
    }

    fn generic_visit_stmt_global(&mut self, mut stmt: StmtGlobal) -> Option<StmtGlobal> {
        if stmt.names.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_stmt_nonlocal(&mut self, mut stmt: StmtNonlocal) -> Option<StmtNonlocal> {
        self.generic_visit_stmt_nonlocal(stmt)
    }

    fn generic_visit_stmt_nonlocal(&mut self, mut stmt: StmtNonlocal) -> Option<StmtNonlocal> {
        if stmt.names.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_stmt_expr(&mut self, mut stmt: StmtExpr) -> Option<StmtExpr> {
        self.generic_visit_stmt_expr(stmt)
    }

    fn generic_visit_stmt_expr(&mut self, mut stmt: StmtExpr) -> Option<StmtExpr> {
        match self.visit_expr(*stmt.value) {
            Some(new_expr) => {
                stmt.value = Box::new(new_expr);
                return Some(stmt);
            }
            None => None,
        }
    }

    fn visit_stmt_raise(&mut self, mut stmt: StmtRaise) -> Option<StmtRaise> {
        self.generic_visit_stmt_raise(stmt)
    }

    fn generic_visit_stmt_raise(&mut self, mut stmt: StmtRaise) -> Option<StmtRaise> {
        if let Some(exc) = stmt.exc {
            stmt.exc = box_expr_option(self.visit_expr(*exc));
        }

        if let Some(cause) = stmt.cause {
            stmt.cause = box_expr_option(self.visit_expr(*cause));
        }

        Some(stmt)
    }

    fn visit_stmt_return(&mut self, mut stmt: StmtReturn) -> Option<StmtReturn> {
        self.generic_visit_stmt_return(stmt)
    }

    fn generic_visit_stmt_return(&mut self, mut stmt: StmtReturn) -> Option<StmtReturn> {
        if let Some(value) = stmt.value {
            stmt.value = box_expr_option(self.visit_expr(*value));
        }

        Some(stmt)
    }

    fn visit_stmt_continue(&mut self, mut stmt: StmtContinue) -> Option<StmtContinue> {
        self.generic_visit_stmt_continue(stmt)
    }

    fn generic_visit_stmt_continue(&mut self, mut stmt: StmtContinue) -> Option<StmtContinue> {
        Some(stmt)
    }

    fn visit_stmt_pass(&mut self, mut stmt: StmtPass) -> Option<StmtPass> {
        self.generic_visit_stmt_pass(stmt)
    }

    fn generic_visit_stmt_pass(&mut self, mut stmt: StmtPass) -> Option<StmtPass> {
        Some(stmt)
    }

    fn visit_stmt_break(&mut self, mut stmt: StmtBreak) -> Option<StmtBreak> {
        self.generic_visit_stmt_break(stmt)
    }

    fn generic_visit_stmt_break(&mut self, mut stmt: StmtBreak) -> Option<StmtBreak> {
        Some(stmt)
    }

    fn visit_stmt_with(&mut self, mut stmt: StmtWith) -> Option<StmtWith> {
        self.generic_visit_stmt_with(stmt)
    }

    fn generic_visit_stmt_with(&mut self, mut stmt: StmtWith) -> Option<StmtWith> {
        stmt.items = self.generic_visit_with_item_vec(stmt.items);
        stmt.body = self.visit_stmt_vec(stmt.body);

        if stmt.body.len() == 0 {
            return None;
        }

        Some(stmt)
    }

    fn visit_stmt_async_with(&mut self, mut stmt: StmtAsyncWith) -> Option<StmtAsyncWith> {
        self.generic_visit_stmt_async_with(stmt)
    }

    fn generic_visit_stmt_async_with(&mut self, mut stmt: StmtAsyncWith) -> Option<StmtAsyncWith> {
        stmt.items = self.generic_visit_with_item_vec(stmt.items);
        stmt.body = self.visit_stmt_vec(stmt.body);
        if stmt.body.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_stmt_function_def(&mut self, mut stmt: StmtFunctionDef) -> Option<StmtFunctionDef> {
        self.generic_visit_stmt_function_def(stmt)
    }

    fn generic_visit_stmt_function_def(
        &mut self,
        mut stmt: StmtFunctionDef,
    ) -> Option<StmtFunctionDef> {
        stmt.type_params = self.generic_visit_type_param_vec(stmt.type_params);
        stmt.decorator_list = self.visit_expr_vec(stmt.decorator_list);
        stmt.args = Box::new(self.visit_arguments(*stmt.args));
        if let Some(returns) = stmt.returns {
            stmt.returns = box_expr_option(self.visit_expr(*returns));
        }
        stmt.body = self.visit_stmt_vec(stmt.body);
        if stmt.body.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_stmt_async_function_def(
        &mut self,
        mut stmt: StmtAsyncFunctionDef,
    ) -> Option<StmtAsyncFunctionDef> {
        self.generic_visit_stmt_async_function_def(stmt)
    }

    fn generic_visit_stmt_async_function_def(
        &mut self,
        mut stmt: StmtAsyncFunctionDef,
    ) -> Option<StmtAsyncFunctionDef> {
        stmt.type_params = self.generic_visit_type_param_vec(stmt.type_params);
        stmt.decorator_list = self.visit_expr_vec(stmt.decorator_list);
        stmt.args = Box::new(self.visit_arguments(*stmt.args));
        if let Some(returns) = stmt.returns {
            stmt.returns = box_expr_option(self.visit_expr(*returns));
        }
        stmt.body = self.visit_stmt_vec(stmt.body);
        if stmt.body.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_stmt_for(&mut self, mut stmt: StmtFor) -> Option<StmtFor> {
        self.generic_visit_for(stmt)
    }

    fn generic_visit_for(&mut self, mut stmt: StmtFor) -> Option<StmtFor> {
        stmt.body = self.visit_stmt_vec(stmt.body);
        stmt.iter = Box::new(
            self.visit_expr(*stmt.iter)
                .expect("Cannot remove iter from async for"),
        );
        stmt.orelse = self.visit_stmt_vec(stmt.orelse);
        stmt.target = Box::new(
            self.visit_expr(*stmt.target)
                .expect("Cannot remove target from async for"),
        );
        if stmt.body.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_stmt_async_for(&mut self, mut stmt: StmtAsyncFor) -> Option<StmtAsyncFor> {
        self.generic_visit_async_for(stmt)
    }

    fn generic_visit_async_for(&mut self, mut stmt: StmtAsyncFor) -> Option<StmtAsyncFor> {
        stmt.body = self.visit_stmt_vec(stmt.body);
        stmt.iter = Box::new(
            self.visit_expr(*stmt.iter)
                .expect("Cannot remove iter from async for"),
        );
        stmt.orelse = self.visit_stmt_vec(stmt.orelse);
        stmt.target = Box::new(
            self.visit_expr(*stmt.target)
                .expect("Cannot remove target from async for"),
        );
        if stmt.body.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_stmt_ann_assign(&mut self, mut stmt: StmtAnnAssign) -> Option<StmtAnnAssign> {
        self.generic_visit_ann_assign(stmt)
    }

    fn generic_visit_ann_assign(&mut self, mut stmt: StmtAnnAssign) -> Option<StmtAnnAssign> {
        stmt.annotation = Box::new(
            self.visit_expr(*stmt.annotation)
                .expect("Cannot remove annotation from annotated assignment"),
        );

        stmt.target = Box::new(
            self.visit_expr(*stmt.target)
                .expect("Cannot remove target from annotated assignment"),
        );

        if let Some(value) = stmt.value {
            stmt.value = box_expr_option(self.visit_expr(*value));
        }

        Some(stmt)
    }

    fn visit_stmt_assert(&mut self, mut stmt: StmtAssert) -> Option<StmtAssert> {
        self.generic_visit_assert(stmt)
    }

    fn generic_visit_assert(&mut self, mut stmt: StmtAssert) -> Option<StmtAssert> {
        if let Some(msg) = stmt.msg {
            stmt.msg = box_expr_option(self.visit_expr(*msg));
        }

        stmt.test = Box::new(
            self.visit_expr(*stmt.test)
                .expect("Assertion test cannot be removed"),
        );

        Some(stmt)
    }

    fn visit_stmt_delete(&mut self, mut stmt: StmtDelete) -> Option<StmtDelete> {
        self.generic_visit_delete(stmt)
    }

    fn generic_visit_delete(&mut self, mut stmt: StmtDelete) -> Option<StmtDelete> {
        stmt.targets = self.visit_expr_vec(stmt.targets);
        if stmt.targets.len() == 0 {
            return None;
        }
        Some(stmt)
    }

    fn visit_expr_vec(&mut self, exprs: Vec<Expr>) -> Vec<Expr> {
        let mut new_exprs: Vec<Expr> = Vec::new();

        for expr in exprs {
            if let Some(new_expr) = self.visit_expr(expr) {
                new_exprs.push(new_expr);
            }
        }

        return new_exprs;
    }

    fn visit_expr(&mut self, expr: Expr) -> Option<Expr> {
        self.generic_visit_expr(expr)
    }

    fn generic_visit_expr(&mut self, expr: Expr) -> Option<Expr> {
        match expr {
            Expr::BoolOp(expr_bool_op) => todo!(),
            Expr::NamedExpr(expr_named_expr) => todo!(),
            Expr::BinOp(expr_bin_op) => todo!(),
            Expr::UnaryOp(expr_unary_op) => todo!(),
            Expr::Lambda(expr_lambda) => todo!(),
            Expr::IfExp(expr_if_exp) => todo!(),
            Expr::Dict(expr_dict) => todo!(),
            Expr::Set(expr_set) => todo!(),
            Expr::ListComp(expr_list_comp) => todo!(),
            Expr::SetComp(expr_set_comp) => todo!(),
            Expr::DictComp(expr_dict_comp) => todo!(),
            Expr::GeneratorExp(expr_generator_exp) => todo!(),
            Expr::Await(expr_await) => todo!(),
            Expr::Yield(expr_yield) => todo!(),
            Expr::YieldFrom(expr_yield_from) => todo!(),
            Expr::Compare(expr_compare) => todo!(),
            Expr::Call(expr_call) => todo!(),
            Expr::FormattedValue(expr_formatted_value) => todo!(),
            Expr::JoinedStr(expr_joined_str) => todo!(),
            Expr::Constant(expr_constant) => todo!(),
            Expr::Attribute(expr_attribute) => todo!(),
            Expr::Subscript(expr_subscript) => todo!(),
            Expr::Starred(expr_starred) => todo!(),
            Expr::Name(expr_name) => todo!(),
            Expr::List(expr_list) => todo!(),
            Expr::Tuple(expr_tuple) => todo!(),
            Expr::Slice(expr_slice) => todo!(),
        }
    }

    fn visit_arg(&mut self, arg: Arg) -> Option<Arg> {
        self.generic_visit_arg(arg)
    }

    fn generic_visit_arg(&mut self, mut arg: Arg) -> Option<Arg> {
        if let Some(annotation) = arg.annotation {
            arg.annotation = box_expr_option(self.visit_expr(*annotation));
        }
        return Some(arg);
    }

    fn visit_arg_with_default(&mut self, mut arg: ArgWithDefault) -> Option<ArgWithDefault> {
        self.generic_visit_arg_with_default(arg)
    }

    fn generic_visit_arg_with_default(
        &mut self,
        mut arg: ArgWithDefault,
    ) -> Option<ArgWithDefault> {
        arg.def = self
            .visit_arg(arg.def)
            .expect("Cannot remove def from arg with default");
        if let Some(default) = arg.default {
            arg.default = box_expr_option(self.visit_expr(*default));
        }

        Some(arg)
    }

    fn generic_visit_args_with_default_vec(
        &mut self,
        mut node: Vec<ArgWithDefault>,
    ) -> Vec<ArgWithDefault> {
        let mut new_nodes: Vec<ArgWithDefault> = Vec::new();

        for arg in node {
            if let Some(new_arg) = self.generic_visit_arg_with_default(arg) {
                new_nodes.push(new_arg);
            }
        }
        return new_nodes;
    }

    fn visit_arguments(&mut self, mut arguments: Arguments) -> Arguments {
        self.generic_visit_arguments(arguments)
    }

    fn generic_visit_arguments(&mut self, mut arguments: Arguments) -> Arguments {
        arguments.args = self.generic_visit_args_with_default_vec(arguments.args);
        if let Some(kwarg) = arguments.kwarg {
            arguments.kwarg = self.visit_arg(*kwarg).map(|new_arg| Box::new(new_arg));
        }
        arguments.kwonlyargs = self.generic_visit_args_with_default_vec(arguments.kwonlyargs);
        arguments.posonlyargs = self.generic_visit_args_with_default_vec(arguments.posonlyargs);
        if let Some(vararg) = arguments.vararg {
            arguments.vararg = self.visit_arg(*vararg).map(|new_arg| Box::new(new_arg));
        }
        return arguments;
    }

    fn generic_visit_type_param_vec(&mut self, mut params: Vec<TypeParam>) -> Vec<TypeParam> {
        let mut new_params: Vec<TypeParam> = Vec::new();
        for param in params {
            if let Some(new_param) = self.visit_type_param(param) {
                new_params.push(new_param);
            }
        }
        return new_params;
    }

    fn visit_type_param(&mut self, mut param: TypeParam) -> Option<TypeParam> {
        self.generic_visit_type_param(param)
    }

    fn generic_visit_type_param(&mut self, mut param: TypeParam) -> Option<TypeParam> {
        match param {
            TypeParam::ParamSpec(param_spec) => self
                .visit_type_param_spec(param_spec)
                .map(|new_param| TypeParam::ParamSpec(new_param)),
            TypeParam::TypeVar(param_var) => self
                .visit_type_param_var(param_var)
                .map(|new_param| TypeParam::TypeVar(new_param)),
            TypeParam::TypeVarTuple(param_var_tuple) => self
                .visit_type_param_var_tuple(param_var_tuple)
                .map(|new_param| TypeParam::TypeVarTuple(new_param)),
        }
    }

    fn visit_type_param_spec(
        &mut self,
        mut param_spec: TypeParamParamSpec,
    ) -> Option<TypeParamParamSpec> {
        self.generic_visit_type_param_spec(param_spec)
    }

    fn generic_visit_type_param_spec(
        &mut self,
        mut param_spec: TypeParamParamSpec,
    ) -> Option<TypeParamParamSpec> {
        Some(param_spec)
    }

    fn visit_type_param_var(
        &mut self,
        mut param_var: TypeParamTypeVar,
    ) -> Option<TypeParamTypeVar> {
        self.generic_visit_type_param_var(param_var)
    }

    fn generic_visit_type_param_var(
        &mut self,
        mut param_var: TypeParamTypeVar,
    ) -> Option<TypeParamTypeVar> {
        if let Some(bound) = param_var.bound {
            param_var.bound = box_expr_option(self.visit_expr(*bound));
        }
        Some(param_var)
    }

    fn visit_type_param_var_tuple(
        &mut self,
        mut param_var_tuple: TypeParamTypeVarTuple,
    ) -> Option<TypeParamTypeVarTuple> {
        self.generic_visit_type_param_var_tuple(param_var_tuple)
    }

    fn generic_visit_type_param_var_tuple(
        &mut self,
        mut param_var_tuple: TypeParamTypeVarTuple,
    ) -> Option<TypeParamTypeVarTuple> {
        Some(param_var_tuple)
    }

    fn generic_visit_with_item_vec(&mut self, with_items: Vec<WithItem>) -> Vec<WithItem> {
        let mut new_with_items: Vec<WithItem> = Vec::new();

        for with_item in with_items {
            if let Some(new_with_item) = self.visit_with_item(with_item) {
                new_with_items.push(new_with_item);
            }
        }

        return new_with_items;
    }

    fn visit_with_item(&mut self, mut with_item: WithItem) -> Option<WithItem> {
        self.generic_visit_with_item(with_item)
    }

    fn generic_visit_with_item(&mut self, mut with_item: WithItem) -> Option<WithItem> {
        with_item.context_expr = self
            .visit_expr(with_item.context_expr)
            .expect("Cannot remove context expr from with item");
        if let Some(optional_vars) = with_item.optional_vars {
            with_item.optional_vars = box_expr_option(self.visit_expr(*optional_vars));
        }
        Some(with_item)
    }
}
