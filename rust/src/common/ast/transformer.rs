use ruff_python_ast::{
    self as ast, Alias, Arguments, BoolOp, BytesLiteral, BytesLiteralValue, CmpOp, Comprehension,
    Decorator, DictItem, ElifElseClause, ExceptHandler, Expr, ExprContext, FString, FStringPart,
    FStringValue, InterpolatedStringElement, InterpolatedStringElements, Keyword, MatchCase,
    Operator, Parameter, ParameterWithDefault, Parameters, Pattern, PatternArguments,
    PatternKeyword, Stmt, StringLiteral, StringLiteralValue, TString, TStringPart, TStringValue,
    TypeParam, TypeParams, UnaryOp, WithItem,
};

fn box_expr_option(expr: Option<Expr>) -> Option<Box<Expr>> {
    expr.map(Box::new)
}

/// A trait for transforming ASTs. Visits all nodes in the AST recursively in evaluation-order.
/// This is essentially a copy of ruff's transformer trait but with...
///  ...mutable passing of the transformer
///  ...owned node values
///  ...optionals as return values to control own removal
#[allow(unused_mut)]
pub trait Transformer {
    fn visit_stmt(&mut self, mut stmt: Stmt) -> Option<Stmt> {
        walk_stmt(self, stmt)
    }
    fn visit_annotation(&mut self, mut expr: Expr) -> Option<Expr> {
        walk_annotation(self, expr)
    }
    fn visit_decorator(&mut self, mut decorator: Decorator) -> Option<Decorator> {
        walk_decorator(self, decorator)
    }
    fn visit_expr(&mut self, mut expr: Expr) -> Option<Expr> {
        walk_expr(self, expr)
    }
    fn visit_expr_context(&mut self, mut expr_context: ExprContext) -> Option<ExprContext> {
        Some(walk_expr_context(self, expr_context))
    }
    fn visit_bool_op(&mut self, mut bool_op: BoolOp) -> Option<BoolOp> {
        Some(walk_bool_op(self, bool_op))
    }
    fn visit_operator(&mut self, mut operator: Operator) -> Option<Operator> {
        Some(walk_operator(self, operator))
    }
    fn visit_unary_op(&mut self, mut unary_op: UnaryOp) -> Option<UnaryOp> {
        Some(walk_unary_op(self, unary_op))
    }
    fn visit_cmp_op(&mut self, mut cmp_op: CmpOp) -> Option<CmpOp> {
        Some(walk_cmp_op(self, cmp_op))
    }
    fn visit_comprehension(&mut self, mut comprehension: Comprehension) -> Option<Comprehension> {
        Some(walk_comprehension(self, comprehension))
    }
    fn visit_except_handler(&mut self, mut except_handler: ExceptHandler) -> Option<ExceptHandler> {
        Some(walk_except_handler(self, except_handler))
    }
    fn visit_arguments(&mut self, mut arguments: Arguments) -> Option<Arguments> {
        Some(walk_arguments(self, arguments))
    }
    fn visit_parameters(&mut self, mut parameters: Parameters) -> Option<Parameters> {
        Some(walk_parameters(self, parameters))
    }
    fn visit_parameter(&mut self, mut parameter: Parameter) -> Option<Parameter> {
        Some(walk_parameter(self, parameter))
    }
    fn visit_keyword(&mut self, mut keyword: Keyword) -> Option<Keyword> {
        Some(walk_keyword(self, keyword))
    }
    fn visit_alias(&mut self, mut alias: Alias) -> Option<Alias> {
        Some(walk_alias(self, alias))
    }
    fn visit_with_item(&mut self, mut with_item: WithItem) -> Option<WithItem> {
        Some(walk_with_item(self, with_item))
    }
    fn visit_type_params(&mut self, mut type_params: TypeParams) -> Option<TypeParams> {
        Some(walk_type_params(self, type_params))
    }
    fn visit_type_param(&mut self, mut type_param: TypeParam) -> Option<TypeParam> {
        Some(walk_type_param(self, type_param))
    }
    fn visit_match_case(&mut self, mut match_case: MatchCase) -> Option<MatchCase> {
        Some(walk_match_case(self, match_case))
    }
    fn visit_pattern(&mut self, mut pattern: Pattern) -> Option<Pattern> {
        Some(walk_pattern(self, pattern))
    }
    fn visit_pattern_arguments(
        &mut self,
        mut pattern_arguments: PatternArguments,
    ) -> Option<PatternArguments> {
        Some(walk_pattern_arguments(self, pattern_arguments))
    }
    fn visit_pattern_keyword(
        &mut self,
        mut pattern_keyword: PatternKeyword,
    ) -> Option<PatternKeyword> {
        Some(walk_pattern_keyword(self, pattern_keyword))
    }
    fn visit_body(&mut self, body: &[Stmt]) -> Vec<Stmt> {
        walk_body(self, body)
    }
    fn visit_elif_else_clause(
        &mut self,
        mut elif_else_clause: ElifElseClause,
    ) -> Option<ElifElseClause> {
        Some(walk_elif_else_clause(self, elif_else_clause))
    }
    fn visit_f_string(&mut self, mut f_string: FString) -> Option<FString> {
        Some(walk_f_string(self, f_string))
    }
    fn visit_interpolated_string_element(
        &mut self,
        mut f_string_element: InterpolatedStringElement,
    ) -> Option<InterpolatedStringElement> {
        Some(walk_interpolated_string_element(self, f_string_element))
    }

    fn visit_t_string(&mut self, mut t_string: TString) -> Option<TString> {
        Some(walk_t_string(self, t_string))
    }

