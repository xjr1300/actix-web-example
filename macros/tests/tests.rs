use validator::Validate;

use domain::common::{DomainError, DomainResult};
use macros::{DomainPrimitive, OptionalStringPrimitive, PrimitiveDisplay, StringPrimitive};

/// `value`メソッドが値を返すドメイン・プリミティブを実装できることを確認
#[test]
fn value_method_returns_value_domain_primitive_works() {
    #[derive(DomainPrimitive)]
    struct TestStruct {
        #[value_getter(ret = "val")]
        value: i32,
    }

    let s = TestStruct { value: 42 };

    assert_eq!(42, s.value());
}

/// `value`メソッドが参照を返すドメイン・プリミティブを実装できることを確認
#[test]
fn value_method_returns_reference_domain_primitive_works() {
    #[derive(DomainPrimitive)]
    struct TestStruct {
        #[value_getter(ret = "ref")]
        value: String,
    }

    let s = TestStruct {
        value: String::from("spam"),
    };

    assert_eq!(&String::from("spam"), s.value());
}

/// `value`メソッドが別の参照を返すドメイン・プリミティブを実装できることを確認
#[test]
fn value_method_returns_another_reference_domain_primitive_works() {
    #[derive(DomainPrimitive)]
    struct TestStruct {
        #[value_getter(ret = "ref", rty = "&str")]
        value: String,
    }

    let s = TestStruct {
        value: "spam".to_string(),
    };

    assert_eq!("spam", s.value());
}

/// `Display`トレイトを実装したドメイン・プリミティブを実装できることを確認
#[test]
fn primitive_display_works() {
    #[derive(DomainPrimitive, PrimitiveDisplay)]
    struct TestStruct {
        #[value_getter(ret = "val")]
        value: i32,
    }

    let s = TestStruct { value: 42 };

    assert_eq!("42", format!("{}", s));
}

#[derive(Validate, DomainPrimitive, StringPrimitive)]
#[primitive(
    name = "プリミティブ名",
    message = "10文字以上20文字以下の文字列を指定してください。"
)]
struct TestStringPrimitive {
    #[value_getter(ret = "ref", rty = "&str")]
    #[validate(length(min = 10, max = 20,))]
    value: String,
}

/// 適切な文字数で文字列プリミティブを構築できることを確認
#[test]
fn string_primitive_can_be_constructed_from_valid_length_characters() {
    let s = TestStringPrimitive {
        value: String::from("spam"),
    };

    assert_eq!(&String::from("spam"), s.value());
}

/// 前後の空白文字をトリムして文字列プリミティブを構築できることを確認
#[test]
fn constructed_string_primitive_was_removed_blank_characters_from_the_beginning_and_end() {
    let candidates = [
        "foo bar baz qux quux ",
        " foo bar baz qux quux",
        " foo bar baz qux quux ",
    ];
    for candidate in candidates {
        let s = TestStringPrimitive::new(candidate).unwrap();
        assert_eq!(&String::from("foo bar baz qux quux"), s.value());
    }
}

/// 指定された文字数より少ない文字数で文字列プリミティブを構築できないことを確認
#[test]
fn string_primitive_can_not_be_constructed_with_a_string_less_than_specified_length() {
    let s = TestStringPrimitive::new(String::from("spam"));

    assert!(s.is_err());
    assert_eq!(
        "10文字以上20文字以下の文字列を指定してください。",
        s.err().unwrap().to_string()
    );
}

/// 指定された文字数より多い文字数で文字列プリミティブを構築できないことを確認
#[test]
fn string_primitive_can_not_be_constructed_with_a_string_more_than_specified_length() {
    let s = TestStringPrimitive::new("s".repeat(21));

    assert!(s.is_err());
    assert_eq!(
        "10文字以上20文字以下の文字列を指定してください。",
        s.err().unwrap().to_string()
    );
}

/// 空文字で文字列プリミティブを構築できないことを確認
#[test]
fn string_primitive_can_not_be_constructed_with_empty_or_blank_strings() {
    let candidates = ["", "     "];

    for candidate in candidates {
        let s = TestStringPrimitive::new(candidate);
        assert!(s.is_err(), "`{}`", candidate);
        assert_eq!(
            "プリミティブ名は空文字を指定できません。",
            s.err().unwrap().to_string()
        );
    }
}

/// 携帯電話番号
#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(name = "携帯電話番号", regex = r"^0[789]0-[0-9]{4}-[0-9]{4}$")]
pub struct OptionalMobileNumber(Option<String>);

/// 携帯電話番号の形式として妥当な文字列から携帯電話番号を構築できることを確認
#[test]
fn mobile_phone_number_can_be_constructed_from_a_valid_string() {
    let candidate = "090-1234-5678";
    let phone_number = OptionalMobileNumber::try_from(candidate).unwrap();

    assert!(phone_number.is_some());
    assert_eq!(Some("090-1234-5678"), phone_number.value())
}

/// 携帯電話番号の形式として妥当でない文字列から携帯電話番号を構築できることを確認
#[test]
fn mobile_phone_number_can_not_be_constructed_from_an_invalid_string() {
    let candidate = "000-1234-5678";
    let phone_number = OptionalMobileNumber::try_from(candidate);

    assert!(phone_number.is_err());
    assert_eq!(
        "携帯電話番号に指定した文字列の形式が誤っています。",
        phone_number.err().unwrap().to_string()
    );
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(name = "オプショナル文字列", max = 10)]
pub struct MaxLengthOptionalString(Option<String>);

