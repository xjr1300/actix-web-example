use async_trait::async_trait;
use secrecy::SecretString;
use time::OffsetDateTime;
use uuid::Uuid;

use domain::common::DomainResult;
use domain::models::passwords::PhcPassword;
use domain::models::primitives::{Address, EmailAddress, FamilyName, GivenName, PostalCode};
use domain::models::user::{User, UserBuilder, UserId};
use domain::repositories::user::UserRepository;

use crate::repositories::postgres::common::PgRepository;
use crate::{
    optional_fixed_phone_number_primitive, optional_mobile_phone_number_primitive,
    optional_remarks_primitive,
};

/// PostgreSQLユーザー・リポジトリ
pub type PgUserRepository = PgRepository<User>;

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(_user: User) -> DomainResult<User> {
        todo!()
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password: String,
    active: bool,
    family_name: String,
    given_name: String,
    postal_code: String,
    address: String,
    fixed_phone_number: Option<String>,
    mobile_phone_number: Option<String>,
    remarks: Option<String>,
    last_logged_in_at: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        UserBuilder::new()
            .id(UserId::new(row.id))
            .email(EmailAddress::new(row.email).unwrap())
            .password(PhcPassword::new(SecretString::new(row.password)).unwrap())
            .active(row.active)
            .family_name(FamilyName::new(row.family_name).unwrap())
            .given_name(GivenName::new(row.given_name).unwrap())
            .postal_code(PostalCode::new(row.postal_code).unwrap())
            .address(Address::new(row.address).unwrap())
            .fixed_phone_number(optional_fixed_phone_number_primitive(
                row.fixed_phone_number,
            ))
            .mobile_phone_number(optional_mobile_phone_number_primitive(
                row.mobile_phone_number,
            ))
            .remarks(optional_remarks_primitive(row.remarks))
            .last_logged_in_at(row.last_logged_in_at)
            .created_at(row.created_at)
            .updated_at(row.updated_at)
            .build()
            .unwrap()
    }
}
