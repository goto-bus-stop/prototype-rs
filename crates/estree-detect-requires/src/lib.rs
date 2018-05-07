extern crate easter;

use easter::stmt::{StmtListItem, Stmt};
use easter::decl::Dtor;
use easter::expr::Expr;
use easter::id::Id;
use easter::prog::Script;

pub fn detect(ast: &Script) -> Vec<String> {
    Detective::new(ast).detect()
}

struct Detective<'a> {
    ast: &'a Script,
    modules: Vec<String>,
}

impl<'a> Detective<'a> {
    fn new(ast: &Script) -> Detective {
        Detective { ast, modules: vec![] }
    }

    fn detect(mut self) -> Vec<String> {
        for item in &self.ast.body {
            match item {
                &StmtListItem::Stmt(ref stmt) => {
                    self.detect_stmt(stmt);
                },
                _ => (),
            }
        }

        self.modules
    }

    fn detect_stmt(&mut self, stmt: &Stmt) -> () {
        match stmt {
            &Stmt::Var(_, ref decls, _) => {
                self.detect_var(decls);
            },
            &Stmt::Expr(_, ref expr, _) => {
                self.detect_expr(expr);
            },
            _ => (),
        }
    }

    fn detect_var(&mut self, decls: &Vec<Dtor>) -> () {
        for decl in decls {
            match decl {
                &Dtor::Simple(_, _, Some(ref expr)) => {
                    self.detect_expr(expr);
                },
                _ => (),
            }
        }
    }

    fn detect_expr(&mut self, expr: &Expr) -> () {
        if let &Expr::Call(_, ref callee, ref args) = expr {
            if is_require_name(callee) {
                if let Some(&Expr::String(_, ref val)) = args.first() {
                    self.modules.push(val.value.clone());
                }
            }
        }
    }
}

fn is_require_name(id: &Expr) -> bool {
    if let &Expr::Id(Id { name: ref fn_name, .. }) = id {
        fn_name.as_ref() == "require"
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    extern crate esprit;
    use self::esprit::script;
    use ::detect;

    #[test]
    fn detects_var_require() {
        assert_eq!(detect(&script("var x = require('y')").unwrap()), vec!["y"]);
    }

    #[test]
    fn detects_bare_require() {
        assert_eq!(detect(&script("require('y')").unwrap()), vec!["y"]);
    }
}
