use syn::punctuated::Punctuated;
use syn::{Field, MetaList, MetaNameValue, Token};

/// `foo = "a", bar = "b"`のような、カンマで区切られた名前と値のリスト
pub(crate) type CommaPunctuatedNameValues = Punctuated<MetaNameValue, Token![,]>;

pub(crate) type CommaPunctuatedMetaList = Punctuated<MetaList, Token![,]>;

/// カンマ区切りのフィールドのリスト
///
/// 名前付きフィールド構造体やタプル構造体のフィールドを表現する。
///
/// ```text
/// struct Bar {
///     x: i32,
///     y: String,
/// }
///
/// struct Foo(i32, String);
/// ```
pub(crate) type CommaPunctuatedFields = Punctuated<Field, Token![,]>;
