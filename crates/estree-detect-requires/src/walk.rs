extern crate easter;

use easter::stmt::{StmtListItem, Stmt};
use easter::decl::{Decl, Dtor};
use easter::expr::Expr;
use easter::obj::PropVal;
use easter::fun::Fun;
use easter::prog::Script;

pub struct Walker<'a, C: Callbacks> {
    ast: &'a Script,
    callbacks: C,
}

pub trait Callbacks {
    fn pre_script(&mut self, _node: &Script) -> () {}
    fn pre_stmt(&mut self, _node: &Stmt) -> () {}
    fn pre_expr(&mut self, _node: &Expr) -> () {}
    fn pre_decl(&mut self, _node: &Decl) -> () {}
    fn pre_fun(&mut self, _node: &Fun) -> () {}
    fn pre_prop_val(&mut self, _node: &PropVal) -> () {}
    fn post_script(&mut self, _node: &Script) -> () {}
    fn post_stmt(&mut self, _node: &Stmt) -> () {}
    fn post_expr(&mut self, _node: &Expr) -> () {}
    fn post_decl(&mut self, _node: &Decl) -> () {}
    fn post_fun(&mut self, _node: &Fun) -> () {}
    fn post_prop_val(&mut self, _node: &PropVal) -> () {}
}

impl<'a, C: Callbacks> Walker<'a, C> {
    pub fn new(ast: &'a Script, callbacks: C) -> Walker<'a, C> {
        Walker { ast, callbacks }
    }

    pub fn walk(mut self) -> C {
        self.walk_script();
        self.callbacks
    }

    fn walk_script(&mut self) -> () {
        self.callbacks.pre_script(&self.ast);
        for item in &self.ast.body {
            self.walk_stmt_item(item);
        }
        self.callbacks.post_script(&self.ast);
    }

    fn walk_stmt_item(&mut self, item: &StmtListItem) -> () {
        match item {
            &StmtListItem::Stmt(ref stmt) => self.walk_stmt(stmt),
            &StmtListItem::Decl(ref decl) => self.walk_decl(decl),
        }
    }

    fn walk_stmt(&mut self, stmt: &Stmt) -> () {
        self.callbacks.pre_stmt(stmt);
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
        self.callbacks.post_stmt(stmt);
    }

    fn walk_decl(&mut self, decl: &Decl) -> () {
        self.callbacks.pre_decl(decl);
        let &Decl::Fun(ref fun) = decl;
        self.walk_fun(fun);
        self.callbacks.post_decl(decl);
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
        self.callbacks.pre_expr(expr);
        match expr {
            // TODO move this into a callback
            // and move the walk_* functions to generic AST walker
            &Expr::Call(_, ref callee, ref args) => {
                self.walk_expr(callee);
                for arg in args {
                    self.walk_expr(arg);
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
        self.callbacks.post_expr(expr);
    }

    fn walk_fun(&mut self, fun: &Fun) -> () {
        self.callbacks.pre_fun(fun);
        for item in &fun.body {
            self.walk_stmt_item(&item);
        }
        self.callbacks.post_fun(fun);
    }
}
