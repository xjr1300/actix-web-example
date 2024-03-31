use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, DeriveInput, Ident, Lit};

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
    let try_from_str = impl_try_from_str_method(ident, &validation);

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            # try_from_str

            pub fn value(&self) -> std::option::Option<&str> {
                self.0.as_deref()
            }

            pub fn none() -> Self {
                Self(std::option::Option::None)
            }

            pub fn is_none(&self) -> std::primitive::bool {
                self.0.is_none()
            }
        }

        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match &self.0 {
                    std::option::Option::Some(value) => write!(f, "{}", value),
                    std::option::Option::None => write!(f, "None"),
                }
            }
        }

        impl std::convert::TryFrom<std::string::String> for #ident {
            type Error = DomainError;

            fn try_from(value: std::string::String) -> std::result::Result<Self, Self::Error> {
                Self::try_from_str(&value)
            }
        }

        impl std::convert::TryFrom<std::option::Option<std::string::String>> for #ident {
            type Error = DomainError;

            fn try_from(value: Option<std::string::String>) -> std::result::Result<Self, Self::Error> {
                match value {
                    std::option::Option::Some(value) => Self::try_from_str(&value),
                    std::option::Option::None => Ok(Self(None)),
                }
            }
        }

        impl std::convert::TryFrom<&str> for #ident {
            type Error = DomainError;

            fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
                Self::try_from_str(value)
            }
        }

        impl std::convert::TryFrom<Option<&str>> for #ident {
            type Error = DomainError;

            fn try_from(value: std::option::Option<&str>) -> std::result::Result<Self, Self::Error> {
                match value {
                    Some(value) => Self::try_from_str(value),
                    None => Ok(Self(None)),
                }
            }
        }
    })
}

fn impl_try_from_str_method(ident: &Ident, validation: &Validation) -> TokenStream2 {
    let mut validation_tokens: Vec<TokenStream2> = vec![];
    if let Some(min) = validation.min {
        validation_tokens.push(
            quote! {
                if value.len() < #min {
                    return Err(DomainError::Validation(format!("the string length of {} must be at least {} characters", stringify!(ident), #min).into()));
                }
            }
        );
    }
    if let Some(max) = validation.max {
        validation_tokens.push(
            quote! {
                if #max < value.len() {
                    return Err(DomainError::Validation(format!("the string length of {} must be {} characters or less", stringify!(ident), #max)into()));
                }
            }
        );
    }
    if let Some(regex) = &validation.regex {
        validation_tokens.push(quote! {
            let re = regex::Regex::new(#regex).unwrap();
            if !re.is_match(value) {
                return Err(DomainError::Validation(
                    format!("{} must match the regular expression", stringify!(ident)).into(),
                ));
            }
        });
    }

    quote! {
        pub fn try_from_str(value: &str) -> DomainResult<Self> {
            let value = value.trim();
            #(#validation_tokens)*

            Ok(Self(Some(value.to_owned())))
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
            result.min = Some(n.base10_parse::<usize>()?);
        }
    }

    Ok(result)
}
