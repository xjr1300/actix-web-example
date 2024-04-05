use async_trait::async_trait;
use secrecy::ExposeSecret;
use sqlx::Postgres;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::models::primitives::*;
use domain::models::user::{User, UserId, UserPermission, UserPermissionCode, UserPermissionName};
use domain::repositories::user::{SignUpInput, SignUpOutput, UserRepository};
use domain::{DomainError, DomainResult};

use crate::repositories::postgres::{commit_transaction, PgRepository};

/// PostgreSQLユーザー・リポジトリ
pub type PgUserRepository = PgRepository<User>;

#[async_trait]
impl UserRepository for PgUserRepository {
    /// ユーザーのリストを取得する。
    async fn list(&self) -> DomainResult<Vec<User>> {
        Ok(list_users_query()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Repository(e.into()))?
            .into_iter()
            .map(|r| r.into())
            .collect::<_>())
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
            .map_err(|e| DomainError::Repository(e.into()))?;
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
    pub last_logged_in_at: Option<OffsetDateTime>,
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
                UserPermissionCode::new(row.user_permission_code),
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
            last_logged_in_at: row.last_logged_in_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
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
    pub last_logged_in_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<InsertedUserRow> for SignUpOutput {
    fn from(row: InsertedUserRow) -> Self {
        Self {
            id: UserId::new(row.id),
            email: EmailAddress::new(row.email).unwrap(),
            active: row.active,
            user_permission_code: UserPermissionCode::new(row.user_permission_code),
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

type PgQueryAs<'a, T> = sqlx::query::QueryAs<'a, sqlx::Postgres, T, sqlx::postgres::PgArguments>;

/// ユーザーのリストを取得するクエリを生成する。
///
/// # 戻り値
///
/// ユーザーの一覧を取得するクエリ
pub fn list_users_query<'a>() -> PgQueryAs<'a, RetrievedUserRow> {
    sqlx::query_as::<Postgres, RetrievedUserRow>(
        r#"
        SELECT
            u.id, u.email, u.password, u.active, u.user_permission_code, p.name user_permission_name,
            u.family_name, u.given_name, u.postal_code, u.address, u.fixed_phone_number, u.mobile_phone_number,
            u.remarks, u.last_logged_in_at, u.created_at, u.updated_at
        FROM users u
        INNER JOIN user_permissions p ON u.user_permission_code = p.code
        ORDER BY created_at
        "#,
    )
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
pub fn insert_user_query<'a>(user: SignUpInput) -> PgQueryAs<'a, InsertedUserRow> {
    let password = user.password.value.expose_secret().to_string();
    let fixed_phone_number = user.fixed_phone_number.value().map(|n| n.to_string());
    let mobile_phone_number = user.mobile_phone_number.value().map(|n| n.to_string());
    let remarks = user.remarks.value().map(|n| n.to_string());

    sqlx::query_as::<Postgres, InsertedUserRow>(
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
