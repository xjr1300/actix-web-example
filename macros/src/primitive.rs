use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, Attribute, DataStruct, DeriveInput, Expr, Field, Fields, FieldsNamed,
    Ident, Lit, MetaNameValue,
};

use crate::types::{CommaPunctuatedMetaList, CommaPunctuatedNameValues};
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
                        DomainError::Validation(format!("{}は空文字を指定できません。", #name).into())
                    );
                }
                let instance = Self {
                    value,
                };
                match instance.validate() {
                    ::core::result::Result::Ok(_) => ::core::result::Result::Ok(instance),
                    ::core::result::Result::Err(_) => ::core::result::Result::Err(
                        DomainError::Validation(#message.into())
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

pub(crate) fn impl_integer_primitive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // フィールドを持つ構造体であることを確認
    let data_struct = is_data_struct(&input, "IntegerPrimitive")?;
    // データ構造に付与された`primitive`属性の`name`の値を取得
    let name_values_list = retrieve_name_values_list(&input.attrs, "primitive")?;
    if name_values_list.is_empty() {
        return Err(syn::Error::new_spanned(
            &input,
            "PrimitiveInteger must have the `primitive` attribute",
        ));
    }
    let name_values = name_values_list
        .first()
        .unwrap()
        .get(&format_ident!("name"));
    if name_values.is_none() {
        return Err(syn::Error::new_spanned(
            &input,
            "`primitive` attribute must have the `name`",
        ));
    }
    let name_values = name_values.unwrap();
    let Lit::Str(name) = name_values.first().unwrap() else {
        return Err(syn::Error::new_spanned(
            &name_values[0],
            "`name` must be a string literal",
        ));
    };

    // タプル構造体でないことを確認
    if data_struct.fields.iter().any(|f| f.ident.is_none()) {
        return Err(syn::Error::new_spanned(
            &input,
            "PrimitiveInteger must be struct",
        ));
    }

    // `value`フィールドを持つか確認
    if data_struct.fields.len() != 1 {
        return Err(syn::Error::new_spanned(
            &input,
            "PrimitiveInteger can have at least one `value` field",
        ));
    }
    // `value`フィールドを取得
    let field = data_struct
        .fields
        .iter()
        .find(|f| f.ident.as_ref().unwrap() == "value")
        .ok_or(syn::Error::new_spanned(
            &input,
            "PrimitiveInteger must have one `value` field",
        ))?;
    // `value`フィールドの型を取得
    let ty = &field.ty;
    // `validate`属性内の`range`属性の`min`と`max`を取得
    let range = retrieve_validate_range_attr(field)?;
    // 値を検証する文を生成
    let min_token = match range.min {
        Some(min) => quote! {
            if value < #min {
                return ::core::result::Result::Err(
                    DomainError::Validation(format!("{}は{}以上の値を指定してください。", #name, #min).into())
                );
            }
        },
        _ => quote! {},
    };
    let max_token = match range.max {
        Some(max) => quote! {
            if #max < value {
                return ::core::result::Result::Err(
                    DomainError::Validation(format!("{}は{}以下の値を指定してください。", #name, #max).into())
                );
            }
        },
        _ => quote! {},
    };

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn new(value: #ty) -> DomainResult<Self> {
                #min_token
                #max_token

                Ok(Self {
                    value
                })
            }
        }
    })
}

#[derive(Default)]
struct ValidateRange {
    min: Option<i32>,
    max: Option<i32>,
}

/// `validate`属性内の`range`属性の`min`と`max`を取得する。
///
/// `#[validate(range(min = 0, max = 20))]`
///                         ^        ^^
fn retrieve_validate_range_attr(field: &Field) -> syn::Result<ValidateRange> {
    // `validate`属性を取得
    let validate_attr = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("validate"))
        .ok_or(syn::Error::new_spanned(
            field,
            "`value` field must have the `validate` attribute",
        ))?;

    // `validator`属性内の名前のリストを取得
    let meta_list: CommaPunctuatedMetaList = validate_attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err(|err| {
            syn::Error::new_spanned(validate_attr, format!("failed to parse attribute: {}", err))
        })?;
    // `validator`属性内の`range`属性を取得
    let range_attr = meta_list
        .iter()
        .find(|meta| meta.path.is_ident("range"))
        .ok_or(syn::Error::new_spanned(
            validate_attr,
            "`validate` attribute must have the `range`",
        ))?;
    // `range`属性内の`min`と`max`を取得
    let name_values: CommaPunctuatedNameValues = range_attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err(|err| {
            syn::Error::new_spanned(range_attr, format!("failed to parse attribute: {}", err))
        })?;
    // `min`と`max`を取得
    let mut range = ValidateRange::default();
    for nv in name_values.iter() {
        if nv.path.is_ident("min") {
            range.min = Some(retrieve_integer_from_name_value(nv)?);
        }
        if nv.path.is_ident("max") {
            range.max = Some(retrieve_integer_from_name_value(nv)?);
        }
    }
    if range.min.is_none() && range.max.is_none() {
        return Err(syn::Error::new_spanned(
            range_attr,
            "range must have at least either `min` or `max`",
        ));
    }

    Ok(range)
}

fn retrieve_integer_from_name_value(nv: &MetaNameValue) -> syn::Result<i32> {
    let Expr::Lit(expr_lit) = &nv.value else {
        return Err(syn::Error::new_spanned(
            nv,
            format!("the value of `{}` is integer", nv.path.get_ident().unwrap()),
        ));
    };
    let Lit::Int(n) = &expr_lit.lit else {
        return Err(syn::Error::new_spanned(
            nv,
            format!("the value of `{}` is integer", nv.path.get_ident().unwrap()),
        ));
    };

    n.base10_parse::<i32>()
}
