extern crate easter;

use easter::stmt::{StmtListItem, Stmt};
use easter::decl::{Decl, Dtor};
use easter::expr::Expr;
use easter::obj::PropVal;
use easter::fun::Fun;
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
        self.walk_script();

        self.modules
    }

    fn walk_script(&mut self) -> () {
        for item in &self.ast.body {
            self.walk_stmt_item(item);
        }
    }

    fn walk_stmt_item(&mut self, item: &StmtListItem) -> () {
        match item {
            &StmtListItem::Stmt(ref stmt) => self.walk_stmt(stmt),
            &StmtListItem::Decl(ref decl) => self.walk_decl(decl),
        }
    }

    fn walk_stmt(&mut self, stmt: &Stmt) -> () {
        match stmt {
            &Stmt::Block(_, ref items) => {
                for item in items {
                    self.walk_stmt_item(item);
                }
            },
            &Stmt::Var(_, ref decls, _) => self.walk_var(decls),
            &Stmt::Expr(_, ref expr, _) => self.walk_expr(expr),
            &Stmt::If(_, ref cond, ref cons, ref alt) => {
                self.walk_expr(cond);
                self.walk_stmt(cons.as_ref());
                if let &Some(ref node) = alt { self.walk_stmt(node.as_ref()); }
            },
            &Stmt::Label(_, _, ref block) => self.walk_stmt(block.as_ref()),
            &Stmt::Switch(_, ref cond, ref cases) => {
                self.walk_expr(cond);
                for case in cases {
                    if let Some(ref test) = case.test { self.walk_expr(test); }
                    for item in &case.body {
                        self.walk_stmt_item(item);
                    }
                }
            },
            &Stmt::Return(_, Some(ref arg), _) => self.walk_expr(arg),
            &Stmt::Throw(_, ref arg, _) => self.walk_expr(arg),
            &Stmt::Try(_, ref block, ref caught, ref finally) => {
                for item in block { self.walk_stmt_item(item); }
                if let &Some(ref caught_block) = caught {
                    for item in &caught_block.body { self.walk_stmt_item(item); }
                }
                if let &Some(ref finally_block) = finally {
                    for item in finally_block { self.walk_stmt_item(item); }
                }
            },
            &Stmt::While(_, ref cond, ref body) => {
                self.walk_expr(cond);
                self.walk_stmt(body.as_ref());
            },
            &Stmt::DoWhile(_, ref body, ref cond, _) => {
                self.walk_stmt(body.as_ref());
                self.walk_expr(cond);
            },
            &Stmt::For(_, ref _init, ref cond, ref update, ref body) => {
                // if let &Some(ref node) = head { self.walk_for_head(node); }
                if let &Some(ref node) = cond { self.walk_expr(&node); }
                if let &Some(ref node) = update { self.walk_expr(&node); }
                self.walk_stmt(body.as_ref());
            },
            &Stmt::ForIn(_, ref _head, ref iterable, ref body) => {
                // if let &Some(ref node) = head { self.walk_for_in_head(node); }
                self.walk_expr(iterable);
                self.walk_stmt(body.as_ref());
            },
            &Stmt::ForOf(_, ref _head, ref iterable, ref body) => {
                // if let &Some(ref node) = head { self.walk_for_of_head(node); }
                self.walk_expr(iterable);
                self.walk_stmt(body.as_ref());
            },
            _ => (),
        }
    }

    fn walk_decl(&mut self, decl: &Decl) -> () {
        let &Decl::Fun(ref fun) = decl;
        self.walk_fun(fun);
    }

    fn walk_var(&mut self, decls: &Vec<Dtor>) -> () {
        for decl in decls {
            match decl {
                &Dtor::Simple(_, _, Some(ref expr)) => self.walk_expr(expr),
                _ => (),
            }
        }
    }

    fn walk_expr(&mut self, expr: &Expr) -> () {
        match expr {
            // TODO move this into a callback
            // and move the walk_* functions to generic AST walker
            &Expr::Call(_, ref callee, ref args) => {
                if is_require_name(callee) {
                    if let Some(&Expr::String(_, ref val)) = args.first() {
                        self.modules.push(val.value.clone());
                    }
                } else {
                    self.walk_expr(callee);
                    for arg in args {
                        self.walk_expr(arg);
                    }
                }
            },
            &Expr::Seq(_, ref exprs) => {
                for expr in exprs {
                    self.walk_expr(expr);
                }
            }
            &Expr::Arr(_, ref elements) => {
                for el in elements {
                    if let &Some(ref node) = el {
                        self.walk_expr(node);
                    }
                }
            },
            &Expr::Obj(_, ref properties) => {
                for prop in properties {
                    match &prop.val {
                        &PropVal::Init(ref value) => self.walk_expr(value),
                        &PropVal::Get(_, ref body) => {
                            for item in body {
                                self.walk_stmt_item(item);
                            }
                        },
                        &PropVal::Set(_, _, ref body) => {
                            for item in body {
                                self.walk_stmt_item(item);
                            }
                        },
                    }
                }
            },
            &Expr::Fun(ref fun) => self.walk_fun(fun),
            &Expr::Unop(_, _, ref expr) => self.walk_expr(expr.as_ref()),
            &Expr::Binop(_, _, ref a, ref b) => {
                self.walk_expr(a.as_ref());
                self.walk_expr(b.as_ref());
            },
            &Expr::Logop(_, _, ref a, ref b) => {
                self.walk_expr(a.as_ref());
                self.walk_expr(b.as_ref());
            },
            &Expr::PreInc(_, ref expr) => self.walk_expr(expr.as_ref()),
            &Expr::PostInc(_, ref expr) => self.walk_expr(expr.as_ref()),
            &Expr::PreDec(_, ref expr) => self.walk_expr(expr.as_ref()),
            &Expr::PostDec(_, ref expr) => self.walk_expr(expr.as_ref()),
            &Expr::Assign(_, _, _, ref expr) => self.walk_expr(expr.as_ref()),
            &Expr::Cond(_, ref cond, ref cons, ref alt) => {
                self.walk_expr(cond.as_ref());
                self.walk_expr(cons.as_ref());
                self.walk_expr(alt.as_ref());
            },
            _ => (),
        }
    }

    fn walk_fun(&mut self, fun: &Fun) -> () {
        for item in &fun.body {
            self.walk_stmt_item(&item);
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

    #[test]
    fn detects_require_in_array_and_obj() {
        assert_eq!(detect(&script("[null,whatever(),{a:require('a')},,b,require('c')]").unwrap()), vec!["a", "c"]);
    }

    #[test]
    fn detects_require_in_fn() {
        assert_eq!(detect(&script("
            var a = null
            var b = function () {
                function c() { require('d') }
                require('e')
            }, c = require('f')
            null, require('g'), void require
        ").unwrap()), vec!["d", "e", "f", "g"]);
    }
}
