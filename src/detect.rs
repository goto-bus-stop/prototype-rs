use easter::stmt::{StmtListItem, Stmt};
use easter::decl::Dtor;
use easter::expr::Expr;
use easter::id::Id;
use easter::prog::Script;

pub fn detect(ast: Script) -> Vec<String> {
    let mut modules = vec![];
    for item in ast.body {
        match item {
            // var
            StmtListItem::Stmt(Stmt::Var(_, decls, _semi)) => {
                detect_var(decls, &mut modules);
            },
            _ => (),
        }
    }
    modules
}

fn detect_var(decls: Vec<Dtor>, modules: &mut Vec<String>) -> () {
    for decl in decls {
        match decl {
            Dtor::Simple(_, _id, Some(Expr::Call(_, expr, args))) => {
                if is_require_name(*expr) {
                    match args.first().unwrap() {
                        &Expr::String(_, ref val) => {
                            modules.push(val.value.clone());
                        },
                        _ => (),
                    }
                }
            },
            _ => (),
        }
    }
}

fn is_require_name(id: Expr) -> bool {
    match id {
        Expr::Id(Id { name: fn_name, .. }) => fn_name.as_ref() == "require",
        _ => false,
    }
}
