use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::js_word;
use swc_core::ecma::visit::VisitWith;
use swc_core::ecma::visit::{noop_visit_type, Visit};
use tracing::trace;

struct CommonJSVisitor {
    has_cjs_syntax: bool,
    cjs_syntax: Option<MemberExpr>,
}

/// Does not handle require statements (yet)
impl Visit for CommonJSVisitor {
    noop_visit_type!();
    fn visit_member_expr(&mut self, n: &MemberExpr) {
        n.visit_children_with(self);
        match (&*n.obj, &n.prop) {
            // `module.exports`
            (
                Expr::Ident(Ident { sym: obj_sym, .. }),
                MemberProp::Ident(Ident { sym: prop_sym, .. }),
            ) if obj_sym == "module" && prop_sym == "exports" => {
                self.has_cjs_syntax = true;
                self.cjs_syntax = Some(n.clone())
            }
            // `exports.`
            (Expr::Ident(Ident { sym: obj_sym, .. }), _) => {
                if obj_sym == "exports" {
                    self.has_cjs_syntax = true;
                    self.cjs_syntax = Some(n.clone())
                }
            }
            _ => {}
        }
    }

    fn visit_call_expr(&mut self, n: &CallExpr) {
        n.visit_children_with(self);
        if let Callee::Expr(expr) = &n.callee {
            match &**expr {
                // `require()`
                Expr::Ident(Ident {
                    sym: js_word!("require"),
                    ..
                }) => {
                    self.has_cjs_syntax = true;
                }
                // `require.resolve`
                Expr::Member(member) => match (&*member.obj, &member.prop) {
                    (
                        Expr::Ident(Ident { sym: obj_sym, .. }),
                        MemberProp::Ident(Ident { sym: prop_sym, .. }),
                    ) if obj_sym == "require" && prop_sym == "resolve" => {
                        self.has_cjs_syntax = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

pub fn has_cjs_syntax(module: &Module) -> bool {
    let mut m = CommonJSVisitor {
        has_cjs_syntax: false,
        cjs_syntax: None,
    };
    module.visit_with(&mut m);

    if let Some(expr) = m.cjs_syntax {
        trace!("CommonJS syntax expression {:?}", expr);
    }
    m.has_cjs_syntax
}

#[cfg(test)]
mod test {
    use super::*;
    use swc_core::{
        common::{
            errors::{ColorConfig, Handler},
            sync::Lrc,
            FileName, SourceMap,
        },
        ecma::parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax},
    };

    fn module_from(code: &str) -> Module {
        let cm: Lrc<SourceMap> = Default::default();
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
        let fm = cm.new_source_file(FileName::Custom("test.js".into()), code.into());

        let lexer = Lexer::new(
            Syntax::Es(Default::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let capturing = Capturing::new(lexer);

        let mut parser = Parser::new_from(capturing);

        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }

        parser
            .parse_module()
            .map_err(|e| e.into_diagnostic(&handler).emit())
            .expect("Failed to parse module.")
    }

    #[test]
    fn test_module_exports() {
        let module = module_from("module.exports = 1");
        assert!(has_cjs_syntax(&module));
    }

    #[test]
    fn test_exports() {
        let module = module_from("exports.CommonJSVisitor = {};");
        assert!(has_cjs_syntax(&module));
    }

    #[test]
    fn test_require() {
        let module = module_from("require('foo')");
        assert!(has_cjs_syntax(&module));
    }

    #[test]
    fn test_require_resolve() {
        let module = module_from("require.resolve('foo')");
        assert!(has_cjs_syntax(&module));
    }
}