    fn visit_string_literal(&mut self, mut string_literal: StringLiteral) -> Option<StringLiteral> {
        Some(walk_string_literal(self, string_literal))
    }
    fn visit_bytes_literal(&mut self, mut bytes_literal: BytesLiteral) -> Option<BytesLiteral> {
        Some(walk_bytes_literal(self, bytes_literal))
    }
}

pub fn walk_body<V: Transformer + ?Sized>(visitor: &mut V, body: &[Stmt]) -> Vec<Stmt> {
    let mut new_stmts: Vec<Stmt> = Vec::new();
    for stmt in body {
        if let Some(new_stmt) = visitor.visit_stmt(stmt.to_owned()) {
            new_stmts.push(new_stmt);
        }
    }
    new_stmts
}

pub fn walk_elif_else_clause<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut elif_else_clause: ElifElseClause,
) -> ElifElseClause {
    if let Some(test) = elif_else_clause.test {
        elif_else_clause.test = visitor.visit_expr(test);
    }
    elif_else_clause.body = visitor.visit_body(&elif_else_clause.body);
    elif_else_clause
}

pub fn walk_stmt<V: Transformer + ?Sized>(visitor: &mut V, stmt: Stmt) -> Option<Stmt> {
    match stmt {
        Stmt::FunctionDef(mut func_def) => {
            let mut new_decorators: Vec<Decorator> = Vec::new();
            for decorator in func_def.decorator_list {
                if let Some(new_decorator) = visitor.visit_decorator(decorator) {
                    new_decorators.push(new_decorator);
                }
            }
            func_def.decorator_list = new_decorators;
            if let Some(type_params) = func_def.type_params {
                func_def.type_params = visitor.visit_type_params(*type_params).map(Box::new);
            }
            func_def.parameters = Box::new(
                visitor
                    .visit_parameters(*func_def.parameters)
                    .expect("Cannot remove parameters from func def"),
            );
            if let Some(expr) = func_def.returns {
                func_def.returns = box_expr_option(visitor.visit_annotation(*expr));
            }
            Some(Stmt::FunctionDef(func_def))
        }
        Stmt::ClassDef(mut class_def) => {
            let mut new_decorators: Vec<Decorator> = Vec::new();
            for decorator in class_def.decorator_list {
                if let Some(new_decorator) = visitor.visit_decorator(decorator) {
                    new_decorators.push(new_decorator);
                }
            }
            class_def.decorator_list = new_decorators;
            if let Some(type_params) = class_def.type_params {
                class_def.type_params = visitor.visit_type_params(*type_params).map(Box::new);
            }
            if let Some(arguments) = class_def.arguments {
                class_def.arguments = visitor.visit_arguments(*arguments).map(Box::new);
            }
            class_def.body = visitor.visit_body(&class_def.body);
            Some(Stmt::ClassDef(class_def))
        }
        Stmt::Return(mut return_) => {
            if let Some(expr) = return_.value {
                return_.value = visitor.visit_expr(*expr).map(Box::new);
            }
            Some(Stmt::Return(return_))
        }
        Stmt::Delete(mut delete) => {
            let mut new_targets: Vec<Expr> = Vec::new();
            for expr in delete.targets {
                if let Some(new_target) = visitor.visit_expr(expr) {
                    new_targets.push(new_target);
                }
            }
            delete.targets = new_targets;
            Some(Stmt::Delete(delete))
        }
        Stmt::TypeAlias(mut type_alias) => {
            type_alias.value = Box::new(
                visitor
                    .visit_expr(*type_alias.value)
                    .expect("Cannot remove value from type alias"),
            );
            if let Some(type_params) = type_alias.type_params {
                type_alias.type_params = visitor.visit_type_params(*type_params).map(Box::new);
            }
            type_alias.name = Box::new(
                visitor
                    .visit_expr(*type_alias.name)
                    .expect("Cannot remove name from type alias"),
            );
            Some(Stmt::TypeAlias(type_alias))
        }
        Stmt::Assign(mut assign) => {
            assign.value = Box::new(
                visitor
                    .visit_expr(*assign.value)
                    .expect("Cannot remove value from assignment"),
            );
            let mut new_targets: Vec<Expr> = Vec::new();
            for expr in assign.targets {
                if let Some(new_target) = visitor.visit_expr(expr) {
                    new_targets.push(new_target);
                }
            }
            assign.targets = new_targets;
            Some(Stmt::Assign(assign))
        }
        Stmt::AugAssign(mut aug_assign) => {
            aug_assign.value = Box::new(
                visitor
                    .visit_annotation(*aug_assign.value)
                    .expect("Cannot remove value from augmented assignment"),
            );
            aug_assign.op = visitor
                .visit_operator(aug_assign.op)
                .expect("Cannot remove value from augmented assignment");
            aug_assign.target = Box::new(
                visitor
                    .visit_annotation(*aug_assign.target)
                    .expect("Cannot remove target from augmented assignment"),
            );
            Some(Stmt::AugAssign(aug_assign))
        }
        Stmt::AnnAssign(mut ann_assign) => {
            if let Some(expr) = ann_assign.value {
                ann_assign.value = visitor.visit_expr(*expr).map(Box::new);
            }
            ann_assign.annotation = Box::new(
                visitor
                    .visit_annotation(*ann_assign.annotation)
                    .expect("Cannot remove annotation from annotated assignment"),
            );
            ann_assign.target = Box::new(
                visitor
                    .visit_expr(*ann_assign.target)
                    .expect("Cannot remove target from annotated assignment"),
            );
            Some(Stmt::AnnAssign(ann_assign))
        }
        Stmt::For(mut for_) => {
            for_.iter = Box::new(
                visitor
                    .visit_expr(*for_.iter)
                    .expect("Cannot remove iter from for statement"),
            );
            for_.target = Box::new(
                visitor
                    .visit_expr(*for_.target)
                    .expect("Cannot remove target from for statement"),
            );
            for_.body = visitor.visit_body(&for_.body);
            for_.orelse = visitor.visit_body(&for_.orelse);
            Some(Stmt::For(for_))
        }
        Stmt::While(mut while_) => {
            while_.test = Box::new(
                visitor
                    .visit_expr(*while_.test)
                    .expect("Cannot remove test from while statement"),
            );
            while_.body = visitor.visit_body(&while_.body);
            while_.orelse = visitor.visit_body(&while_.orelse);
            Some(Stmt::While(while_))
        }
        Stmt::If(mut if_) => {
            if_.test = Box::new(
                visitor
                    .visit_expr(*if_.test)
                    .expect("Cannot remove test from if statement"),
            );
            if_.body = visitor.visit_body(&if_.body);
            let mut new_elif_else_clauses: Vec<ElifElseClause> = Vec::new();
            for clause in if_.elif_else_clauses {
                if let Some(new_elif_else_clause) = visitor.visit_elif_else_clause(clause) {
                    new_elif_else_clauses.push(new_elif_else_clause);
                }
            }
            if_.elif_else_clauses = new_elif_else_clauses;
            Some(Stmt::If(if_))
        }
        Stmt::With(mut with) => {
            let mut new_with_items: Vec<WithItem> = Vec::new();
            for with_item in with.items {
                if let Some(new_with_item) = visitor.visit_with_item(with_item) {
                    new_with_items.push(new_with_item);
                }
            }
            with.items = new_with_items;
            with.body = visitor.visit_body(&with.body);
            Some(Stmt::With(with))
        }
        Stmt::Match(mut match_) => {
            match_.subject = Box::new(
                visitor
                    .visit_expr(*match_.subject)
                    .expect("Cannot remove subject from match statement"),
            );
            let mut new_match_cases: Vec<MatchCase> = Vec::new();
            for match_case in match_.cases {
                if let Some(new_match_case) = visitor.visit_match_case(match_case) {
                    new_match_cases.push(new_match_case);
                }
            }
            match_.cases = new_match_cases;
            Some(Stmt::Match(match_))
        }
        Stmt::Raise(mut raise) => {
            if let Some(expr) = raise.exc {
                raise.exc = visitor.visit_expr(*expr).map(Box::new);
            };
            if let Some(expr) = raise.cause {
                raise.cause = visitor.visit_expr(*expr).map(Box::new);
            };
            Some(Stmt::Raise(raise))
        }
        Stmt::Try(mut try_) => {
            try_.body = visitor.visit_body(&try_.body);
            let mut new_handlers: Vec<ExceptHandler> = Vec::new();
            for except_handler in try_.handlers {
                if let Some(new_handler) = visitor.visit_except_handler(except_handler) {
                    new_handlers.push(new_handler);
                }
            }
            try_.handlers = new_handlers;
            try_.orelse = visitor.visit_body(&try_.orelse);
            try_.finalbody = visitor.visit_body(&try_.finalbody);
            Some(Stmt::Try(try_))
        }
        Stmt::Assert(mut assert) => {
            assert.test = Box::new(
                visitor
                    .visit_expr(*assert.test)
                    .expect("Cannot remove test from assertion"),
            );
            if let Some(expr) = assert.msg {
                assert.msg = visitor.visit_expr(*expr).map(Box::new);
            }
            Some(Stmt::Assert(assert))
        }
        Stmt::Import(mut import) => {
            let mut new_aliases: Vec<Alias> = Vec::new();
            for alias in import.names {
                if let Some(new_alias) = visitor.visit_alias(alias) {
                    new_aliases.push(new_alias);
                }
            }
            import.names = new_aliases;
            Some(Stmt::Import(import))
        }
        Stmt::ImportFrom(mut import_from) => {
            let mut new_aliases: Vec<Alias> = Vec::new();
            for alias in import_from.names {
                if let Some(new_alias) = visitor.visit_alias(alias) {
                    new_aliases.push(new_alias);
                }
            }
            import_from.names = new_aliases;
            Some(Stmt::ImportFrom(import_from))
        }
        Stmt::Global(_) => Some(stmt),
        Stmt::Nonlocal(_) => Some(stmt),
        Stmt::Expr(mut expr) => {
            expr.value = Box::new(visitor.visit_expr(*expr.value).expect(""));
            Some(Stmt::Expr(expr))
        }
        Stmt::Pass(_) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::IpyEscapeCommand(_) => {
            Some(stmt)
        }
    }
}

