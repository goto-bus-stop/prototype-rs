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
                // var
                &StmtListItem::Stmt(Stmt::Var(_, ref decls, _semi)) => {
                    self.detect_var(decls);
                },
                _ => (),
            }
        }

        self.modules
    }

    fn detect_var(&mut self, decls: &Vec<Dtor>) -> () {
        for decl in decls {
            match decl {
                &Dtor::Simple(_, _, Some(Expr::Call(_, ref expr, ref args))) => {
                    if is_require_name(expr) {
                        match args.first().unwrap() {
                            &Expr::String(_, ref val) => {
                                self.modules.push(val.value.clone());
                            },
                            _ => (),
                        }
                    }
                },
                _ => (),
            }
        }
    }
}

fn is_require_name(id: &Expr) -> bool {
    match id {
        &Expr::Id(Id { name: ref fn_name, .. }) => fn_name.as_ref() == "require",
        _ => false,
    }
}
