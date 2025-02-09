use rustpython_ast::{Arg, ArgWithDefault, Arguments, Visitor};
// patch for missing visits in RustPython visitor
// https://github.com/RustPython/Parser/issues/133
pub trait VisitorPatch: Visitor {
    fn generic_visit_arg_patch(&mut self, arg: Arg) {
        if let Some(annotation) = arg.annotation {
            self.visit_expr(*annotation);
        }
    }

    fn generic_visit_arguments_patch(&mut self, args: Arguments) {
        for arg in args.args {
            self.visit_arg_with_default(arg);
        }

        for posonly in args.posonlyargs {
            self.visit_arg_with_default(posonly);
        }

        for kwonly in args.kwonlyargs {
            self.visit_arg_with_default(kwonly);
        }
        if let Some(vararg) = args.vararg {
            self.visit_arg(*vararg);
        }
        if let Some(kwarg) = args.kwarg {
            self.visit_arg(*kwarg);
        }
    }

    fn visit_arg_with_default(&mut self, arg: ArgWithDefault) {
        self.generic_visit_arg_with_default(arg);
    }

    fn generic_visit_arg_with_default(&mut self, arg: ArgWithDefault) {
        self.visit_arg(arg.def);
        if let Some(default) = arg.default {
            self.visit_expr(*default);
        }
    }
}