#[allow(unused_mut)]
pub fn walk_annotation<V: Transformer + ?Sized>(visitor: &mut V, mut expr: Expr) -> Option<Expr> {
    visitor.visit_expr(expr)
}

pub fn walk_decorator<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut decorator: Decorator,
) -> Option<Decorator> {
    decorator.expression = visitor
        .visit_expr(decorator.expression)
        .expect("Cannot remove expression from decorator");
    Some(decorator)
}

pub fn walk_expr<V: Transformer + ?Sized>(visitor: &mut V, expr: Expr) -> Option<Expr> {
    match expr {
        Expr::BoolOp(mut bool_op) => {
            bool_op.op = visitor
                .visit_bool_op(bool_op.op)
                .expect("Cannot remove bool op from bool op");
            let mut new_values: Vec<Expr> = Vec::new();
            for expr in bool_op.values {
                if let Some(new_value) = visitor.visit_expr(expr) {
                    new_values.push(new_value);
                }
            }
            bool_op.values = new_values;
            Some(Expr::BoolOp(bool_op))
        }
        Expr::Named(mut named) => {
            named.value = Box::new(
                visitor
                    .visit_expr(*named.value)
                    .expect("Cannot remove value from named expression"),
            );
            named.target = Box::new(
                visitor
                    .visit_expr(*named.target)
                    .expect("Cannot remove target from named expression"),
            );
            Some(Expr::Named(named))
        }
        Expr::BinOp(mut bin_op) => {
            bin_op.left = Box::new(
                visitor
                    .visit_expr(*bin_op.left)
                    .expect("Cannot remove left from binary operation"),
            );
            bin_op.op = visitor
                .visit_operator(bin_op.op)
                .expect("Cannot remove operator from binary operation");
            bin_op.right = Box::new(
                visitor
                    .visit_expr(*bin_op.right)
                    .expect("Cannot remove right from binary operation"),
            );
            Some(Expr::BinOp(bin_op))
        }
        Expr::UnaryOp(mut unary_op) => {
            unary_op.op = visitor
                .visit_unary_op(unary_op.op)
                .expect("Cannot remove operator from unary operation");
            unary_op.operand = Box::new(
                visitor
                    .visit_expr(*unary_op.operand)
                    .expect("Cannot remove operand from unary operation"),
            );

            Some(Expr::UnaryOp(unary_op))
        }
        Expr::Lambda(mut lambda) => {
            if let Some(parameters) = lambda.parameters {
                lambda.parameters = visitor.visit_parameters(*parameters).map(Box::new);
            }
            lambda.body = Box::new(
                visitor
                    .visit_expr(*lambda.body)
                    .expect("Cannot remove body from lambda expression"),
            );
            Some(Expr::Lambda(lambda))
        }
        Expr::If(mut if_) => {
            if_.test = Box::new(
                visitor
                    .visit_expr(*if_.test)
                    .expect("Cannot remove test from if expression"),
            );
            if_.body = Box::new(
                visitor
                    .visit_expr(*if_.body)
                    .expect("Cannot remove body from if expression"),
            );
            if_.orelse = Box::new(
                visitor
                    .visit_expr(*if_.orelse)
                    .expect("Cannot remove orelse from if expression"),
            );
            Some(Expr::If(if_))
        }
        Expr::Dict(mut dict_) => {
            let mut new_items: Vec<DictItem> = Vec::new();
            for mut dict_item in dict_.items {
                if let Some(key) = dict_item.key {
                    dict_item.key = visitor.visit_expr(key);
                }
                dict_item.value = visitor
                    .visit_expr(dict_item.value)
                    .expect("Cannot remove value from dictionary item");
                new_items.push(dict_item);
            }
            dict_.items = new_items;
            Some(Expr::Dict(dict_))
        }
        Expr::Set(mut set_) => {
            let mut new_elts: Vec<Expr> = Vec::new();
            for expr in set_.elts {
                if let Some(new_expr) = visitor.visit_expr(expr) {
                    new_elts.push(new_expr);
                }
            }
            set_.elts = new_elts;
            Some(Expr::Set(set_))
        }
        Expr::ListComp(mut list_comp) => {
            let mut new_generators: Vec<Comprehension> = Vec::new();
            for comprehension in list_comp.generators {
                if let Some(new_generator) = visitor.visit_comprehension(comprehension) {
                    new_generators.push(new_generator);
                }
            }
            list_comp.generators = new_generators;
            list_comp.elt = Box::new(
                visitor
                    .visit_expr(*list_comp.elt)
                    .expect("Cannot remove elt from list comprehension"),
            );
            Some(Expr::ListComp(list_comp))
        }
        Expr::SetComp(mut set_comp) => {
            let mut new_generators: Vec<Comprehension> = Vec::new();
            for comprehension in set_comp.generators {
                if let Some(new_generator) = visitor.visit_comprehension(comprehension) {
                    new_generators.push(new_generator);
                }
            }
            set_comp.generators = new_generators;
            set_comp.elt = Box::new(
                visitor
                    .visit_expr(*set_comp.elt)
                    .expect("Cannot remove elt from set comprehension"),
            );
            Some(Expr::SetComp(set_comp))
        }
        Expr::DictComp(mut dict_comp) => {
            let mut new_generators: Vec<Comprehension> = Vec::new();
            for comprehension in dict_comp.generators {
                if let Some(new_generator) = visitor.visit_comprehension(comprehension) {
                    new_generators.push(new_generator);
                }
            }
            dict_comp.generators = new_generators;
            dict_comp.key = Box::new(
                visitor
                    .visit_expr(*dict_comp.key)
                    .expect("Cannot remove key from dict comprehension"),
            );
            dict_comp.value = Box::new(
                visitor
                    .visit_expr(*dict_comp.value)
                    .expect("Cannot remove value from dict comprehension"),
            );
            Some(Expr::DictComp(dict_comp))
        }
        Expr::Generator(mut generator) => {
            let mut new_generators: Vec<Comprehension> = Vec::new();
            for comprehension in generator.generators {
                if let Some(new_generator) = visitor.visit_comprehension(comprehension) {
                    new_generators.push(new_generator);
                }
            }
            generator.generators = new_generators;
            generator.elt = Box::new(
                visitor
                    .visit_expr(*generator.elt)
                    .expect("Cannot remove elt from generator expression"),
            );
            Some(Expr::Generator(generator))
        }
        Expr::Await(mut await_) => {
            await_.value = Box::new(
                visitor
                    .visit_expr(*await_.value)
                    .expect("Cannot remove value from await expression"),
            );
            Some(Expr::Await(await_))
        }
        Expr::Yield(mut yield_) => {
            if let Some(expr) = yield_.value {
                yield_.value = visitor.visit_expr(*expr).map(Box::new);
            }
            Some(Expr::Yield(yield_))
        }
        Expr::YieldFrom(mut yield_from) => {
            yield_from.value = Box::new(
                visitor
                    .visit_expr(*yield_from.value)
                    .expect("Cannot remove value from yield from expression"),
            );

            Some(Expr::YieldFrom(yield_from))
        }
        Expr::Compare(mut compare) => {
            compare.left = Box::new(
                visitor
                    .visit_expr(*compare.left)
                    .expect("Cannot remove left from comparison"),
            );
            let mut new_cmp_ops: Vec<CmpOp> = Vec::new();
            for cmp_op in compare.ops {
                if let Some(new_cmp_op) = visitor.visit_cmp_op(cmp_op) {
                    new_cmp_ops.push(new_cmp_op);
                }
            }
            compare.ops = new_cmp_ops.into_boxed_slice();

            let mut new_comparators: Vec<Expr> = Vec::new();
            for comparator in compare.comparators {
                if let Some(new_comparator) = visitor.visit_expr(comparator) {
                    new_comparators.push(new_comparator);
                }
            }
            compare.comparators = new_comparators.into_boxed_slice();
            Some(Expr::Compare(compare))
        }
        Expr::Call(mut call) => {
            call.func = Box::new(
                visitor
                    .visit_expr(*call.func)
                    .expect("Cannot remove func from function call"),
            );
            call.arguments = visitor
                .visit_arguments(call.arguments)
                .expect("Cannot remove arguments from function call");

            Some(Expr::Call(call))
        }
        Expr::FString(mut f_string) => {
            let mut new_f_string_parts: Vec<FStringPart> = Vec::new();
            for f_string_part in f_string.value.iter() {
                if let Some(new_f_string_part) = match f_string_part {
                    ast::FStringPart::Literal(string_literal) => visitor
                        .visit_string_literal(string_literal.to_owned())
                        .map(ast::FStringPart::Literal),

                    ast::FStringPart::FString(f_string) => visitor
                        .visit_f_string(f_string.to_owned())
                        .map(ast::FStringPart::FString),
                } {
                    new_f_string_parts.push(new_f_string_part);
                }
            }
            if new_f_string_parts.len() > 1 {
                f_string.value = FStringValue::concatenated(new_f_string_parts);
            } else if new_f_string_parts.len() == 1 {
                f_string.value = FStringValue::single(
                    new_f_string_parts[0]
                        .as_f_string()
                        .expect("Expected f-string")
                        .to_owned(),
                );
            }
            Some(Expr::FString(f_string))
        }
        Expr::TString(mut t_string) => {
            let mut new_t_string_parts: Vec<TStringPart> = Vec::new();
            for t_string_part in t_string.value.iter() {
                if let Some(new_t_string_part) = match t_string_part {
                    ast::TStringPart::Literal(string_literal) => visitor
                        .visit_string_literal(string_literal.to_owned())
                        .map(ast::TStringPart::Literal),

                    ast::TStringPart::TString(t_string) => visitor
                        .visit_t_string(t_string.to_owned())
                        .map(ast::TStringPart::TString),
                    ast::TStringPart::FString(f_string) => visitor
                        .visit_f_string(f_string.to_owned())
                        .map(ast::TStringPart::FString),
                } {
                    new_t_string_parts.push(new_t_string_part);
                }
            }
            if new_t_string_parts.len() > 1 {
                t_string.value = TStringValue::concatenated(new_t_string_parts);
            } else if new_t_string_parts.len() == 1 {
                t_string.value = TStringValue::single(
                    new_t_string_parts[0]
                        .as_t_string()
                        .expect("Expected t-string")
                        .to_owned(),
                );
            }
            Some(Expr::TString(t_string))
        }

        Expr::StringLiteral(mut string_literal) => {
            let mut new_string_literals: Vec<StringLiteral> = Vec::new();
            for string_literal in string_literal.value.iter() {
                if let Some(new_string_literal) =
                    visitor.visit_string_literal(string_literal.to_owned())
                {
                    new_string_literals.push(new_string_literal);
                }
            }
            if new_string_literals.len() > 1 {
                string_literal.value = StringLiteralValue::concatenated(new_string_literals);
            } else {
                string_literal.value =
                    StringLiteralValue::single(new_string_literals[0].to_owned());
            }
            Some(Expr::StringLiteral(string_literal))
        }
        Expr::BytesLiteral(mut bytes_literal) => {
            let mut new_bytes_literals: Vec<BytesLiteral> = Vec::new();
            for bytes_literal in bytes_literal.value.iter() {
                if let Some(new_bytes_literal) =
                    visitor.visit_bytes_literal(bytes_literal.to_owned())
                {
                    new_bytes_literals.push(new_bytes_literal);
                }
            }
            if new_bytes_literals.len() > 1 {
                bytes_literal.value = BytesLiteralValue::concatenated(new_bytes_literals);
            } else {
                bytes_literal.value = BytesLiteralValue::single(new_bytes_literals[0].to_owned());
            }
            Some(Expr::BytesLiteral(bytes_literal))
        }
        Expr::NumberLiteral(_)
        | Expr::BooleanLiteral(_)
        | Expr::NoneLiteral(_)
        | Expr::EllipsisLiteral(_) => Some(expr),
        Expr::Attribute(mut attribute) => {
            attribute.value = Box::new(
                visitor
                    .visit_expr(*attribute.value)
                    .expect("Cannot remove value from attribute"),
            );
            attribute.ctx = visitor
                .visit_expr_context(attribute.ctx)
                .expect("Cannot remove context from attribute");

            Some(Expr::Attribute(attribute))
        }
        Expr::Subscript(mut subscript) => {
            subscript.value = Box::new(
                visitor
                    .visit_expr(*subscript.value)
                    .expect("Cannot remove value from subscript"),
            );
            subscript.slice = Box::new(
                visitor
                    .visit_expr(*subscript.slice)
                    .expect("Cannot remove slice from subscript"),
            );

            subscript.ctx = visitor
                .visit_expr_context(subscript.ctx)
                .expect("Cannot remove context from subscript");

            Some(Expr::Subscript(subscript))
        }
        Expr::Starred(mut starred) => {
            starred.value = Box::new(
                visitor
                    .visit_expr(*starred.value)
                    .expect("Cannot remove value from starred expression"),
            );
            starred.ctx = visitor
                .visit_expr_context(starred.ctx)
                .expect("Cannot remove context from starred expression");
            Some(Expr::Starred(starred))
        }
        Expr::Name(mut name) => {
            name.ctx = visitor
                .visit_expr_context(name.ctx)
                .expect("Cannot remove context from name expression");
            Some(Expr::Name(name))
        }
        Expr::List(mut list) => {
            let mut new_elts: Vec<Expr> = Vec::new();
            for expr in list.elts {
                if let Some(new_expr) = visitor.visit_expr(expr) {
                    new_elts.push(new_expr);
                }
            }
            list.elts = new_elts;
            list.ctx = visitor
                .visit_expr_context(list.ctx)
                .expect("Cannot remove context from list expression");
            Some(Expr::List(list))
        }
        Expr::Tuple(mut tuple) => {
            let mut new_elts: Vec<Expr> = Vec::new();
            for expr in tuple.elts {
                if let Some(new_expr) = visitor.visit_expr(expr) {
                    new_elts.push(new_expr);
                }
            }
            tuple.elts = new_elts;
            tuple.ctx = visitor
                .visit_expr_context(tuple.ctx)
                .expect("Cannot remove context from tuple expression");
            Some(Expr::Tuple(tuple))
        }
        Expr::Slice(mut slice) => {
            if let Some(expr) = slice.lower {
                slice.lower = visitor.visit_expr(*expr).map(Box::new);
            }
            if let Some(expr) = slice.upper {
                slice.upper = visitor.visit_expr(*expr).map(Box::new);
            }
            if let Some(expr) = slice.step {
                slice.step = visitor.visit_expr(*expr).map(Box::new);
            }
            Some(Expr::Slice(slice))
        }
        Expr::IpyEscapeCommand(_) => Some(expr),
    }
}

