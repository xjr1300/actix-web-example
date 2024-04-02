use secrecy::ExposeSecret as _;

use domain::models::primitives::EmailAddress;
use domain::models::user::{User, UserId};
use infra::repositories::postgres::user::{insert_user_query, UserRow};
use infra::repositories::postgres::{
    commit_transaction, IsolationLevel, PgRepository, PgTransaction,
};
use time::Duration;

use crate::helpers::{generate_user, spawn_test_app};

/// トランザクションを開始して、コミットできるか確認
#[tokio::test]
#[ignore]
async fn transaction_works() -> anyhow::Result<()> {
    // 準備
    let app = spawn_test_app().await?;

    // リード・コミット
    let user = generate_user(UserId::default(), EmailAddress::new("foo@example.com")?);
    let repo = PgRepository::<i32>::new(app.pool.clone());
    let tx = repo.begin().await?;
    act_and_verify(tx, &user).await?;

    // リピータブル・リード
    let user = generate_user(UserId::default(), EmailAddress::new("bar@example.com")?);
    let repo = PgRepository::<i32>::new(app.pool.clone());
    let tx = repo.begin_with_level(IsolationLevel::ReadCommit).await?;
    act_and_verify(tx, &user).await?;

    // シリアライザブル
    let user = generate_user(UserId::default(), EmailAddress::new("baz@example.com")?);
    let repo = PgRepository::<i32>::new(app.pool.clone());
    let tx = repo.begin_with_level(IsolationLevel::Serializable).await?;
    act_and_verify(tx, &user).await?;

    Ok(())
}

async fn act_and_verify(tx: PgTransaction<'_>, user: &User) -> anyhow::Result<()> {
    // 実行
    let inserted = insert_user_to_database(tx, user).await?;

    // 検証
    verity_user(user, &inserted);
    assert_eq!(inserted.created_at(), inserted.updated_at());
    assert!(
        user.created_at() - Duration::seconds(3) <= inserted.created_at(),
        "does not satisfy `{} <= {}`",
        user.created_at(),
        inserted.created_at()
    );

    Ok(())
}

async fn insert_user_to_database<'c>(
    mut tx: PgTransaction<'c>,
    user: &User,
) -> anyhow::Result<User> {
    let user_row: UserRow = insert_user_query(user).fetch_one(&mut *tx).await?;
    commit_transaction(tx).await?;

    Ok(user_row.into())
}

macro_rules! verify_primitive {
    ($left:ident, $right:ident, $field:ident) => {
        assert_eq!($left.$field().value(), $right.$field().value());
    };
}

fn verity_user(left: &User, right: &User) {
    verify_primitive!(left, right, id);
    verify_primitive!(left, right, email);
    assert_eq!(
        left.password().value().expose_secret(),
        right.password().value().expose_secret()
    );
    assert_eq!(left.active(), right.active());
    verify_primitive!(left, right, family_name);
    verify_primitive!(left, right, given_name);
    verify_primitive!(left, right, postal_code);
    verify_primitive!(left, right, address);
    verify_primitive!(left, right, fixed_phone_number);
    verify_primitive!(left, right, mobile_phone_number);
    verify_primitive!(left, right, remarks);
    assert_eq!(left.last_logged_in_at(), right.last_logged_in_at());
}
