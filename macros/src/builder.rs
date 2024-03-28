use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments, Attribute, Data,
    DataStruct, DeriveInput, Error, Expr, Fields, FieldsNamed, GenericArgument, Ident, Lit, Path,
    PathArguments, PathSegment, Result, Type, TypePath, Visibility,
};

use crate::types::CommaPunctuatedNameValues;

pub(crate) fn impl_builder(input: DeriveInput) -> Result<TokenStream2> {
    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        let ident = input.ident;
        let vis = input.vis;
        let builder = format_ident!("{}Builder", ident);
        let instance = format_ident!("{}", "instance");
        let validator = match inspect_attr_and_name(&input.attrs, "builder_validation", "func")? {
            Some(ident) => {
                quote!(
                    #ident(&#instance)?;
                )
            }
            None => quote!(),
        };
        let fields = named
            .iter()
            .map(|f| (f.ident.as_ref().expect("field have ident"), &f.ty));
        let idents = fields.clone().map(|(ident, _)| ident);
        let builder_fields = fields
            .clone()
            .map(|(ident, ty)| quote! {#ident: ::core::option::Option<#ty>});
        let builder_init_fields = fields.clone().map(builder_init_field);
        let each_attributes = named
            .iter()
            .map(|f| match f.attrs.first() {
                Some(attr) => inspect_name_value(attr, "builder", "each"),
                None => Ok(None),
            })
            .collect::<Result<Vec<_>>>()?;
        let builder_methods = fields
            .clone()
            .zip(each_attributes)
            .map(|((ident, ty), maybe_each)| impl_builder_method(&vis, ident, ty, maybe_each));

        Ok(quote! {
            #vis struct #builder {
                #(#builder_fields),*
            }

            impl #builder {
                #(#builder_methods)*

                #vis fn build(&mut self) -> ::core::result::Result<
                    #ident,
                    ::std::boxed::Box<dyn ::std::error::Error>
                >
                {
                    let #instance = #ident {
                        #(
                            #idents: self.#idents.take().ok_or_else(||
                                format!("{} is not provided", stringify!(#idents))
                            )?
                        ),*
                    };
                    #validator

                    Ok(#instance)
                }
            }

            impl #ident {
                #vis fn builder() -> #builder {
                    #builder {
                        #(#builder_init_fields),*
                    }
                }
            }
        })
    } else {
        Err(Error::new(input.span(), "only struct supported"))
    }
}

fn impl_builder_method(
    vis: &Visibility,
    ident: &Ident,
    ty: &Type,
    each: Option<Ident>,
) -> TokenStream2 {
    let has_each = each.is_some();
    match field_type(ty) {
        FieldType::Option(inner_ty) => {
            quote! {
                #vis fn #ident (&mut self, #ident: #inner_ty) -> &mut Self {
                    self.#ident = ::core::option::Option::Some(
                        ::core::option::Option::Some(#ident)
                    );
                    self
                }
            }
        }
        FieldType::Vec(inner_ty) if has_each => {
            let each = each.unwrap();
            quote! {
                #vis fn #each (&mut self, #each: #inner_ty) -> &mut Self {
                    self.#ident.as_mut().map(|v| v.push(#each));
                    self
                }
            }
        }
        _ => {
            quote! {
                #vis fn #ident (&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = ::core::option::Option::Some(#ident);
                    self
                }
            }
        }
    }
}

fn builder_init_field((ident, ty): (&Ident, &Type)) -> TokenStream2 {
    match field_type(ty) {
        FieldType::Option(_inner_ty) => {
            quote! { #ident: ::core::option::Option::Some(::core::option::Option::None)}
        }
        FieldType::Vec(_inner_ty) => {
            quote! { #ident: ::core::option::Option::Some(::std::vec::Vec::new())}
        }
        FieldType::Raw => {
            quote! { #ident: ::core::option::Option::None}
        }
    }
}

fn field_type(ty: &Type) -> FieldType {
    if let Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon,
            segments,
        },
    }) = ty
    {
        if leading_colon.is_none() && segments.len() == 1 {
            if let Some(PathSegment {
                ident,
                arguments:
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
            }) = segments.first()
            {
                if let (1, Some(GenericArgument::Type(t))) = (args.len(), args.first()) {
                    if ident == "Option" {
                        return FieldType::Option(t.clone());
                    } else if ident == "Vec" {
                        return FieldType::Vec(t.clone());
                    }
                }
            }
        }
    }
    FieldType::Raw
}

enum FieldType {
    Raw,
    Option(Type),
    Vec(Type),
}

fn inspect_name_value(
    attr: &Attribute,
    attr_name: &str,
    value_name: &str,
) -> Result<Option<Ident>> {
    // 属性でない場合
    if !attr.path().is_ident(attr_name) {
        return Ok(None);
    }
    // builder属性内にある名前と値のリストを取得
    let name_values: CommaPunctuatedNameValues = attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err(|err| {
            syn::Error::new_spanned(attr, format!("failed to parse builder attribute: {}", err))
        })?;

    // builder属性内の名前を`を検索
    for name_value in name_values.iter() {
        if name_value.path.is_ident(value_name) {
            match &name_value.value {
                Expr::Lit(expr_lit) => match &expr_lit.lit {
                    Lit::Str(value) => return Ok(Some(format_ident!("{}", value.value()))),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            attr,
                            format!("expected `builder({} = \"...\")`", value_name),
                        ))
                    }
                },
                _ => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format!("expected `builder({} = \"...\")`", value_name),
                    ))
                }
            }
        }
    }

    Ok(None)
}

fn inspect_attr_and_name(
    attrs: &[Attribute],
    attr_name: &str,
    value_name: &str,
) -> Result<Option<Ident>> {
    for attr in attrs {
        if let Some(ident) = inspect_name_value(attr, attr_name, value_name)? {
            return Ok(Some(ident));
        }
    }

    Ok(None)
}