pub fn walk_comprehension<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut comprehension: Comprehension,
) -> Comprehension {
    comprehension.iter = visitor
        .visit_expr(comprehension.iter)
        .expect("Cannot remove iter from comprehension");
    comprehension.target = visitor
        .visit_expr(comprehension.target)
        .expect("Cannot remove target from comprehension");
    let mut new_ifs: Vec<Expr> = Vec::new();
    for expr in comprehension.ifs {
        if let Some(new_if) = visitor.visit_expr(expr) {
            new_ifs.push(new_if);
        }
    }
    comprehension.ifs = new_ifs;
    comprehension
}

pub fn walk_except_handler<V: Transformer + ?Sized>(
    visitor: &mut V,
    except_handler: ExceptHandler,
) -> ExceptHandler {
    match except_handler {
        ExceptHandler::ExceptHandler(mut except_except_handler) => {
            if let Some(expr) = except_except_handler.type_ {
                except_except_handler.type_ = visitor.visit_expr(*expr).map(Box::new);
            }
            except_except_handler.body = visitor.visit_body(&except_except_handler.body);
            ExceptHandler::ExceptHandler(except_except_handler)
        }
    }
}

pub fn walk_arguments<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut arguments: Arguments,
) -> Arguments {
    // Note that the there might be keywords before the last arg, e.g. in
    // f(*args, a=2, *args2, **kwargs)`, but we follow Python in evaluating first `args` and then
    // `keywords`. See also [Arguments::arguments_source_order`].
    let mut new_args: Vec<Expr> = Vec::new();
    for arg in arguments.args {
        if let Some(new_arg) = visitor.visit_expr(arg) {
            new_args.push(new_arg);
        }
    }
    arguments.args = new_args.into_boxed_slice();

    let mut new_keywords: Vec<Keyword> = Vec::new();
    for keyword in arguments.keywords {
        if let Some(new_keyword) = visitor.visit_keyword(keyword) {
            new_keywords.push(new_keyword);
        }
    }
    arguments.keywords = new_keywords.into_boxed_slice();

    arguments
}

