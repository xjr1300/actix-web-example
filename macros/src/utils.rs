use std::collections::HashMap;

use syn::{punctuated::Punctuated, Attribute, Data, DataStruct, DeriveInput, Expr, Ident, Lit};

use crate::types::CommaPunctuatedNameValues;

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
