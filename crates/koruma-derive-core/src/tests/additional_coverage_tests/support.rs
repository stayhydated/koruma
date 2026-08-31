pub(super) use crate::{
    DataFieldKorumaAttr, DataFieldKorumaItem, FieldInfo, FieldModifierKind, KnownTypeShape,
    ParsedDataField, SetterDefault, SetterPresence, StructKorumaAttr, StructKorumaItem, StructMode,
    ValidatorAttr, ValidatorFieldRole, ValidatorLabel, ValidatorTargetSelector,
    contains_infer_type, expr_as_simple_ident, first_generic_arg, option_inner_type, parse_field,
    parse_struct_options, parse_validator_struct, substitute_infer_type,
    substitute_infer_type_from_source, type_to_ident, vec_inner_type,
};

pub(super) fn parse_field_info(field: &syn::Field) -> FieldInfo {
    let ParsedDataField::Participating(info) = parse_field(field, 0).expect("expected field parse")
    else {
        panic!("expected participating field")
    };
    info
}
