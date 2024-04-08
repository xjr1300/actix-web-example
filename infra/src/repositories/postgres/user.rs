use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use sqlx::Postgres;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::models::primitives::*;
use domain::models::user::{User, UserId, UserPermission, UserPermissionCode, UserPermissionName};
use domain::repositories::user::{SignUpInput, SignUpOutput, UserCredential, UserRepository};
use domain::{DomainError, DomainResult};

use crate::repositories::postgres::{commit_transaction, PgRepository};

/// PostgreSQLユーザーリポジトリ
pub type PgUserRepository = PgRepository<User>;

type PgQueryAs<'q, T> = sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>;
type PgQuery<'q> = sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>;

#[async_trait]
impl UserRepository for PgUserRepository {
    /// ユーザーのリストを取得する。
    ///
    /// # 戻り値
    ///
    /// ユーザーを格納したベクタ
    async fn list(&self) -> DomainResult<Vec<User>> {
        Ok(list_users_query()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })?
            .into_iter()
            .map(|r| r.into())
            .collect::<_>())
    }

    /// ユーザーを取得する。
    ///
    /// # 戻り値
    ///
    /// ユーザー
    async fn by_id(&self, user_id: UserId) -> DomainResult<Option<User>> {
        Ok(user_by_id_query(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })?
            .map(|r| r.into()))
    }

    /// ユーザーのクレデンシャルを取得する。
    ///
    /// # 引数
    ///
    /// * `email` - ユーザーのEメールアドレス
    ///
    /// # 戻り値
    ///
    /// ユーザーのクレデンシャル
    async fn user_credential(&self, email: EmailAddress) -> DomainResult<Option<UserCredential>> {
        user_credential_query(email)
            .fetch_optional(&self.pool)
            .await
            .map(|r| r.map(|r| r.into()))
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })
    }

    /// ユーザーが最後にサインインした日時を更新する。
    ///
    /// サインインした日時を現在の日時、最初にサインインに失敗した日時をNULL、そしてサインイン失敗回数を0にする。
    ///
    /// # 引数
    ///
    /// * `user_id` - ユーザーID
    async fn update_last_sign_in(&self, user_id: UserId) -> DomainResult<Option<OffsetDateTime>> {
        let mut tx = self.begin().await?;
        let row = update_last_sign_in_at_query(user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })?;
        commit_transaction(tx).await?;

        Ok(row.map(|r| r.last_sign_in_at))
    }

    /// 最初にサインインに失敗した日時を保存する。
    ///
    /// サインインに失敗した回数は1になる。
    ///
    /// # 引数
    ///
    /// * `user_id` - ユーザーID
    async fn record_first_sign_in_failed(
        &self,
        user_id: UserId,
    ) -> DomainResult<Option<UserCredential>> {
        let mut tx = self.begin().await?;
        let row = record_first_sign_in_failed_query(user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })?;
        commit_transaction(tx).await?;

        Ok(row.map(|r| r.into()))
    }

    /// サインイン失敗回数をインクリメントする。
    ///
    /// # 引数
    ///
    /// * `user_id` - ユーザーID
    ///
    /// # 戻り値
    ///
    /// インクリメント後のサインイン失敗回数
    async fn increment_number_of_sign_in_failures(
        &self,
        user_id: UserId,
    ) -> DomainResult<Option<UserCredential>> {
        let mut tx = self.begin().await?;
        let row = increment_number_of_sign_in_failures_query(user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })?;
        commit_transaction(tx).await?;

        Ok(row.map(|r| r.into()))
    }

    /// ユーザーのアカウントをロックする。
    ///
    /// # 引数
    ///
    /// * `user_id` - ユーザーID
    async fn lock_user_account(&self, user_id: UserId) -> DomainResult<()> {
        let mut tx = self.begin().await?;
        let _ = set_active_query(user_id, false)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })?;
        commit_transaction(tx).await?;

        Ok(())
    }

    /// ユーザーのアカウントをアンロックする。
    ///
    /// # 引数
    ///
    /// * `user_id` - ユーザーID
    async fn unlock_user_account(&self, user_id: UserId) -> DomainResult<()> {
        let mut tx = self.begin().await?;
        let _ = set_active_query(user_id, true)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })?;
        commit_transaction(tx).await?;

        Ok(())
    }

    /// 最初にサインインに失敗した日時をNULL、サインイン失敗回数を0にする。
    ///
    /// # 引数
    ///
    /// * `user_id` - ユーザーID
    async fn clear_sign_in_failed_history(
        &self,
        user_id: UserId,
    ) -> DomainResult<Option<UserCredential>> {
        let mut tx = self.begin().await?;
        let row = clear_sign_in_failed_history_query(user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })?;
        commit_transaction(tx).await?;

        Ok(row.map(|r| r.into()))
    }

    /// ユーザーを登録する。
    ///
    /// ユーザーを登録するとき、ユーザーの作成日時と更新日時は何らかの日時を設定する。
    /// 登録後に返されるユーザーの作成日時と更新日時の作成日時と更新日時には、データベースに登録
    /// した日時が設定されている。
    async fn create(&self, user: SignUpInput) -> DomainResult<SignUpOutput> {
        let mut tx = self.begin().await?;
        let inserted_user = insert_user_query(user)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("{} ({}:{})", e, file!(), line!());
                DomainError::Repository(e.into())
            })?;
        commit_transaction(tx).await?;

        Ok(inserted_user.into())
    }
}