pub fn walk_parameters<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut parameters: Parameters,
) -> Parameters {
    // Defaults are evaluated before annotations.

    let mut new_posonlyargs: Vec<ParameterWithDefault> = Vec::new();
    for mut arg in parameters.posonlyargs {
        if let Some(default) = arg.default {
            arg.default = visitor.visit_expr(*default).map(Box::new);
        }
        if let Some(new_parameter) = visitor.visit_parameter(arg.parameter) {
            arg.parameter = new_parameter;
            new_posonlyargs.push(arg);
        }
    }
    parameters.posonlyargs = new_posonlyargs;
    let mut new_args: Vec<ParameterWithDefault> = Vec::new();
    for mut arg in parameters.args {
        if let Some(default) = arg.default {
            arg.default = visitor.visit_expr(*default).map(Box::new);
        }
        if let Some(new_parameter) = visitor.visit_parameter(arg.parameter) {
            arg.parameter = new_parameter;
            new_args.push(arg);
        }
    }
    parameters.args = new_args;
    if let Some(arg) = parameters.vararg {
        parameters.vararg = visitor.visit_parameter(*arg).map(Box::new);
    }
    let mut new_kwonlyargs: Vec<ParameterWithDefault> = Vec::new();
    for mut arg in parameters.kwonlyargs {
        if let Some(default) = arg.default {
            arg.default = visitor.visit_expr(*default).map(Box::new);
        }
        if let Some(new_parameter) = visitor.visit_parameter(arg.parameter) {
            arg.parameter = new_parameter;
            new_kwonlyargs.push(arg);
        }
    }
    parameters.kwonlyargs = new_kwonlyargs;

    if let Some(arg) = parameters.kwarg {
        parameters.kwarg = visitor.visit_parameter(*arg).map(Box::new);
    }
    parameters
}

