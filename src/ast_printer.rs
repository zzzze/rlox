use super::expr::{Visitor, Expr, Unary, Binary, Literal, Grouping};

pub struct AstPrinter;

impl Visitor<String> for AstPrinter {
    fn visit_unary_expr(&mut self, expr: &Unary) -> String {
        let exprs = [expr.right.as_ref()];
        self.parenthesize(&expr.operator.lexeme, &exprs)
    }

    fn visit_binary_expr(&mut self, expr: &Binary) -> String {
        let exprs = [expr.left.as_ref(), expr.right.as_ref()];
        self.parenthesize(&expr.operator.lexeme, &exprs)
    }

    fn visit_literal_expr(&mut self, expr: &Literal) -> String {
        format!("{}", expr.value)
    }

    fn visit_grouping_expr(&mut self, expr: &Grouping) -> String {
        let exprs = [expr.expression.as_ref()];
        self.parenthesize("group", &exprs)
    }
}

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        expr.accept(self as &mut dyn Visitor<String>)
    }

    fn parenthesize(&mut self, name: &str, exprs: &[&Expr]) -> String {
        let mut result = String::new();
        result.push_str("(");
        result.push_str(name);
        for expr in exprs {
            result.push_str(" ");
            result.push_str(&expr.accept(self as &mut dyn Visitor<String>));
        }
        result.push_str(")");
        return result;
    }
}
