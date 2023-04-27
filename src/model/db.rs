use std::env;

use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

/// 環境変数を用いて、db に接続する
pub async fn connect_db() -> anyhow::Result<MySqlPool> {
    dotenv::dotenv().ok();
    let hostname = env::var("MARIADB_HOSTNAME").unwrap();
    let database = env::var("MARIADB_DATABASE").unwrap();
    let username = env::var("MARIADB_USERNAME").unwrap();
    let password = env::var("MARIADB_PASSWORD").unwrap();

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&format!(
            "mysql://{}:{}@{}/{}",
            username, password, hostname, database
        ))
        .await?;
    Ok(pool)
}

pub async fn save(
    pool: &MySqlPool,
    key: &str,
    value: &str,
    user_id: &str,
    user_name: &str,
) -> sqlx::Result<()> {
    let query = r#"
        INSERT INTO `regexps` (`key`, `regexp`, `user_id`, `user_name`) VALUES (?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(key)
        .bind(value)
        .bind(user_id)
        .bind(user_name)
        .execute(pool)
        .await?;

    Ok(())
}

#[derive(sqlx::FromRow, Debug)]
pub struct Count {
    pub count: i64,
}
pub async fn remove(pool: &MySqlPool, key: &str, user_id: &str) -> sqlx::Result<bool> {
    let query = r#"
        DELETE FROM `regexps` WHERE `key` = ? AND `user_id` = ?
    "#;

    sqlx::query(query)
        .bind(key)
        .bind(user_id)
        .execute(pool)
        .await?;

    let query = r#"
        SELECT ROW_COUNT() AS `count`
    "#;

    let row: Count = sqlx::query_as(query).fetch_one(pool).await?;

    Ok(row.count > 0)
}

#[derive(sqlx::FromRow, Debug)]
pub struct Regexp {
    pub key: String,
    pub regexp: String,
    pub user_id: String,
    pub user_name: String,
}
pub async fn get(pool: &MySqlPool, key: &str) -> sqlx::Result<Option<String>> {
    let query = r#"
        SELECT `regexp` FROM `regexps` WHERE `key` = ?
    "#;

    let row: Option<Regexp> = sqlx::query_as(query).bind(key).fetch_optional(pool).await?;

    Ok(row.map(|row| row.regexp))
}
