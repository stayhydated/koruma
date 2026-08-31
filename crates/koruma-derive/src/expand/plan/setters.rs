use super::*;

pub(crate) fn planned_setter_calls(
    calls: &[BuilderMethodCall],
    known_field_names: &[Ident],
) -> Result<Vec<PlannedSetterCall>, syn::Error> {
    let mut errors = ErrorBag::new();
    let mut planned = Vec::new();

    for call in calls {
        let mut planned_args = Vec::new();
        for arg in call.args() {
            match plan_setter_arg(arg, known_field_names) {
                Ok(arg) => planned_args.push(arg),
                Err(error) => errors.push(error),
            }
        }

        if planned_args.len() == call.args().len() {
            planned.push(PlannedSetterCall {
                method: call.method().clone(),
                args: planned_args,
            });
        }
    }

    errors.finish()?;
    Ok(planned)
}

pub(crate) fn plan_setter_arg(
    arg: &ValidatorSetterArg,
    known_field_names: &[Ident],
) -> Result<PlannedSetterArg, syn::Error> {
    match arg {
        ValidatorSetterArg::Expr(expr) => {
            if let Some(field_ident) = expr_as_simple_ident(expr)
                && known_field_names.iter().any(|name| name == field_ident)
            {
                return Err(syn::Error::new_spanned(
                    expr,
                    format!(
                        "bare field argument `{field_ident}` is ambiguous; use `self.{field_ident}.clone()` explicitly"
                    ),
                ));
            }

            Ok(PlannedSetterArg::Expr(expr.clone()))
        },
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedSetterArg {
    Expr(syn::Expr),
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedSetterCall {
    pub method: Ident,
    pub args: Vec<PlannedSetterArg>,
}