pub fn walk_parameter<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut parameter: Parameter,
) -> Parameter {
    if let Some(expr) = parameter.annotation {
        parameter.annotation = visitor.visit_annotation(*expr).map(Box::new);
    }
    parameter
}

pub fn walk_keyword<V: Transformer + ?Sized>(visitor: &mut V, mut keyword: Keyword) -> Keyword {
    keyword.value = visitor
        .visit_expr(keyword.value)
        .expect("Cannot remove value from keyword");
    keyword
}

pub fn walk_with_item<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut with_item: WithItem,
) -> WithItem {
    with_item.context_expr = visitor
        .visit_expr(with_item.context_expr)
        .expect("Cannot remove context expression from with item");
    if let Some(expr) = with_item.optional_vars {
        with_item.optional_vars = visitor.visit_expr(*expr).map(Box::new);
    }
    with_item
}

pub fn walk_type_params<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut type_params: TypeParams,
) -> TypeParams {
    let mut new_type_params: Vec<TypeParam> = Vec::new();
    for type_param in type_params.type_params {
        if let Some(new_type_param) = visitor.visit_type_param(type_param) {
            new_type_params.push(new_type_param);
        }
    }
    type_params.type_params = new_type_params;
    type_params
}

pub fn walk_type_param<V: Transformer + ?Sized>(
    visitor: &mut V,
    type_param: TypeParam,
) -> TypeParam {
    match type_param {
        TypeParam::TypeVar(mut type_param_var) => {
            if let Some(expr) = type_param_var.bound {
                type_param_var.bound = visitor.visit_expr(*expr).map(Box::new);
            }
            if let Some(expr) = type_param_var.default {
                type_param_var.default = visitor.visit_expr(*expr).map(Box::new);
            }
            TypeParam::TypeVar(type_param_var)
        }
        TypeParam::TypeVarTuple(mut type_var_tuple) => {
            if let Some(expr) = type_var_tuple.default {
                type_var_tuple.default = visitor.visit_expr(*expr).map(Box::new);
            }
            TypeParam::TypeVarTuple(type_var_tuple)
        }
        TypeParam::ParamSpec(mut param_spec) => {
            if let Some(expr) = param_spec.default {
                param_spec.default = visitor.visit_expr(*expr).map(Box::new);
            }
            TypeParam::ParamSpec(param_spec)
        }
    }
}