/// 最大文字数以下の文字列を持つオプショナル文字列プリミティブを構築できることを確認
#[test]
fn optional_string_can_be_constructed_from_max_length_or_less_of_characters() {
    let candidates = [
        ("a", String::from("a")),
        ("aaaaaaaaaa", "a".repeat(10)),
        ("aaaaaaaaaa", String::from(" aaaaaaaaaa")),
        ("aaaaaaaaaa", String::from("aaaaaaaaaa ")),
        ("aaaaaaaaaa", String::from(" aaaaaaaaaa ")),
    ];
    for (expected, candidate) in candidates {
        let s = MaxLengthOptionalString::try_from(candidate.clone()).unwrap();

        assert_eq!(expected, s.value().unwrap());
    }
}

/// 最大文字数以上の文字列を持つオプショナル文字列プリミティブを構築できないことを確認
#[test]
fn optional_string_can_not_be_constructed_from_over_length_characters() {
    let candidates = ["a".repeat(11), "a".repeat(12)];
    for candidate in candidates {
        let s = MaxLengthOptionalString::try_from(candidate.clone());
        assert!(s.is_err(), "{}", candidate);
        assert_eq!(
            "オプショナル文字列は10文字以下の文字列を指定してください。",
            s.err().unwrap().to_string(),
            "{}",
            candidate
        );
    }
}

/// 空白文字列でオプショナル文字列プリミティブを構築したとき、`None`になることを確認
#[test]
fn optional_string_was_constructed_from_empty_or_blank_string_is_none() {
    let candidates = [String::new(), " ".repeat(10), " ".repeat(11)];
    for candidate in candidates {
        let s = MaxLengthOptionalString::try_from(candidate.clone()).unwrap();
        assert_eq!(None, s.value(), "{}", candidate);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(name = "オプショナル文字列", min = 10)]
pub struct MinLengthOptionalString(Option<String>);

/// オプショナル文字列プリミティブが指定された最小文字数以上の文字列で構築できることを確認
#[test]
fn optional_string_can_be_constructed_from_min_length_or_more_characters() {
    let candidates = ["a".repeat(10), "a".repeat(11)];
    for candidate in candidates {
        let s = MinLengthOptionalString::try_from(candidate.clone()).unwrap();
        assert_eq!(candidate, s.value().unwrap());
    }
}

/// オプショナル文字列プリミティブが指定された最小文字数未満の文字列で構築できないことを確認
#[test]
fn optional_string_can_not_be_constructed_from_few_specified_length_characters() {
    let candidates = [
        "a".repeat(9),
        "a".repeat(8),
        String::from("aaaaaaaaa "),
        String::from(" aaaaaaaaa"),
        String::from(" aaaaaaaaa "),
    ];
    for candidate in candidates {
        let s = MinLengthOptionalString::try_from(candidate.clone());
        assert!(s.is_err(), "{}", candidate);
        assert_eq!(
            "オプショナル文字列は10文字以上の文字列を指定してください。",
            s.err().unwrap().to_string()
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(name = "オプショナル文字列", min = 10, max = 20)]
pub struct MinMaxLengthOptionalString(Option<String>);

/// オプショナル文字列プリミティブが指定された範囲内の文字列から構築できることを確認
#[test]
fn optional_string_can_be_constructed_from_between_min_and_max_length_characters() {
    let candidates = [
        "a".repeat(10),
        "a".repeat(11),
        "a".repeat(19),
        "a".repeat(20),
    ];
    for candidate in candidates {
        let s = MinMaxLengthOptionalString::try_from(candidate.clone()).unwrap();
        assert_eq!(candidate, s.value().unwrap());
    }
}

/// オプショナル文字列プリミティブが指定された範囲外の文字列から構築できることを確認
#[test]
fn optional_string_can_not_be_constructed_from_out_of_range_length_characters() {
    let candidates = ["a".repeat(8), "a".repeat(9)];
    for candidate in candidates {
        let s = MinMaxLengthOptionalString::try_from(candidate.clone());
        assert!(s.is_err(), "{}", candidate);
        assert_eq!(
            "オプショナル文字列は10文字以上の文字列を指定してください。",
            s.err().unwrap().to_string()
        );
    }

    let candidates = ["a".repeat(21), "a".repeat(22)];
    for candidate in candidates {
        let s = MinMaxLengthOptionalString::try_from(candidate.clone());
        assert!(s.is_err(), "{}: {}", candidate, s.is_err());
        assert_eq!(
            "オプショナル文字列は20文字以下の文字列を指定してください。",
            s.err().unwrap().to_string()
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, OptionalStringPrimitive)]
#[primitive(name = "オプショナル文字列", regex = r#"^[0-9]{10}$"#)]
pub struct RegexOptionalString(Option<String>);

/// オプショナル文字列プリミティブが正規表現とマッチする文字列から構築できることを確認
#[test]
fn optional_string_can_be_constructed_from_matching_string() {
    let candidates = ["0123456789", " 0123456789", "0123456789 ", " 0123456789 "];
    for candidate in candidates {
        let s = RegexOptionalString::try_from(candidate).unwrap();
        assert_eq!(candidate.trim(), s.value().unwrap());
    }
}

/// オプショナル文字列プリミティブが正規表現とマッチする文字列から構築できることを確認
#[test]
fn optional_string_can_not_be_constructed_from_non_matching_string() {
    let candidates = ["a", "a123456789"];
    for candidate in candidates {
        let s = RegexOptionalString::try_from(candidate);
        assert!(s.is_err(), "{}", candidate);
        assert_eq!(
            "オプショナル文字列に指定した文字列の形式が誤っています。",
            s.err().unwrap().to_string()
        );
    }
}
