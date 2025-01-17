use secrecy::SecretString;
use time::{Duration, OffsetDateTime};

use domain::models::primitives::*;
use domain::models::user::{User, UserId, UserPermissionCode};
use domain::repositories::token::{TokenPairWithTtl, TokenRepository};
use domain::repositories::user::{SignUpInputBuilder, SignUpOutput, UserRepository};
use macros::Builder;

use crate::jwt::generate_token_pair;
use crate::passwords::{generate_phc_string, verify_password};
use crate::settings::{AuthorizationSettings, PasswordSettings};
use crate::{
    UseCaseError, UseCaseErrorKind, UseCaseResult, ERR_SAME_EMAIL_ADDRESS_IS_REGISTERED,
    ERR_SPECIFY_FIXED_OR_MOBILE_NUMBER,
};

/// サインアップユースケース入力
#[derive(Debug, Clone, Builder)]
pub struct SignUpUseCaseInput {
    /// Eメールアドレス
    pub email: EmailAddress,
    /// パスワード
    pub password: RawPassword,
    /// アクティブフラグ
    pub active: bool,
    /// ユーザー権限コード
    pub user_permission_code: UserPermissionCode,
    /// 苗字
    pub family_name: FamilyName,
    /// 名前
    pub given_name: GivenName,
    /// 郵便番号
    pub postal_code: PostalCode,
    /// 住所
    pub address: Address,
    /// 固定電話番号
    pub fixed_phone_number: OptionalFixedPhoneNumber,
    /// 携帯電話番号
    pub mobile_phone_number: OptionalMobilePhoneNumber,
    /// 備考
    pub remarks: OptionalRemarks,
}

/// サインアップユースケース出力
pub struct SignUpUseCaseOutput {
    /// ユーザーID
    pub id: UserId,
    /// Eメールアドレス
    pub email: EmailAddress,
    /// アクティブフラグ
    pub active: bool,
    /// ユーザー権限コード
    pub user_permission_code: UserPermissionCode,
    /// 苗字
    pub family_name: FamilyName,
    /// 名前
    pub given_name: GivenName,
    /// 郵便番号
    pub postal_code: PostalCode,
    /// 住所
    pub address: Address,
    /// 固定電話番号
    pub fixed_phone_number: OptionalFixedPhoneNumber,
    /// 携帯電話番号
    pub mobile_phone_number: OptionalMobilePhoneNumber,
    /// 備考
    pub remarks: OptionalRemarks,
    /// 作成日時
    pub created_at: OffsetDateTime,
    /// 更新日時
    pub updated_at: OffsetDateTime,
}

