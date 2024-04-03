use async_trait::async_trait;
use secrecy::ExposeSecret;
use sqlx::Postgres;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::models::primitives::*;
use domain::models::user::{User, UserId};
use domain::repositories::user::{SignUpUser, SignedUpUser, UserRepository};
use domain::{DomainError, DomainResult};

use crate::repositories::postgres::{commit_transaction, PgRepository};

/// PostgreSQLユーザー・リポジトリ
pub type PgUserRepository = PgRepository<User>;

#[async_trait]
impl UserRepository for PgUserRepository {
    /// ユーザーを登録する。
    ///
    /// ユーザーを登録するとき、ユーザーの作成日時と更新日時は何らかの日時を設定する。
    /// 登録後に返されるユーザーの作成日時と更新日時の作成日時と更新日時には、データベースに登録
    /// した日時が設定されている。
    async fn create(&self, user: SignUpUser) -> DomainResult<SignedUpUser> {
        let mut tx = self.begin().await?;
        let added_user = insert_user_query(user)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| DomainError::Unexpected(e.into()))?;
        commit_transaction(tx).await?;

        Ok(added_user.into())
    }
}

#[derive(sqlx::FromRow)]
pub struct UserRow {
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
    pub last_logged_in_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<UserRow> for SignedUpUser {
    fn from(value: UserRow) -> Self {
        Self {
            id: UserId::new(value.id),
            email: EmailAddress::new(value.email).unwrap(),
            created_at: value.created_at,
            updated_at: value.updated_at,
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
pub fn insert_user_query(
    user: SignUpUser,
) -> sqlx::query::QueryAs<'static, sqlx::Postgres, UserRow, sqlx::postgres::PgArguments> {
    let password = user.password.value.expose_secret().to_string();
    let fixed_phone_number = user.fixed_phone_number.value().map(|n| n.to_string());
    let mobile_phone_number = user.mobile_phone_number.value().map(|n| n.to_string());
    let remarks = user.remarks.value().map(|n| n.to_string());

    sqlx::query_as::<Postgres, UserRow>(
        r#"
        INSERT INTO users (
            id, email, password, active, user_permission_code, family_name, given_name,
            postal_code, address, fixed_phone_number, mobile_phone_number,
            remarks, last_logged_in_at, created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
            NULL, STATEMENT_TIMESTAMP(), STATEMENT_TIMESTAMP()
        )
        RETURNING *
        "#,
    )
    .bind(user.id.value)
    .bind(user.email.value)
    .bind(password)
    .bind(user.active)
    .bind(user.user_permission_code.value)
    .bind(user.family_name.value)
    .bind(user.given_name.value)
    .bind(user.postal_code.value)
    .bind(user.address.value)
    .bind(fixed_phone_number)
    .bind(mobile_phone_number)
    .bind(remarks)
}
