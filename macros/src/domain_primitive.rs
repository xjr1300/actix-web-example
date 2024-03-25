use std::str::FromStr as _;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{Data, DataStruct, DeriveInput, Expr, Field, Fields, FieldsNamed, Ident, Lit, LitStr};

use crate::types::CommaPunctuatedNameValues;

const MACRO_NAME: &str = "DomainPrimitive";

pub(crate) fn impl_domain_primitive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // フィールドを持つ構造体であることを確認
    let data_struct = is_data_struct(&input, MACRO_NAME)?;

    // 名前付きフィールドを取得して、タプル構造体、またはユニット構造体でないことを確認
    let fields = retrieve_named_fields(ident, data_struct, MACRO_NAME)?;

    // `value`フィールドを取得
    let field = retrieve_value_field(ident, fields, MACRO_NAME)?;
    let vis = &field.vis;
    let ty = &field.ty;

    // `value_getter`属性に定義された名前と値のリストを取得
    let name_values = retrieve_name_values_in_field_attr(field, "value_getter");
    if name_values.is_none() {
        return Err(syn::Error::new(
            field.span(),
            "value_getter attribute should have name values",
        ));
    }
    let name_values = name_values.unwrap();

    // `value_getter`属性の値を取得
    let attr_value = retrieve_value_getter_attr(&name_values)?;
    let token = match attr_value.is_val {
        true => {
            quote! {
                #vis fn value(&self) -> #ty {
                    self.value
                }
            }
        }
        false => match attr_value.rty {
            None => {
                quote! {
                    #vis fn value(&self) -> &#ty {
                        &self.value
                    }
                }
            }
            Some(rty) => {
                let rty = TokenStream2::from_str(&rty).unwrap();
                quote! {
                    #vis fn value(&self) -> #rty {
                        &self.value
                    }
                }
            }
        },
    };

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause{
            #token
        }
    })
}

pub(crate) fn impl_primitive_display(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // フィールドを持つ構造体であることを確認
    let data_struct = is_data_struct(&input, MACRO_NAME)?;

    // 名前付きフィールドを取得して、タプル構造体、またはユニット構造体でないことを確認
    let fields = retrieve_named_fields(ident, data_struct, MACRO_NAME)?;

    // 構造体が`value`フィールドを持つか確認
    if !has_value_field(fields) {
        return Err(syn::Error::new(
            ident.span(),
            "PrimitiveDisplay must have the `value` field",
        ));
    }

    Ok(quote! {
        impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause{
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.value)
            }
        }
    })
}
/// フィールドを持つ構造体であることを確認する。
fn is_data_struct<'a>(input: &'a DeriveInput, macro_name: &str) -> syn::Result<&'a DataStruct> {
    match &input.data {
        Data::Struct(data_struct) => Ok(data_struct),
        _ => Err(syn::Error::new(
            input.ident.span(),
            format!("{} is expected a struct", macro_name),
        )),
    }
}

/// 構造体の名前付きフィールドを取得する。
fn retrieve_named_fields<'a>(
    ident: &'a Ident,
    data_struct: &'a DataStruct,
    macro_name: &str,
) -> syn::Result<&'a FieldsNamed> {
    match &data_struct.fields {
        Fields::Named(fields) => Ok(fields),
        _ => Err(syn::Error::new(
            ident.span(),
            format!(
                "{} is expected a struct contain the `value` field",
                macro_name
            ),
        )),
    }
}

fn retrieve_value_field<'a>(
    ident: &'a Ident,
    fields: &'a FieldsNamed,
    macro_name: &str,
) -> syn::Result<&'a Field> {
    match fields
        .named
        .iter()
        .find(|f| *f.ident.as_ref().unwrap() == "value")
    {
        Some(field) => Ok(field),
        None => Err(syn::Error::new(
            ident.span(),
            format!(
                "{} is expected a struct contain the `value` field",
                macro_name
            ),
        )),
    }
}

