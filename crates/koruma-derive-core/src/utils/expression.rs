use super::*;

/// Check if an expression is a simple identifier (bare field name like `password`).
///
/// If so, return the identifier. This is used to detect field references in validator args.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::expr_as_simple_ident;
/// use syn::Expr;
///
/// let expr: Expr = parse_quote!(password);
/// let ident = expr_as_simple_ident(&expr);
/// assert_eq!(ident.unwrap().to_string(), "password");
///
/// let expr2: Expr = parse_quote!(self.password);
/// let ident2 = expr_as_simple_ident(&expr2);
/// assert!(ident2.is_none());
/// ```
pub fn expr_as_simple_ident(expr: &Expr) -> Option<&Ident> {
    if let Expr::Path(expr_path) = expr
        && expr_path.qself.is_none()
        && expr_path.path.segments.len() == 1
        && expr_path.path.segments[0].arguments.is_empty()
    {
        Some(&expr_path.path.segments[0].ident)
    } else {
        None
    }
}
