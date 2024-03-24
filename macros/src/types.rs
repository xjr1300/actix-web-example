use syn::punctuated::Punctuated;
use syn::{MetaNameValue, Token};

/// `foo = "a", bar = "b"`のような、カンマで区切られた名前と値のリスト
pub(crate) type CommaPunctuatedNameValues = Punctuated<MetaNameValue, Token![,]>;