pub fn walk_match_case<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut match_case: MatchCase,
) -> MatchCase {
    match_case.pattern = visitor
        .visit_pattern(match_case.pattern)
        .expect("Cannot expect pattern from match case");
    if let Some(expr) = match_case.guard {
        match_case.guard = visitor.visit_expr(*expr).map(Box::new);
    }
    match_case.body = visitor.visit_body(&match_case.body);
    match_case
}

pub fn walk_pattern<V: Transformer + ?Sized>(visitor: &mut V, pattern: Pattern) -> Pattern {
    match pattern {
        Pattern::MatchValue(mut match_value) => {
            match_value.value = Box::new(
                visitor
                    .visit_expr(*match_value.value)
                    .expect("Cannot remove value from value match pattern"),
            );
            Pattern::MatchValue(match_value)
        }
        Pattern::MatchSingleton(_) => pattern,
        Pattern::MatchSequence(mut match_sequence) => {
            let mut new_patterns: Vec<Pattern> = Vec::new();
            for pattern in match_sequence.patterns {
                if let Some(new_pattern) = visitor.visit_pattern(pattern) {
                    new_patterns.push(new_pattern);
                }
            }
            match_sequence.patterns = new_patterns;
            Pattern::MatchSequence(match_sequence)
        }
        Pattern::MatchMapping(mut match_mapping) => {
            let mut new_keys: Vec<Expr> = Vec::new();
            for expr in match_mapping.keys {
                if let Some(new_expr) = visitor.visit_expr(expr) {
                    new_keys.push(new_expr);
                }
            }

            match_mapping.keys = new_keys;

            let mut new_patterns: Vec<Pattern> = Vec::new();
            for pattern in match_mapping.patterns {
                if let Some(new_pattern) = visitor.visit_pattern(pattern) {
                    new_patterns.push(new_pattern);
                }
            }
            match_mapping.patterns = new_patterns;
            Pattern::MatchMapping(match_mapping)
        }
        Pattern::MatchClass(mut match_class) => {
            match_class.cls = Box::new(
                visitor
                    .visit_expr(*match_class.cls)
                    .expect("Cannot remove cls from match class"),
            );
            match_class.arguments = visitor
                .visit_pattern_arguments(match_class.arguments)
                .expect("Cannot remove pattern arguments");
            Pattern::MatchClass(match_class)
        }
        Pattern::MatchStar(_) => pattern,
        Pattern::MatchAs(mut match_as) => {
            if let Some(pattern) = match_as.pattern {
                match_as.pattern = visitor.visit_pattern(*pattern).map(Box::new);
            }
            Pattern::MatchAs(match_as)
        }
        Pattern::MatchOr(mut match_or) => {
            let mut new_patterns: Vec<Pattern> = Vec::new();
            for pattern in match_or.patterns {
                if let Some(new_pattern) = visitor.visit_pattern(pattern) {
                    new_patterns.push(new_pattern);
                }
            }
            match_or.patterns = new_patterns;
            Pattern::MatchOr(match_or)
        }
    }
}

