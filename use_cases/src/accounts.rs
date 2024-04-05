use domain::models::user::User;
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
    repository: impl UserRepository,
) -> ProcessUseCaseResult<SingUpOutput> {
    // ユーザーを登録
    match repository.create(input).await {
        Ok(signed_up_user) => Ok(signed_up_user),
        Err(e) => {
            let message = e.to_string();
            if message.contains("ak_users_email") {
                Err(UseCaseError::domain_rule(
                    "同じEメール・アドレスを持つユーザーが、すでに登録されています。",
                ))
            } else if message.contains("fk_users_permission") {
                Err(UseCaseError::validation(
                    "ユーザー権限区分コードが範囲外です。",
                ))
            } else if message.contains("ck_users_either_phone_numbers_must_be_not_null") {
                // インフラストラクチャ層で検証されるため、実際にはここは実行されない
                Err(UseCaseError::domain_rule(
                    "固定電話番号または携帯電話番号を指定する必要があります。",
                ))
            } else {
                Err(UseCaseError::repository(message))
            }
        }
    }
}

/// ユーザーのリストを取得する。
///
/// # 引数
///
/// * `repository` - ユーザー・リポジトリ
///
/// # 戻り値
///
/// * ユーザーを格納したベクタ
#[tracing::instrument(name = "list users use case", skip(repository))]
pub async fn list_users(repository: impl UserRepository) -> ProcessUseCaseResult<Vec<User>> {
    repository
        .list()
        .await
        .map_err(|e| UseCaseError::repository(e.to_string()))
}
