use secrecy::SecretString;

use domain::models::passwords::generate_phc_string;
use domain::models::passwords::RawPassword;
use domain::models::primitives::*;
use domain::models::user::{User, UserBuilder, UserId};
use domain::now_jst;
use domain::repositories::user::UserRepository;
use macros::{Builder, Getter};

use crate::{ProcessUseCaseResult, UseCaseError};

#[derive(Debug, Clone, Getter, Builder)]
pub struct SignupUser {
    /// Eメールアドレス
    #[getter(ret = "ref")]
    email: EmailAddress,
    /// 未加工なパスワード
    #[getter(ret = "ref")]
    password: RawPassword,
    /// 苗字
    #[getter(ret = "ref")]
    family_name: FamilyName,
    /// 名前
    #[getter(ret = "ref")]
    given_name: GivenName,
    /// 郵便番号
    #[getter(ret = "ref")]
    postal_code: PostalCode,
    /// 住所
    #[getter(ret = "ref")]
    address: Address,
    /// 固定電話番号
    #[getter(ret = "ref")]
    fixed_phone_number: OptionalFixedPhoneNumber,
    /// 携帯電話番号
    #[getter(ret = "ref")]
    mobile_phone_number: OptionalMobilePhoneNumber,
    /// 備考
    #[getter(ret = "ref")]
    remarks: OptionalRemarks,
}

/// ユーザーを登録する。
///
/// # 引数
///
/// * `signup_user` - 登録するユーザー
/// * `pepper` - 未加工なパスワードに付与するペッパー
/// * `repository` - ユーザー・リポジトリ
///
/// # 戻り値
///
/// * 登録したユーザー
#[tracing::instrument(
    name = "signup use case", skip(signup_user, pepper, repository),
    fields(user.email = %signup_user.email())
)]
pub async fn signup(
    signup_user: SignupUser,
    pepper: &SecretString,
    repository: impl UserRepository,
) -> ProcessUseCaseResult<User> {
    // 現在日時を取得
    let dt = now_jst();
    // パスワードをハッシュ化
    let phc_password = generate_phc_string(&signup_user.password, pepper)
        .map_err(|e| UseCaseError::unexpected(e.to_string()))?;
    // ユーザーを構築
    let user = UserBuilder::new()
        .id(UserId::default())
        .email(signup_user.email.to_owned())
        .password(phc_password)
        .active(true)
        .family_name(signup_user.family_name.to_owned())
        .given_name(signup_user.given_name.to_owned())
        .postal_code(signup_user.postal_code.to_owned())
        .address(signup_user.address.to_owned())
        .fixed_phone_number(signup_user.fixed_phone_number.to_owned())
        .mobile_phone_number(signup_user.mobile_phone_number.to_owned())
        .remarks(signup_user.remarks.to_owned())
        .last_logged_in_at(None)
        .created_at(dt)
        .updated_at(dt)
        .build()
        .map_err(|e| UseCaseError::domain_rule(e.to_string()))?;

    // ユーザーを登録
    match repository.create(user).await {
        Ok(created_user) => Ok(created_user),
        Err(e) => {
            let message = e.to_string();
            match message.contains("ak_users_email") {
                true => Err(UseCaseError::domain_rule(
                    "同じEメール・アドレスを持つユーザーが、すでに登録されています。",
                )),
                false => Err(UseCaseError::unexpected(message)),
            }
        }
    }
}
