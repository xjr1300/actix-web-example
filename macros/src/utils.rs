use syn::{Attribute, Data, DataStruct, DeriveInput, Expr, Field, Fields, FieldsNamed, Lit};

use crate::types::CommaPunctuatedFields;

/// 構造体のフィールドを取得する。
pub(crate) fn retrieve_struct_fields(input: &DeriveInput) -> syn::Result<CommaPunctuatedFields> {
    match input.clone().data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => Ok(named),
        _ => Err(syn::Error::new_spanned(
            input,
            "expected struct has name fields",
        )),
    }
}

pub(crate) struct FieldAttrPair<'a> {
    /// フィールド
    pub field: &'a Field,
    /// 属性
    pub attr: &'a Attribute,
}

/// 指定された属性が付与されたフィールドとその属性を取得する。
pub(crate) fn retrieve_field_attrs_by_names<'a>(
    fields: &'a CommaPunctuatedFields,
    names: &[&str],
) -> Vec<FieldAttrPair<'a>> {
    // 指定された属性が付与されたフィールドとその属性を格納するベクタ
    let mut result = vec![];
    // 構造体のフィールドを走査
    for field in fields {
        field.attrs.iter().for_each(|attr| {
            // 指定された属性が付与されたフィールドか確認
            for name in names {
                if attr.path().is_ident(name) {
                    result.push(FieldAttrPair { field, attr });
                }
            }
        });
    }

    result
}

pub(crate) fn expr_to_string(expr: Option<Expr>) -> Option<String> {
    let expr = expr.as_ref()?;
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(lit_str) => Some(lit_str.value()),
            _ => None,
        },
        _ => None,
    }
}
