use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use sqlx::Postgres;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::models::passwords::PhcPassword;
use domain::models::primitives::*;
use domain::models::user::{User, UserBuilder, UserId};
use domain::repositories::user::UserRepository;
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
    async fn create(&self, user: User) -> DomainResult<User> {
        let mut tx = self.begin().await?;
        let added_user = insert_user_query(&user)
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
        // データベースから取得した値を変換するため確認しないで値を取り出し
        let fixed_phone_number =
            OptionalFixedPhoneNumber::try_from(row.fixed_phone_number).unwrap();
        let mobile_phone_number =
            OptionalMobilePhoneNumber::try_from(row.mobile_phone_number).unwrap();
        let remarks = OptionalRemarks::try_from(row.remarks).unwrap();

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
/// # 引数
///
/// * `user` - データベースに登録するユーザー
///
/// # 戻り値
///
/// ユーザーをデータベースに登録するクエリ
pub fn insert_user_query(
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
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NULL, STATEMENT_TIMESTAMP(), STATEMENT_TIMESTAMP()
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
