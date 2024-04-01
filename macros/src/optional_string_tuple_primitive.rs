use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, DeriveInput, Lit};

use crate::utils::{is_data_struct, retrieve_name_values_list};

pub(crate) fn impl_tuple_optional_string_primitive(
    input: DeriveInput,
) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // フィールドを持つ構造体であることを確認
    is_data_struct(&input, "TupleOptionalStringPrimitive")?;
    // 検証属性を取得
    let primitive_attr = retrieve_primitive_attr(&input)?;
    // try_from_strメソッドを実装
    let try_from_str = impl_try_from_str_method(&primitive_attr);

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            # try_from_str

            pub fn value(&self) -> ::core::option::Option<&::std::primitive::str> {
                self.0.as_deref()
            }

            pub fn none() -> Self {
                Self(::core::option::Option::None)
            }

            pub fn is_some(&self) -> ::core::primitive::bool {
                self.0.is_some()
            }

            pub fn is_none(&self) -> ::core::primitive::bool {
                self.0.is_none()
            }
        }

        impl ::core::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match &self.0 {
                    ::core::option::Option::Some(value) => ::std::write!(f, "{}", value),
                    ::core::option::Option::None => ::std::write!(f, "None"),
                }
            }
        }

        impl ::core::convert::TryFrom<::std::string::String> for #ident {
            type Error = DomainError;

            fn try_from(value: ::std::string::String) -> ::core::result::Result<Self, Self::Error> {
                Self::try_from_str(&value)
            }
        }

        impl ::core::convert::TryFrom<::core::option::Option<::std::string::String>> for #ident {
            type Error = DomainError;

            fn try_from(value: ::core::option::Option<::std::string::String>) -> ::core::result::Result<Self, Self::Error> {
                match value {
                    ::core::option::Option::Some(value) => Self::try_from_str(&value),
                    ::core::option::Option::None => Ok(Self(::core::option::Option::None)),
                }
            }
        }

        impl ::core::convert::TryFrom<&::std::primitive::str> for #ident {
            type Error = DomainError;

            fn try_from(value: &::std::primitive::str) -> ::core::result::Result<Self, Self::Error> {
                Self::try_from_str(value)
            }
        }

        impl ::core::convert::TryFrom<::core::option::Option<&::std::primitive::str>> for #ident {
            type Error = DomainError;

            fn try_from(value: ::core::option::Option<&::std::primitive::str>) -> ::core::result::Result<Self, Self::Error> {
                match value {
                    ::core::option::Option::Some(value) => Self::try_from_str(value),
                    ::core::option::Option::None => ::core::result::Result::Ok(Self(None)),
                }
            }
        }
    })
}

fn impl_try_from_str_method(primitive_attr: &PrimitiveAttr) -> TokenStream2 {
    let mut validation_tokens: Vec<TokenStream2> = vec![];
    let name = &primitive_attr.name;
    if let Some(min) = primitive_attr.min {
        validation_tokens.push(quote! {
            if value.len() < #min {
                return ::core::result::Result::Err(
                    DomainError::Validation(
                        ::std::format!(
                            "{}は{}文字以上の文字列を指定してください。", #name, #min
                        ).into()
                    )
                );
            }
        });
    }
    if let Some(max) = primitive_attr.max {
        validation_tokens.push(quote! {
            if #max < value.len() {
                return ::core::result::Result::Err(
                    DomainError::Validation(
                        ::std::format!(
                            "{}は{}文字以下の文字列を指定してください。", #name, #max
                        ).into()
                    )
                );
            }
        });
    }
    if let Some(regex) = &primitive_attr.regex {
        validation_tokens.push(quote! {
            let re = regex::Regex::new(#regex).unwrap();
            if !re.is_match(value) {
                return ::core::result::Result::Err(
                    DomainError::Validation(
                        ::std::format!(
                            "{}に指定した文字列の形式が誤っています。", #name
                        ).into()
                    )
                );
            }
        });
    }

    quote! {
        pub fn try_from_str(value: &::std::primitive::str) -> DomainResult<Self> {
            let value = value.trim();
            if value.is_empty() {
                return Ok(Self(None));
            }

            #(#validation_tokens)*

            ::core::result::Result::Ok(Self(::core::option::Option::Some(value.to_owned())))
        }
    }
}

#[derive(Default)]
struct PrimitiveAttr {
    name: String,
    regex: Option<String>,
    min: Option<usize>,
    max: Option<usize>,
}

fn retrieve_primitive_attr(input: &DeriveInput) -> syn::Result<PrimitiveAttr> {
    let mut name: Option<String> = None;
    let mut regex: Option<String> = None;
    let mut min: Option<usize> = None;
    let mut max: Option<usize> = None;

    // primitive属性の名前と値を取得
    let name_values_list = retrieve_name_values_list(&input.attrs, "primitive")?;

    // primitive属性が2つ以上指定されている場合はエラー
    if name_values_list.len() > 1 {
        return Err(syn::Error::new(
            input.attrs[1].span(),
            "only one primitive can be specified",
        ));
    }

    let name_values = &name_values_list[0];
    // すべてのキーが1つだけ指定されているか確認
    for (ident, values) in name_values.iter() {
        if values.len() > 1 {
            return Err(syn::Error::new(
                ident.span(),
                format!("only one {} can be specified", ident),
            ));
        }
    }

    // nameの値を取得
    if let Some(values) = name_values.get(&format_ident!("name")) {
        if let Lit::Str(s) = &values[0] {
            name = Some(s.value());
        }
    }
    // regexの値を取得
    if let Some(lits) = name_values.get(&format_ident!("regex")) {
        if let Lit::Str(lit_str) = &lits[0] {
            regex = Some(lit_str.value());
        }
    }
    // minの値を取得
    if let Some(lits) = name_values.get(&format_ident!("min")) {
        if let Lit::Int(lit_int) = &lits[0] {
            min = Some(lit_int.base10_parse::<usize>()?);
        }
    }
    // maxの値を取得
    if let Some(lits) = name_values.get(&format_ident!("max")) {
        if let Lit::Int(lit_int) = &lits[0] {
            max = Some(lit_int.base10_parse::<usize>()?);
        }
    }

    // nameが指定されていない場合はエラー
    if name.is_none() {
        return Err(syn::Error::new(
            input.ident.span(),
            "primitive attribute must have `name`",
        ));
    }

    Ok(PrimitiveAttr {
        name: name.unwrap(),
        regex,
        min,
        max,
    })
}