/// 構造体のフィールドに付与された属性に定義された名前と値のリストを取得する。
///
/// # 引数
///
/// * `field` - 構造体のフィールド
/// * `attr_name` - 名前と値のリストを取得する構造体のフィールドに付与された属性の名前
///
/// # 戻り値
///
/// 名前と値のリスト
fn retrieve_name_values_in_field_attr(
    field: &Field,
    attr_name: &str,
) -> Option<CommaPunctuatedNameValues> {
    for attr in field.attrs.iter() {
        if attr.path().is_ident(attr_name) {
            let name_values: syn::Result<CommaPunctuatedNameValues> =
                attr.parse_args_with(Punctuated::parse_terminated);
            if let Ok(name_values) = name_values {
                return Some(name_values);
            }
        }
    }

    None
}

struct ValueGetterAttrValue {
    /// `value`メソッドが値を返すかを示すフラグ
    is_val: bool,
    /// `value`メソッドが参照を返すときの参照型を示す文字列
    rty: Option<String>,
}

/// 構造体の`value`フィールドに付与された`ValueGetter`マクロの属性を取得する。
///
/// * ret属性は必須で、値は"val"または"ref"
/// * rty属性はオプションで、値はマクロが実装する`value`メソッドが返す参照型を示す文字列
fn retrieve_value_getter_attr(
    name_values: &CommaPunctuatedNameValues,
) -> syn::Result<ValueGetterAttrValue> {
    // retキーの値を取得
    let ret_value = retrieve_lit_str_of_name(name_values, "ret");
    if ret_value.is_none() {
        return Err(syn::Error::new(
            name_values.span(),
            "value_getter must have ret",
        ));
    }
    let ret_value = ret_value.unwrap().value();
    // retキーの値が"val"または"ref"であるか確認
    if !["val", "ref"].contains(&ret_value.as_str()) {
        return Err(syn::Error::new(
            name_values.span(),
            "ret value should be `val` or `ref`",
        ));
    }
    let is_val = ret_value == "val";

    // rtyキーの値を取得
    let rty = retrieve_lit_str_of_name(name_values, "rty");
    let rty = rty.map(|rty| rty.value());

    Ok(ValueGetterAttrValue { is_val, rty })
}

/// 属性に付与されたキーと値について、指定された名前に指定された文字列リテラルを取得する。
///
/// # 引数
///
/// * `name_values` - フィールドに付与された属性に定義された名前と値のリスト
/// * `name` - 文字列リテラルを取得する名前
fn retrieve_lit_str_of_name<'a>(
    name_values: &'a CommaPunctuatedNameValues,
    name: &str,
) -> Option<&'a LitStr> {
    for name_value in name_values {
        if name_value.path.is_ident(name) {
            if let Expr::Lit(expr_lit) = &name_value.value {
                if let Lit::Str(value) = &expr_lit.lit {
                    return Some(value);
                }
            }
        }
    }

    None
}

pub(crate) fn impl_string_primitive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // フィールドを持つ構造体であることを確認
    let data_struct = is_data_struct(&input, MACRO_NAME)?;

    // 名前付きフィールドを取得して、タプル構造体、またはユニット構造体でないことを確認
    let fields = retrieve_named_fields(ident, data_struct, MACRO_NAME)?;

    // 構造体が`value`フィールドを持つか確認
    // FIXME: `value`フィールドが`String`型であることを確認する実装
    if !has_value_field(fields) {
        return Err(syn::Error::new(
            ident.span(),
            "StringPrimitive must have the `value` field of type `String`",
        ));
    }

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn new<T: std::string::ToString>(value: T) -> DomainResult<Self> {
                let value = value.to_string().trim().to_string();
                let instance = Self {
                    value,
                };
                match instance.validate() {
                    Ok(_) => Ok(instance),
                    Err(e) => Err(DomainError::DomainRule(format!("values is invalid: {e}").into())),
                }
            }
        }
    })
}

/// 構造体が`value`フィールドを持つか確認する。
fn has_value_field(fields: &FieldsNamed) -> bool {
    fields
        .named
        .iter()
        .any(|f| *f.ident.as_ref().unwrap() == "value")
}