impl From<SignUpOutput> for SignUpUseCaseOutput {
    fn from(value: SignUpOutput) -> Self {
        Self {
            id: value.id,
            email: value.email,
            active: value.active,
            user_permission_code: value.user_permission_code,
            family_name: value.family_name,
            given_name: value.given_name,
            postal_code: value.postal_code,
            address: value.address,
            fixed_phone_number: value.fixed_phone_number,
            mobile_phone_number: value.mobile_phone_number,
            remarks: value.remarks,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// ユーザーを登録する。
///
/// # 引数
///
/// * `password_settings` - パスワード設定
/// * `user_repository` - ユーザーリポジトリ
/// * `input` - サインアップユースケース入力
///
/// # 戻り値
///
/// * 登録したユーザー
#[tracing::instrument(
    name = "sign up use case", skip(password_settings, user_repository, input),
    fields(user.email = %input.email)
)]
pub async fn sign_up(
    password_settings: &PasswordSettings,
    user_repository: impl UserRepository,
    input: SignUpUseCaseInput,
) -> UseCaseResult<SignUpUseCaseOutput> {
    let id = UserId::default();
    let password =
        generate_phc_string(&input.password, password_settings).map_err(UseCaseError::from)?;

    let input = SignUpInputBuilder::new()
        .id(id)
        .email(input.email)
        .password(password)
        .active(input.active)
        .user_permission_code(input.user_permission_code)
        .family_name(input.family_name)
        .given_name(input.given_name)
        .postal_code(input.postal_code)
        .address(input.address)
        .fixed_phone_number(input.fixed_phone_number)
        .mobile_phone_number(input.mobile_phone_number)
        .remarks(input.remarks)
        .build()
        .map_err(|e| UseCaseError::domain_rule(e.to_string()))?;

    // ユーザーを登録
    match user_repository.create(input).await {
        Ok(inserted_user) => Ok(inserted_user.into()),
        Err(e) => {
            let message = e.to_string();
            if message.contains("ak_users_email") {
                Err(UseCaseError::new(
                    UseCaseErrorKind::DomainRule,
                    ERR_SAME_EMAIL_ADDRESS_IS_REGISTERED,
                    "同じEメールアドレスを持つユーザーが、すでに登録されています。",
                ))
            } else if message.contains("fk_users_permission") {
                Err(UseCaseError::validation(
                    "ユーザー権限区分コードが範囲外です。",
                ))
            } else if message.contains("ck_users_either_phone_numbers_must_be_not_null") {
                // インフラストラクチャ層で検証されるため、実際にはここは実行されない
                Err(UseCaseError::new(
                    UseCaseErrorKind::DomainRule,
                    ERR_SPECIFY_FIXED_OR_MOBILE_NUMBER,
                    "固定電話番号または携帯電話番号を指定する必要があります。",
                ))
            } else {
                Err(UseCaseError::repository(message))
            }
        }
    }
}

/// ユーザーがサインインする。
///
/// ユーザーが最初にサインインに失敗した日時: last_failed_at
/// 現在の日時: now_dt
/// ユーザーのアカウントをロックするサインイン失敗回数: number_of_failures
/// 上記サインイン失敗回数をカウントする期間（秒）: attempting_seconds
///
/// 最初にサインインに失敗した日時か記録されていない場合、または最初にサインインに失敗した日時に失敗回数を
/// カウントする期間を足した日時が、現在の日時よりも過去の場合は、最初のサインインの失敗として記録
///
/// * last_failed_at.is_none()
/// * last_failed_at + attempting_seconds < now_dt
///
/// 最初にサインインに失敗した日時に失敗回数をカウントする期間を足した日時が、現在の日時より未来の場合は、
/// サインイン失敗回数をインクリメント
///
/// * last_failed_at + attempting_seconds >= now_dt
///
/// 上記の結果、サインイン失敗回数がユーザーのアカウントをロックするサインイン失敗回数に達した場合は、
/// ユーザーのアカウントをロック
///
/// * サインイン失敗回数 >= number_of_failures
///
/// # 引数
///
/// * `password_settings` - パスワード設定
/// * `authorization_settings` - 認証設定
/// * `user_repository` - ユーザーリポジトリ
/// * `token_repository` - トークンリポジトリ
/// * `input` - サインインユースケース入力
///
/// # 戻り値
///
/// * アクセストークンとリフレッシュトークン
pub async fn sign_in(
    password_settings: &PasswordSettings,
    authorization_settings: &AuthorizationSettings,
    user_repo: impl UserRepository,
    token_repo: impl TokenRepository,
    input: SignInUseCaseInput,
) -> UseCaseResult<SignInUseCaseOutput> {
    // 現在の日時
    let now_dt = OffsetDateTime::now_utc();
    // 不許可／未認証エラー
    let unauthorized_error =
        UseCaseError::unauthorized("Eメールアドレスまたはパスワードが間違っています。");
    // サイン履歴保存エラー
    let history_record_error =
        UseCaseError::repository("ユーザーのサインイン履歴の保存に失敗しました。");

    // ユーザーのクレデンシャルを取得
    let credential = user_repo
        .user_credential(input.email)
        .await
        .map_err(UseCaseError::from)?;
    if credential.is_none() {
        return Err(unauthorized_error);
    }
    let credential = credential.unwrap();
    // アカウントがアクティブか確認
    if !credential.active {
        return Err(UseCaseError::unauthorized(
            "ユーザーのアカウントがロックされています。",
        ));
    }
    // パスワードを検証
    if !verify_password(
        &input.password,
        &password_settings.pepper,
        &credential.password,
    )? {
        // ユーザーの最初にサインインに失敗した日時が記録されていない
        // または最初にサインインに失敗した日時に失敗回数をカウントする期間を足した日時が、現在の日時よりも過去
        let latest_credential = if credential.attempted_at.is_none()
            || credential.attempted_at.unwrap()
                + Duration::seconds(authorization_settings.attempting_seconds.into())
                < now_dt
        {
            // 最初のサインインの失敗として記録
            user_repo
                .record_first_sign_in_failed(credential.user_id)
                .await
                .map_err(|_| history_record_error.clone())?
        } else {
            // サインイン失敗回数をインクリメント
            user_repo
                .increment_number_of_sign_in_failures(credential.user_id)
                .await
                .map_err(|_| history_record_error.clone())?
        };
        // サインイン失敗回数がユーザーのアカウントをロックする失敗回数に達した場合、
        // ユーザーのアカウントをロック
        let latest_credential = latest_credential.unwrap();
        if authorization_settings.number_of_failures <= latest_credential.number_of_failures as u16
        {
            user_repo
                .lock_user_account(latest_credential.user_id)
                .await
                .map_err(|_| history_record_error)?;
        }

        return Err(unauthorized_error);
    }

    // 最後にサインインした日時を更新
    let credential = user_repo
        .update_last_sign_in(credential.user_id)
        .await
        .map_err(UseCaseError::from)?;
    let credential = credential.unwrap();

    // アクセストークン及びリフレッシュトークンを生成
    let dt = OffsetDateTime::now_utc();
    let access_expiration =
        dt + Duration::seconds(authorization_settings.access_token_seconds as i64);
    let refresh_expiration =
        dt + Duration::seconds(authorization_settings.refresh_token_seconds as i64);
    let tokens = generate_token_pair(
        credential.user_id,
        access_expiration,
        refresh_expiration,
        &authorization_settings.jwt_token_secret,
    )?;

    // アクセストークン及びリフレッシュトークンをリポジトリに保存
    let token_with_ttls = TokenPairWithTtl {
        access: &tokens.access,
        access_ttl: authorization_settings.access_token_seconds,
        refresh: &tokens.refresh,
        refresh_ttl: authorization_settings.refresh_token_seconds,
    };
    token_repo
        .register_token_pair(
            credential.user_id,
            token_with_ttls,
            credential.user_permission_code,
        )
        .await?;

    Ok(SignInUseCaseOutput {
        access: tokens.access,
        access_expiration,
        refresh: tokens.refresh,
        refresh_expiration,
    })
}

/// サインインユースケース入力
pub struct SignInUseCaseInput {
    /// Eメールアドレス
    pub email: EmailAddress,
    /// 加工していないパスワード
    pub password: RawPassword,
}

/// サインインユースケース出力
pub struct SignInUseCaseOutput {
    /// アクセストークン
    pub access: SecretString,
    /// アクセストークンの有効期限
    pub access_expiration: OffsetDateTime,
    /// リフレッシュトークン
    pub refresh: SecretString,
    /// リフレッシュトークンの有効期限
    pub refresh_expiration: OffsetDateTime,
}

/// JWTトークンの正規表現
pub const JWT_TOKEN_EXPRESSION: &str =
    r#"^([a-zA-Z0-9_=]+)\.([a-zA-Z0-9_=]+)\.([a-zA-Z0-9_\-\+\/=]*)$"#;

/// ユーザーのリストを取得する。
///
/// # 引数
///
/// * `repository` - ユーザーリポジトリ
///
/// # 戻り値
///
/// * ユーザーを格納したベクタ
#[tracing::instrument(name = "list users use case", skip(repository))]
pub async fn list_users(repository: impl UserRepository) -> UseCaseResult<Vec<User>> {
    repository
        .list()
        .await
        .map_err(|e| UseCaseError::repository(e.to_string()))
}
