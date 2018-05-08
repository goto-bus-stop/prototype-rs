extern crate easter;

mod walk;

use easter::expr::{Expr, ExprListItem};
use easter::id::Id;
use easter::stmt::Script;
use walk::{Walker, Callbacks};

/// Find require() calls in an ESTree Script node (from the easter crate).
///
/// # Examples
///
/// ```rust
/// use esprit::script;
/// use estree_detect_requires::detect;
///
/// let requires = detect(&script("var x = require('y')").unwrap());
/// assert_eq!(requires, vec!["y"]);
/// ```
pub fn detect(ast: &Script) -> Vec<String> {
    let walker = Walker::new(ast, FindRequires::new());
    let find = walker.walk();

    find.get_modules()
}

/// A tree walker that tracks require() calls.
struct FindRequires {
    modules: Vec<String>,
}

impl FindRequires {
    pub fn new() -> FindRequires {
        FindRequires { modules: vec![] }
    }
    pub fn get_modules(self) -> Vec<String> {
        self.modules
    }
}

impl Callbacks for FindRequires {
    fn pre_expr(&mut self, expr: &Expr) -> () {
        if let Expr::Call(_, ref callee, ref args) = *expr {
            if is_require_name(callee) {
                if let Some(&ExprListItem::Expr(Expr::String(_, ref val))) = args.first() {
                    self.modules.push(val.value.clone());
                }
            }
        }
    }
}

fn is_require_name(id: &Expr) -> bool {
    if let Expr::Id(Id { name: ref fn_name, .. }) = *id {
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

    #[test]
    fn detects_require_in_member_expression_object() {
        assert_eq!(detect(&script("require('util').inherits").unwrap()), vec!["util"]);
    }
}
