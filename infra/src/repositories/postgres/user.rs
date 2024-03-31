use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use sqlx::Postgres;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::common::{DomainError, DomainResult};
use domain::models::passwords::PhcPassword;
use domain::models::primitives::*;
use domain::models::user::{User, UserBuilder, UserId};
use domain::repositories::user::UserRepository;

use crate::repositories::postgres::common::PgRepository;

/// PostgreSQLユーザー・リポジトリ
pub type PgUserRepository = PgRepository<User>;

#[async_trait]
impl UserRepository for PgUserRepository {
    /// ユーザーを登録する。
    async fn create(&self, _user: User) -> DomainResult<User> {
        let mut _tx = self.begin().await?;

        Err(DomainError::Validation(String::from("error").into()))
    }
}

#[derive(sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub active: bool,
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

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        // データベースから取得した値を変換するためアンラップ
        let fixed_phone_number = FixedPhoneNumber::try_from(row.fixed_phone_number).unwrap();
        let mobile_phone_number = MobilePhoneNumber::try_from(row.mobile_phone_number).unwrap();
        let remarks = Remarks::try_from(row.remarks).unwrap();

        UserBuilder::new()
            .id(UserId::new(row.id))
            .email(EmailAddress::new(row.email).unwrap())
            .password(PhcPassword::new(SecretString::new(row.password)).unwrap())
            .active(row.active)
            .family_name(FamilyName::new(row.family_name).unwrap())
            .given_name(GivenName::new(row.given_name).unwrap())
            .postal_code(PostalCode::new(row.postal_code).unwrap())
            .address(Address::new(row.address).unwrap())
            .fixed_phone_number(fixed_phone_number)
            .mobile_phone_number(mobile_phone_number)
            .remarks(remarks)
            .last_logged_in_at(row.last_logged_in_at)
            .created_at(row.created_at)
            .updated_at(row.updated_at)
            .build()
            .unwrap()
    }
}

/// ユーザーをデータベースに登録するクエリを生成する。
///
/// FIXME: 実装できたが呼び出しする方法がわからない。
///
/// # 引数
///
/// * `user` - データベースに登録するユーザー
///
/// # 戻り値
///
/// ユーザーをデータベースに登録するクエリ
#[allow(dead_code)]
fn insert_user_query(
    user: &User,
) -> sqlx::query::QueryAs<'_, sqlx::Postgres, UserRow, sqlx::postgres::PgArguments> {
    sqlx::query_as::<Postgres, UserRow>(
        r#"
        INSERT INTO users (
            id, email, password, active, family_name, given_name,
            postal_code, address, fixed_phone_number, mobile_phone_number,
            remarks, last_logged_in_at, created_at, updated_at
        )
        VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, STATEMENT_TIMESTAMP(), STATEMENT_TIMESTAMP()
        )
        RETURNING *
        "#,
    )
    .bind(user.id().value())
    .bind(user.email().value())
    .bind(user.password().value().expose_secret())
    .bind(user.active())
    .bind(user.family_name().value())
    .bind(user.given_name().value())
    .bind(user.postal_code().value())
    .bind(user.address().value())
    .bind(user.fixed_phone_number().value())
    .bind(user.mobile_phone_number().value())
    .bind(user.remarks().value())
}
