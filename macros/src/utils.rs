use std::collections::HashMap;

use syn::{
    punctuated::Punctuated, Attribute, Data, DataStruct, DeriveInput, Expr, Field, Fields,
    FieldsNamed, Ident, Lit,
};

use crate::types::{CommaPunctuatedFields, CommaPunctuatedNameValues};

/// フィールドを持つ構造体であることを確認する。
pub(crate) fn is_data_struct<'a>(
    input: &'a DeriveInput,
    macro_name: &str,
) -> syn::Result<&'a DataStruct> {
    match &input.data {
        Data::Struct(data_struct) => Ok(data_struct),
        _ => Err(syn::Error::new(
            input.ident.span(),
            format!("{} is expected a struct", macro_name),
        )),
    }
}

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

/// 特定の属性の、特定の名前の値を文字列で取得する。
///
/// # 引数
///
/// * `attr` - 属性
/// * `attr_name` - 取得する属性の名前
/// * `value_name` - 上記属性内に定義された、値を取得する名前
///
/// # 戻り値
///
/// 値
pub fn inspect_name_value_str(
    attr: &Attribute,
    attr_name: &str,
    value_name: &str,
) -> syn::Result<Option<String>> {
    // 属性でない場合
    if !attr.path().is_ident(attr_name) {
        return Ok(None);
    }
    // 属性内にある名前と値のリストを取得
    let name_values: CommaPunctuatedNameValues = attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err(|err| {
            syn::Error::new_spanned(attr, format!("failed to parse attribute: {}", err))
        })?;

    // 属性内の名前を検索
    for name_value in name_values.iter() {
        if name_value.path.is_ident(value_name) {
            match &name_value.value {
                Expr::Lit(expr_lit) => match &expr_lit.lit {
                    Lit::Str(value) => return Ok(Some(value.value())),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            attr,
                            format!("expected `({} = \"...\")`", value_name),
                        ))
                    }
                },
                _ => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format!("expected `({} = \"...\")`", value_name),
                    ))
                }
            }
        }
    }

    Ok(None)
}

/// 識別子に付けられた複数の属性の中から、特定の属性の、特定の名前の値を文字列で取得する。
///
/// # 引数
///
/// * `attrs` - 識別子に付けられた複数の属性
/// * `attr_name` - 取得する属性の名前
/// * `value_name` - 上記属性内に定義された、値を取得する名前
///
/// # 戻り値
///
/// 値
pub fn inspect_attr_and_name_str(
    attrs: &[Attribute],
    attr_name: &str,
    value_name: &str,
) -> syn::Result<Option<String>> {
    for attr in attrs {
        if let Some(value) = inspect_name_value_str(attr, attr_name, value_name)? {
            return Ok(Some(value));
        }
    }

    Ok(None)
}

/// 識別子に付与された属性について、それらの名前と値をすべて取得する。
///
/// ```text
/// [attr(name1 = 1, name1 = 2, name2 = "val")]
/// [attr(name1 = 3]
/// [attr(foo = "a")]
/// [attr(bar)]
/// [any(name1 = 1)]
/// struct A
///
/// let name_values_list = retrieve_name_values_list(attrs, "attr");
///
/// [
///     [
///         name1, [1, 2],
///         name2, ["val"],
///     ],
///     [
///         name1, [3],
///     ],
///     [
///         foo, ["a"],
///     ],
/// ]
/// ```
///
/// # 引数
///
/// * `attrs` - 識別子に付けられた属性のベクタ
///
/// # 戻り値
///
/// 名前と値を格納した`HashMap`
pub fn retrieve_name_values_list(
    attrs: &[Attribute],
    attr_name: &str,
) -> syn::Result<Vec<HashMap<Ident, Vec<Lit>>>> {
    let mut result = vec![];
    for attr in attrs {
        // 指定された属性でない場合は、次の属性を処理
        if !attr.path().is_ident(attr_name) {
            continue;
        }
        // 属性内にある名前と値のリストを取得
        let name_values: CommaPunctuatedNameValues = attr
            .parse_args_with(Punctuated::parse_terminated)
            .map_err(|err| {
                syn::Error::new_spanned(attr, format!("failed to parse attribute: {}", err))
            })?;
        // 属性内の名前を検索
        let mut nvs: HashMap<Ident, Vec<Lit>> = HashMap::new();
        for name_value in name_values.iter() {
            let ident = name_value.path.get_ident().unwrap().to_owned();
            if let Expr::Lit(expr_lit) = &name_value.value {
                nvs.entry(ident).or_default().push(expr_lit.lit.to_owned());
            }
        }
        if !nvs.is_empty() {
            result.push(nvs);
        }
    }
    Ok(result)
}
