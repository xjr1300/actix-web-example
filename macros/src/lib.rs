use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod types;
mod utils;

mod primitive;
use primitive::{impl_integer_primitive, impl_primitive_display, impl_string_primitive};
mod optional_string_primitive;
use optional_string_primitive::impl_optional_string_primitive;
mod builder;
use builder::impl_builder;

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
/// `validator`クレートの`Validate`導出マクロと合わせて使用することを前提にしており、
/// `value`フィールドを持つ構造体に、`new`メソッドを実装する。
///
/// ドメインプリミティブ構造体のインスタンスを構築する`new`メソッドは、引数として渡された
/// 文字列の前後の空白文字を除去した文字列を値として格納する。
///
/// `primitive`属性の`name`には、プリミティブの名前を指定する。
/// `primitive`属性の`message`には、プリミティブの検証に失敗したときのメッセージを指定する。
///
/// ```text
/// #[derive(Validator, StringPrimitive)]
/// #[primitive(
///     name = "Eメールアドレス",
///     message = "文字列がEメールアドレスの形式と一致していません。"
/// )]
/// pub struct EmailAddress {
///     #[validate(email)]
///     #[validate(length(
///         min = 1, max = 254,
///     ))]
///     value: String,
/// }
/// ```
#[proc_macro_derive(StringPrimitive, attributes(primitive))]
pub fn derive_string_primitive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_string_primitive(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// `IntegerPrimitive`導出マクロ
///
/// `validator`クレートの`Validate`導出マクロと合わせて使用することを前提にしており、
/// `value`フィールドを持つ構造体に`new`メソッドを実装する。
///
/// `primitive`属性の`name`には、プリミティブの名前を指定する。
///
/// ```text
/// #[derive(Validator, IntegerPrimitive)]
/// #[primitive(name = "数量")]
/// pub struct Amount {
///     #[validate(range(min = 0, max = 20))]
///     value: i32,
/// }
/// ```
#[proc_macro_derive(IntegerPrimitive, attributes(primitive))]
pub fn derive_integer_primitive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_integer_primitive(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// `OptionalStringPrimitive`導出マクロ
///
/// `Option<String>`を持つタプル構造体のメソッドを実装する。
///
/// `primitive`属性の`name`には、プリミティブの名前を指定する。
/// `primitive`属性の`regex`には、格納する文字列がマッチする正規表現を指定する。
/// `primitive`属性の`min`と`max`には、格納する文字列の最小及び最大長さを指定する。
///
/// ```text
/// /// 携帯電話番号
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
/// #[primitive(
///     name = "携帯電話番号",
///     regex = r"^0[789]0-[0-9]{4}-[0-9]{4}$",
/// )]
/// pub struct MobilePhoneNumber(Option<String>);
///
/// /// 備考
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
/// #[primitive(
///     name = "備考"
///     min = 10, max = 400,
/// )]
/// pub struct Remarks(Option<String>);
/// ```
#[proc_macro_derive(OptionalStringPrimitive, attributes(primitive))]
pub fn derive_optional_string_primitive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_optional_string_primitive(input) {
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
