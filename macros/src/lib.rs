use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod types;
mod utils;

mod primitive;
use primitive::{impl_domain_primitive, impl_primitive_display, impl_string_primitive};
mod optional_string_tuple_primitive;
use optional_string_tuple_primitive::impl_tuple_optional_string_primitive;
mod getter;
use getter::impl_getter;
mod builder;
use builder::impl_builder;

/// `DomainPrimitive`導出マクロ
///
/// `value`フィールドを持つ構造体に`value`メソッドを実装する。
///
/// 1. `value`フィールドが`Copy`トレイトを実装している型の場合、`#[value_getter(ret = "val")]`
/// 2. `value`フィールドが`Copy`トレイトを実装していない型で、その参照を`value`メソッドが返す場合、`#[value_getter(ret = "ref")]`
/// 3. `value`フィールドが`Copy`トレイトを実装していない型で、その型と異なる参照を`value`メソッドが返す場合、`#[value_getter(ret = "ref", rty = "&str")]`
///
/// 上記3は、`value`フィールドの型が`String`の場合を示す。
#[proc_macro_derive(DomainPrimitive, attributes(value_getter))]
pub fn derive_domain_primitive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_domain_primitive(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// `PrimitiveDisplay`導出マクロ
///
/// `value`フィールドを持つ構造体に`std::fmt::Display`を実装する。
#[proc_macro_derive(PrimitiveDisplay)]
pub fn derive_primitive_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_primitive_display(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// `StringPrimitive`導出マクロ
///
/// `value`フィールドを持つ構造体に、`new`メソッドを実装する。
///
/// ドメイン・プリミティブ構造体のインスタンスを構築する`new`メソッドは、引数として渡された
/// 文字列の前後の空白文字を除去（トリム）した文字列を値として格納する。
#[proc_macro_derive(StringPrimitive)]
pub fn derive_string_primitive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_string_primitive(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// `TupleOptionalStringPrimitive`導出マクロ
///
/// `Option<String>`を持つタプル構造体のメソッドを実装する。
///
/// ```text
/// /// 携帯電話番号
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, TupleOptionalStringPrimitive)]
/// #[primitive_validation(regex = r"^0[789]0-[0-9]{4}-[0-9]{4}$")]
/// pub struct MobilePhoneNumber(Option<String>);
///
/// /// 備考
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, TupleOptionalStringPrimitive)]
/// #[primitive_validation(min = 10, max = 400)]
/// pub struct Remarks(Option<String>);
/// ```
#[proc_macro_derive(TupleOptionalStringPrimitive, attributes(primitive_validation))]
pub fn derive_tuple_optional_string_primitive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_tuple_optional_string_primitive(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// ゲッター導出マクロ
///
/// ```text
/// #[derive(Getter)]
/// pub struct Foo {
///     #[getter(ret="val")]
///     a: i32,
///     #[getter(ret="ref")]
///     b: PathBuf,
///     #[getter(ret="ref", rty="&str")]
///     c: String,
/// }
/// ```
///
/// 上記構造体から次を導出する。
///
/// ```text
/// impl Foo {
///     pub fn a(&self) -> i32 {
///         self.a
///     }
///     pub fn b(&self) -> &PathBuf {
///          &self.b
///     }
///     pub fn c(&self) -> &str {
///        &self.c
///     }
/// }
/// ```
#[proc_macro_derive(Getter, attributes(getter))]
pub fn derive_getter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_getter(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// `Builder`導出マクロ
///
/// 構造体のビルダーを実装する。
///
/// ```text
/// #[derive(Builder)]
/// pub struct Command {
///     executable: String,
///     #[builder(each = "arg")]
///     args: Vec<String>,
///     current_dir: Option<String>,
///     value: Option<u8>,
/// }
///
/// let command = CommandBuilder::new()
///     .executable("cargo".to_owned())
///     .arg("build".to_owned())
///     .arg("--release".to_owned())
///     .current_dir(Some(String::from("/home")))
///     .value(None)
///     .build()
///     .unwrap();
/// assert_eq!(command.executable, "cargo");
/// ```
#[proc_macro_derive(Builder, attributes(builder_validation, builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    match impl_builder(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}
