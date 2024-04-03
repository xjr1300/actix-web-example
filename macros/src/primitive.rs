use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Attribute, DataStruct, DeriveInput, Fields, FieldsNamed, Ident, Lit};

use crate::utils::{is_data_struct, retrieve_name_values_list};

pub(crate) fn impl_primitive_display(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // フィールドを持つ構造体であることを確認
    let data_struct = is_data_struct(&input, "PrimitiveDisplay")?;

    // 名前付きフィールドを取得して、タプル構造体、またはユニット構造体でないことを確認
    let fields = retrieve_named_fields(ident, data_struct, "PrimitiveDisplay")?;

    // 構造体が`value`フィールドを持つか確認
    if !has_value_field(fields) {
        return Err(syn::Error::new(
            ident.span(),
            "PrimitiveDisplay must have the `value` field",
        ));
    }

    Ok(quote! {
        impl #impl_generics ::std::fmt::Display for #ident #ty_generics #where_clause{
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", self.value)
            }
        }
    })
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

pub(crate) fn impl_string_primitive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // ドメイン・プリミティブの名前を取得
    let StringPrimitiveAttr { name, message } = retrieve_primitive_info(ident, &input.attrs)?;

    // フィールドを持つ構造体であることを確認
    let data_struct = is_data_struct(&input, "StringPrimitive")?;

    // 名前付きフィールドを取得して、タプル構造体、またはユニット構造体でないことを確認
    let fields = retrieve_named_fields(ident, data_struct, "StringPrimitive")?;

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
            pub fn new<T: ::std::string::ToString>(value: T) -> DomainResult<Self> {
                let value = value.to_string().trim().to_string();
                if value.is_empty() {
                    return ::core::result::Result::Err(
                        DomainError::DomainRule(format!("{}は空文字を指定できません。", #name).into())
                    );
                }
                let instance = Self {
                    value,
                };
                match instance.validate() {
                    ::core::result::Result::Ok(_) => ::core::result::Result::Ok(instance),
                    ::core::result::Result::Err(_) => ::core::result::Result::Err(
                        DomainError::DomainRule(#message.into())
                    ),
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

struct StringPrimitiveAttr {
    name: String,
    message: String,
}

/// ドメイン・プリミティブの属性を取得する。
///
/// ```text
/// #[derive(StringPrimitive)]
/// #[primitive(name = "プリミティブ", error = "エラー")]
/// pub struct Foo { ... }
/// ```
///
/// 上記`ThisIsPrimitiveName`を取得する。
fn retrieve_primitive_info(ident: &Ident, attrs: &[Attribute]) -> syn::Result<StringPrimitiveAttr> {
    let mut name: Option<String> = None;
    let mut message: Option<String> = None;

    let name_value_list = retrieve_name_values_list(attrs, "primitive")?;
    // primitive属性が付与されていない場合はエラー
    if name_value_list.is_empty() {
        return Err(syn::Error::new(
            ident.span(),
            "domain primitive must have the `primitive` attribute",
        ));
    }
    // primitive属性が1つより多く付与されている場合はエラー
    if 1 < name_value_list.len() {
        return Err(syn::Error::new(
            ident.span(),
            "domain primitive only have one `primitive` attribute",
        ));
    }
    // primitive属性の名前と値の組みは2つのみ
    let name_values = &name_value_list[0];
    if name_values.len() != 2 {
        return Err(syn::Error::new(
            ident.span(),
            "`primitive` attributes must have `name` and `message`",
        ));
    }

    // nameの値を取得
    if let Some(lits) = name_values.get(&format_ident!("name")) {
        if let Lit::Str(lit_str) = &lits[0] {
            name = Some(lit_str.value());
        }
    }
    // errorの値を取得
    if let Some(lits) = name_values.get(&format_ident!("message")) {
        if let Lit::Str(lit_str) = &lits[0] {
            message = Some(lit_str.value());
        }
    }

    if name.is_none() {
        return Err(syn::Error::new(
            ident.span(),
            "`primitive` must have `name`",
        ));
    }
    if message.is_none() {
        return Err(syn::Error::new(
            ident.span(),
            "`primitive` must have `message`",
        ));
    }

    Ok(StringPrimitiveAttr {
        name: name.unwrap(),
        message: message.unwrap(),
    })
}