pub fn walk_pattern_arguments<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut pattern_arguments: PatternArguments,
) -> PatternArguments {
    let mut new_patterns: Vec<Pattern> = Vec::new();
    for pattern in pattern_arguments.patterns {
        if let Some(new_pattern) = visitor.visit_pattern(pattern) {
            new_patterns.push(new_pattern);
        }
    }
    pattern_arguments.patterns = new_patterns;
    let mut new_keywords: Vec<PatternKeyword> = Vec::new();
    for keyword in pattern_arguments.keywords {
        if let Some(new_keyword) = visitor.visit_pattern_keyword(keyword) {
            new_keywords.push(new_keyword);
        }
    }
    pattern_arguments.keywords = new_keywords;
    pattern_arguments
}

pub fn walk_pattern_keyword<V: Transformer + ?Sized>(
    visitor: &mut V,
    mut pattern_keyword: PatternKeyword,
) -> PatternKeyword {
    pattern_keyword.pattern = visitor
        .visit_pattern(pattern_keyword.pattern)
        .expect("Cannot remove pattern from pattern keyword");

    pattern_keyword
}

pub fn walk_f_string<V: Transformer + ?Sized>(visitor: &mut V, mut f_string: FString) -> FString {
    let mut new_elements: Vec<InterpolatedStringElement> = Vec::new();
    for element in &f_string.elements {
        if let Some(new_element) = visitor.visit_interpolated_string_element(element.to_owned()) {
            new_elements.push(new_element)
        };
    }
    f_string.elements = InterpolatedStringElements::from(new_elements);
    f_string
}

pub fn walk_interpolated_string_element<V: Transformer + ?Sized>(
    visitor: &mut V,
    interpolated_string_element: InterpolatedStringElement,
) -> InterpolatedStringElement {
    if let ast::InterpolatedStringElement::Interpolation(mut interpolation) =
        interpolated_string_element
    {
        interpolation.expression = Box::new(
            visitor
                .visit_expr(*interpolation.expression)
                .expect("Cannot remove expression from f-string element expression"),
        );
        if let Some(mut format_spec) = interpolation.format_spec {
            let mut new_spec_elements: Vec<InterpolatedStringElement> = Vec::new();
            for spec_element in &format_spec.elements {
                if let Some(new_spec_element) =
                    visitor.visit_interpolated_string_element(spec_element.to_owned())
                {
                    new_spec_elements.push(new_spec_element);
                }
            }
            format_spec.elements = InterpolatedStringElements::from(new_spec_elements);
            interpolation.format_spec = Some(format_spec)
        }
        return ast::InterpolatedStringElement::Interpolation(interpolation);
    }

    interpolated_string_element
}

pub fn walk_t_string<V: Transformer + ?Sized>(visitor: &mut V, mut t_string: TString) -> TString {
    let mut new_elements: Vec<InterpolatedStringElement> = Vec::new();
    for element in &t_string.elements {
        if let Some(new_element) = visitor.visit_interpolated_string_element(element.to_owned()) {
            new_elements.push(new_element)
        };
    }
    t_string.elements = InterpolatedStringElements::from(new_elements);
    t_string
}

pub fn walk_expr_context<V: Transformer + ?Sized>(
    _visitor: &mut V,
    mut _expr_context: ExprContext,
) -> ExprContext {
    _expr_context
}

pub fn walk_bool_op<V: Transformer + ?Sized>(_visitor: &mut V, mut _bool_op: BoolOp) -> BoolOp {
    _bool_op
}

pub fn walk_operator<V: Transformer + ?Sized>(
    _visitor: &mut V,
    mut _operator: Operator,
) -> Operator {
    _operator
}

pub fn walk_unary_op<V: Transformer + ?Sized>(_visitor: &mut V, mut _unary_op: UnaryOp) -> UnaryOp {
    _unary_op
}

pub fn walk_cmp_op<V: Transformer + ?Sized>(_visitor: &mut V, mut _cmp_op: CmpOp) -> CmpOp {
    _cmp_op
}

pub fn walk_alias<V: Transformer + ?Sized>(_visitor: &mut V, mut _alias: Alias) -> Alias {
    _alias
}

pub fn walk_string_literal<V: Transformer + ?Sized>(
    _visitor: &mut V,
    mut _string_literal: StringLiteral,
) -> StringLiteral {
    _string_literal
}

pub fn walk_bytes_literal<V: Transformer + ?Sized>(
    _visitor: &mut V,
    mut _bytes_literal: BytesLiteral,
) -> BytesLiteral {
    _bytes_literal
}
