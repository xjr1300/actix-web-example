use time::Duration;

use domain::models::primitives::EmailAddress;
use domain::models::user::{User, UserId};
use domain::repositories::user::{SignUpInputBuilder, SingUpOutput};
use infra::repositories::postgres::user::{insert_user_query, UserRow};
use infra::repositories::postgres::{
    commit_transaction, IsolationLevel, PgRepository, PgTransaction,
};

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
    let inserted = insert_user_to_database(tx, user.clone()).await?;

    // 検証
    verity_user(user, &inserted);
    assert_eq!(inserted.created_at, inserted.updated_at);
    assert!(
        user.created_at - Duration::seconds(3) <= inserted.created_at,
        "does not satisfy `{} <= {}`",
        user.created_at,
        inserted.created_at
    );

    Ok(())
}

async fn insert_user_to_database(
    mut tx: PgTransaction<'_>,
    user: User,
) -> anyhow::Result<SingUpOutput> {
    let input = SignUpInputBuilder::new()
        .id(user.id)
        .email(user.email)
        .password(user.password)
        .active(user.active)
        .user_permission_code(user.user_permission.code)
        .family_name(user.family_name)
        .given_name(user.given_name)
        .postal_code(user.postal_code)
        .address(user.address)
        .fixed_phone_number(user.fixed_phone_number)
        .mobile_phone_number(user.mobile_phone_number)
        .remarks(user.remarks)
        .build()
        .unwrap();
    let user_row: UserRow = insert_user_query(input).fetch_one(&mut *tx).await?;
    commit_transaction(tx).await?;

    Ok(user_row.into())
}

fn verity_user(left: &User, right: &SingUpOutput) {
    assert_eq!(left.id, right.id);
    assert_eq!(left.email, right.email);
    //assert_eq!(
    //    left.password.value.expose_secret(),
    //    right.password.value.expose_secret()
    //);
    //assert_eq!(left.active, right.active);
    //verify_primitive!(left, right, family_name);
    //verify_primitive!(left, right, given_name);
    //verify_primitive!(left, right, postal_code);
    //verify_primitive!(left, right, address);
    //verify_primitive_value!(left, right, fixed_phone_number);
    //verify_primitive_value!(left, right, mobile_phone_number);
    //verify_primitive_value!(left, right, remarks);
}
