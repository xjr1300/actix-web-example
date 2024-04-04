use secrecy::SecretString;

use domain::repositories::user::{SignUpInput, SingUpOutput, UserRepository};

use crate::{ProcessUseCaseResult, UseCaseError};

/// ユーザーを登録する。
///
/// # 引数
///
/// * `user` - 登録するユーザー
/// * `pepper` - 未加工なパスワードに付与するペッパー
/// * `repository` - ユーザー・リポジトリ
///
/// # 戻り値
///
/// * 登録したユーザー
#[tracing::instrument(
    name = "sign up use case", skip(input, repository),
    fields(user.email = %input.email)
)]
pub async fn sign_up(
    input: SignUpInput,
    pepper: &SecretString,
    repository: impl UserRepository,
) -> ProcessUseCaseResult<SingUpOutput> {
    // ユーザーを登録
    match repository.create(input).await {
        Ok(signed_up_user) => Ok(signed_up_user),
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
