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
    let validation = retrieve_primitive_validation_value(&input)?;
    // try_from_strメソッドを実装
    let try_from_str = impl_try_from_str_method(&ident.to_string(), &validation);

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            # try_from_str

            pub fn value(&self) -> ::core::option::Option<&::std::primitive::str> {
                self.0.as_deref()
            }

            pub fn none() -> Self {
                Self(::core::option::Option::None)
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

fn impl_try_from_str_method(name: &str, validation: &Validation) -> TokenStream2 {
    let mut validation_tokens: Vec<TokenStream2> = vec![];
    if let Some(min) = validation.min {
        validation_tokens.push(quote! {
            if value.len() < #min {
                return ::core::result::Result::Err(DomainError::Validation(::std::format!("the string length of {} must be at least {} characters", #name, #min).into()));
            }
        });
    }
    if let Some(max) = validation.max {
        validation_tokens.push(quote! {
            if #max < value.len() {
                return ::core::result::Result::Err(DomainError::Validation(::std::format!("the string length of {} must be {} characters or less", #name, #max).into()));
            }
        });
    }
    if let Some(regex) = &validation.regex {
        validation_tokens.push(quote! {
            let re = regex::Regex::new(#regex).unwrap();
            if !re.is_match(value) {
                return ::core::result::Result::Err(DomainError::Validation(
                    ::std::format!("{} must match the regular expression", #name).into(),
                ));
            }
        });
    }

    quote! {
        pub fn try_from_str(value: &::std::primitive::str) -> DomainResult<Self> {
            let value = value.trim();
            #(#validation_tokens)*

            ::core::result::Result::Ok(Self(::core::option::Option::Some(value.to_owned())))
        }
    }
}

#[derive(Default)]
struct Validation {
    regex: Option<String>,
    min: Option<usize>,
    max: Option<usize>,
}

fn retrieve_primitive_validation_value(input: &DeriveInput) -> syn::Result<Validation> {
    let mut result = Validation::default();

    // primitive_validation属性の名前と値を取得
    let name_values_list = retrieve_name_values_list(&input.attrs, "primitive_validation")?;

    // primitive_validation属性が指定されていない場合は検証しない
    if name_values_list.is_empty() {
        return Ok(result);
    }
    // primitive_validation属性が2つ以上指定されている場合はエラー
    if name_values_list.len() > 1 {
        return Err(syn::Error::new(
            input.attrs[1].span(),
            "only one primitive_validation can be specified",
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

    // regexの値を取得
    if let Some(values) = name_values.get(&format_ident!("regex")) {
        if let Lit::Str(s) = &values[0] {
            result.regex = Some(s.value());
        }
    }
    // minの値を取得
    if let Some(values) = name_values.get(&format_ident!("min")) {
        if let Lit::Int(n) = &values[0] {
            result.min = Some(n.base10_parse::<usize>()?);
        }
    }
    // maxの値を取得
    if let Some(values) = name_values.get(&format_ident!("max")) {
        if let Lit::Int(n) = &values[0] {
            result.max = Some(n.base10_parse::<usize>()?);
        }
    }

    Ok(result)
}