#[derive(sqlx::FromRow)]
pub struct RetrievedUserRow {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub active: bool,
    pub user_permission_code: i16,
    pub user_permission_name: String,
    pub family_name: String,
    pub given_name: String,
    pub postal_code: String,
    pub address: String,
    pub fixed_phone_number: Option<String>,
    pub mobile_phone_number: Option<String>,
    pub remarks: Option<String>,
    pub last_sign_in_at: Option<OffsetDateTime>,
    pub sign_in_attempted_at: Option<OffsetDateTime>,
    pub number_of_sign_in_failures: i16,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<RetrievedUserRow> for User {
    fn from(row: RetrievedUserRow) -> Self {
        Self {
            id: UserId::new(row.id),
            email: EmailAddress::new(row.email).unwrap(),
            active: row.active,
            user_permission: UserPermission::new(
                UserPermissionCode::try_from(row.user_permission_code).unwrap(),
                UserPermissionName::new(row.user_permission_name).unwrap(),
            ),
            family_name: FamilyName::new(row.family_name).unwrap(),
            given_name: GivenName::new(row.given_name).unwrap(),
            postal_code: PostalCode::new(row.postal_code).unwrap(),
            address: Address::new(row.address).unwrap(),
            fixed_phone_number: OptionalFixedPhoneNumber::try_from(row.fixed_phone_number).unwrap(),
            mobile_phone_number: OptionalMobilePhoneNumber::try_from(row.mobile_phone_number)
                .unwrap(),
            remarks: OptionalRemarks::try_from(row.remarks).unwrap(),
            last_sign_in_at: row.last_sign_in_at,
            sign_in_attempted_at: row.sign_in_attempted_at,
            number_of_sign_in_failures: NumberOfSignInFailures::new(row.number_of_sign_in_failures)
                .unwrap(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// ユーザーのリストを取得するクエリを生成する。
///
/// # 戻り値
///
/// ユーザーの一覧を取得するクエリ
pub fn list_users_query<'q>() -> PgQueryAs<'q, RetrievedUserRow> {
    sqlx::query_as::<Postgres, RetrievedUserRow>(
        r#"
        SELECT
            u.id, u.email, u.password, u.active, u.user_permission_code, p.name
            user_permission_name, u.family_name, u.given_name, u.postal_code, u.address,
            u.fixed_phone_number, u.mobile_phone_number, u.remarks, u.last_sign_in_at,
            u.sign_in_attempted_at, u.number_of_sign_in_failures, u.created_at,
            u.updated_at
        FROM users u
        INNER JOIN user_permissions p ON u.user_permission_code = p.code
        ORDER BY created_at
    "#,
    )
}

/// ユーザーIDを元にユーザーを取得するクエリを生成する。
///
/// # 引数
///
/// * `user_id` - ユーザーID
///
/// # 戻り値
///
/// ユーザーIDを元にユーザーを取得するクエリ
pub fn user_by_id_query<'q>(user_id: UserId) -> PgQueryAs<'q, RetrievedUserRow> {
    sqlx::query_as::<Postgres, RetrievedUserRow>(
        r#"
        SELECT
            u.id, u.email, u.password, u.active, u.user_permission_code, p.name
            user_permission_name, u.family_name, u.given_name, u.postal_code, u.address,
            u.fixed_phone_number, u.mobile_phone_number, u.remarks, u.last_sign_in_at,
            u.sign_in_attempted_at, u.number_of_sign_in_failures, u.created_at,
            u.updated_at
        FROM users u
        INNER JOIN user_permissions p ON u.user_permission_code = p.code
        WHERE u.id = $1
        "#,
    )
    .bind(user_id.value)
}

#[derive(sqlx::FromRow)]
pub struct UserCredentialRow {
    #[sqlx(rename = "id")]
    pub user_id: Uuid,
    pub email: String,
    pub password: String,
    pub active: bool,
    #[sqlx(rename = "sign_in_attempted_at")]
    pub attempted_at: Option<OffsetDateTime>,
    #[sqlx(rename = "number_of_sign_in_failures")]
    pub number_of_failures: i16,
}

impl From<UserCredentialRow> for UserCredential {
    fn from(row: UserCredentialRow) -> Self {
        Self {
            user_id: UserId::new(row.user_id),
            email: EmailAddress::new(row.email).unwrap(),
            password: PhcPassword::new(SecretString::new(row.password)).unwrap(),
            active: row.active,
            attempted_at: row.attempted_at,
            number_of_failures: row.number_of_failures,
        }
    }
}

/// ユーザークレデンシャルを取得するクエリを生成する。
///
/// # 引数
///
/// * `email` - ユーザークレデンシャルを取得するユーザーのEメール・アドレス
///
/// # 戻り値
///
/// ユーザークレデンシャルを取得するクエリ
pub fn user_credential_query<'q>(email: EmailAddress) -> PgQueryAs<'q, UserCredentialRow> {
    sqlx::query_as::<Postgres, UserCredentialRow>(
        r#"
        SELECT
            id, email, password, active, sign_in_attempted_at, number_of_sign_in_failures
        FROM
            users
        WHERE
            email = $1
        "#,
    )
    .bind(email.value)
}

/// サインインした日時を現在の日時、最初にサインインに失敗した日時をNULL、そしてサインイン失敗回数を0にするクエリを生成する。
///
/// # 引数
///
/// * `user_id` - 最後にサインインした日時を更新するユーザー
///
/// # 戻り値
///
/// 更新日時
pub fn update_last_sign_in_at_query<'q>(user_id: UserId) -> PgQueryAs<'q, LastSignInAtRow> {
    sqlx::query_as::<Postgres, LastSignInAtRow>(
        r#"
        UPDATE
            users
        SET
            last_sign_in_at = CURRENT_TIMESTAMP,
            sign_in_attempted_at = NULL,
            number_of_sign_in_failures = 0
        WHERE
            id = $1
        RETURNING
            last_sign_in_at
        "#,
    )
    .bind(user_id.value)
}

/// 最初にサインインに失敗したことを保存するクエリを生成する。
///
/// # 引数
///
/// * `user_id` - ユーザーID
///
/// # 戻り値
///
/// 最初にサインインに失敗したことを保存するクエリ
pub fn record_first_sign_in_failed_query<'q>(user_id: UserId) -> PgQueryAs<'q, UserCredentialRow> {
    sqlx::query_as::<Postgres, UserCredentialRow>(
        r#"
        UPDATE
            users
        SET
            sign_in_attempted_at = CURRENT_TIMESTAMP,
            number_of_sign_in_failures = 1
        WHERE
            id = $1
        RETURNING
            id, email, password, active, sign_in_attempted_at, number_of_sign_in_failures
        "#,
    )
    .bind(user_id.value)
}

/// サインイン失敗回数をインクリメントするクエリを生成する。
///
/// # 引数
///
/// * `user_id` - ユーザーID
///
/// # 戻り値
///
/// サインイン失敗回数をインクリメントするクエリ
pub fn increment_number_of_sign_in_failures_query<'q>(
    user_id: UserId,
) -> PgQueryAs<'q, UserCredentialRow> {
    sqlx::query_as::<Postgres, UserCredentialRow>(
        r#"
        UPDATE
            users
        SET
            number_of_sign_in_failures = number_of_sign_in_failures + 1
        WHERE
            id = $1
        RETURNING
            id, email, password, active, sign_in_attempted_at, number_of_sign_in_failures
        "#,
    )
    .bind(user_id.value)
}

/// アクティブフラグを更新するクエリを生成する。
///
/// # 引数
///
/// * `user_id` - ユーザーID
/// * `active` - アクティブフラグ
///
/// # 戻り値
///
/// アクティブフラグを更新するクエリ
pub fn set_active_query<'q>(user_id: UserId, active: bool) -> PgQuery<'q> {
    sqlx::query::<Postgres>(
        r#"
        UPDATE
            users
        SET
            active = $1
        WHERE
            id = $2
        "#,
    )
    .bind(active)
    .bind(user_id.value)
}

/// 最初にサインインに失敗した日時をNULL、サインイン失敗回数を0にするクエリを生成する。
///
/// # 引数
///
/// * `user_id` - ユーザーID
///
/// # 戻り値
///
/// 最初にサインインに失敗したことを保存するクエリ
pub fn clear_sign_in_failed_history_query<'q>(user_id: UserId) -> PgQueryAs<'q, UserCredentialRow> {
    sqlx::query_as::<Postgres, UserCredentialRow>(
        r#"
        UPDATE
            users
        SET
            sign_in_attempted_at = NULL,
            number_of_sign_in_failures = 0
        WHERE
            id = $1
        RETURNING
            id, email, password, active, sign_in_attempted_at, number_of_sign_in_failures
        "#,
    )
    .bind(user_id.value)
}

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct LastSignInAtRow {
    last_sign_in_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
pub struct InsertedUserRow {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub active: bool,
    pub user_permission_code: i16,
    pub family_name: String,
    pub given_name: String,
    pub postal_code: String,
    pub address: String,
    pub fixed_phone_number: Option<String>,
    pub mobile_phone_number: Option<String>,
    pub remarks: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<InsertedUserRow> for SignUpOutput {
    fn from(row: InsertedUserRow) -> Self {
        Self {
            id: UserId::new(row.id),
            email: EmailAddress::new(row.email).unwrap(),
            active: row.active,
            user_permission_code: UserPermissionCode::try_from(row.user_permission_code).unwrap(),
            family_name: FamilyName::new(row.family_name).unwrap(),
            given_name: GivenName::new(row.given_name).unwrap(),
            postal_code: PostalCode::new(row.postal_code).unwrap(),
            address: Address::new(row.address).unwrap(),
            fixed_phone_number: OptionalFixedPhoneNumber::try_from(row.fixed_phone_number).unwrap(),
            mobile_phone_number: OptionalMobilePhoneNumber::try_from(row.mobile_phone_number)
                .unwrap(),
            remarks: OptionalRemarks::try_from(row.remarks).unwrap(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// ユーザーをデータベースに登録するクエリを生成する。
///
/// # 引数
///
/// * `user` - データベースに登録するユーザー
///
/// # 戻り値
///
/// ユーザーをデータベースに登録するクエリ
pub fn insert_user_query<'q>(user: SignUpInput) -> PgQueryAs<'q, InsertedUserRow> {
    let password = user.password.value.expose_secret().to_string();
    let fixed_phone_number = user.fixed_phone_number.owned_value();
    let mobile_phone_number = user.mobile_phone_number.owned_value();
    let remarks = user.remarks.owned_value();

    sqlx::query_as::<Postgres, InsertedUserRow>(
        r#"
        INSERT INTO users (
            id, email, password, active, user_permission_code, family_name, given_name,
            postal_code, address, fixed_phone_number, mobile_phone_number,
            remarks, created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
            STATEMENT_TIMESTAMP(), STATEMENT_TIMESTAMP()
        )
        RETURNING *
        "#,
    )
    .bind(user.id.value)
    .bind(user.email.value)
    .bind(password)
    .bind(user.active)
    .bind(user.user_permission_code as i16)
    .bind(user.family_name.value)
    .bind(user.given_name.value)
    .bind(user.postal_code.value)
    .bind(user.address.value)
    .bind(fixed_phone_number)
    .bind(mobile_phone_number)
    .bind(remarks)
}
