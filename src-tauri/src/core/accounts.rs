use crate::caldav::{AccountTestResult, CalDavClient};
use crate::models::{Account, CreateAccount, UpdateAccount};
use crate::sync::{AccountSyncResult, SyncEngine};
use chrono::Utc;
use sqlx::SqlitePool;

use super::error::CoreError;
use super::tasks::parse_datetime;

type Result<T> = std::result::Result<T, CoreError>;

/// Result of creating an account with immediate sync.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateAccountResult {
    pub account: Account,
    pub sync_result: AccountSyncResult,
}

/// Get all accounts.
pub async fn get_accounts(pool: &SqlitePool) -> Result<Vec<Account>> {
    let rows = sqlx::query_as!(
        AccountRow,
        r#"
        SELECT id, name, server_url, username, password,
               principal_url, calendar_home_url, created_at, updated_at
        FROM accounts
        ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Account::from).collect())
}

/// Get a single account by ID.
pub async fn get_account(id: i64, pool: &SqlitePool) -> Result<Account> {
    let row = sqlx::query_as!(
        AccountRow,
        r#"
        SELECT id, name, server_url, username, password,
               principal_url, calendar_home_url, created_at, updated_at
        FROM accounts
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| CoreError::NotFound(format!("Account with id {}", id)))?;

    Ok(Account::from(row))
}

/// Create a new account.
pub async fn create_account(data: CreateAccount, pool: &SqlitePool) -> Result<Account> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query!(
        r#"
        INSERT INTO accounts (name, server_url, username, password,
                              principal_url, calendar_home_url, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        data.name,
        data.server_url,
        data.username,
        data.password,
        data.principal_url,
        data.calendar_home_url,
        now,
        now
    )
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    get_account(id, pool).await
}

/// Update an existing account.
pub async fn update_account(id: i64, data: UpdateAccount, pool: &SqlitePool) -> Result<Account> {
    let existing = get_account(id, pool).await?;
    let now = Utc::now().to_rfc3339();

    let name = data.name.unwrap_or(existing.name);
    let server_url = data.server_url.unwrap_or(existing.server_url);
    let username = data.username.unwrap_or(existing.username);
    let password = data.password.unwrap_or(existing.password);
    let principal_url = data.principal_url.or(existing.principal_url);
    let calendar_home_url = data.calendar_home_url.or(existing.calendar_home_url);

    sqlx::query!(
        r#"
        UPDATE accounts
        SET name = ?, server_url = ?, username = ?, password = ?,
            principal_url = ?, calendar_home_url = ?, updated_at = ?
        WHERE id = ?
        "#,
        name,
        server_url,
        username,
        password,
        principal_url,
        calendar_home_url,
        now,
        id
    )
    .execute(pool)
    .await?;

    get_account(id, pool).await
}

/// Delete an account.
pub async fn delete_account(id: i64, pool: &SqlitePool) -> Result<()> {
    let result = sqlx::query!("DELETE FROM accounts WHERE id = ?", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(CoreError::NotFound(format!("Account with id {}", id)));
    }

    Ok(())
}

/// Create an account and immediately sync to import calendars.
pub async fn create_account_and_sync(
    data: CreateAccount,
    pool: &SqlitePool,
) -> Result<CreateAccountResult> {
    // 1. Create the account
    let account = create_account(data, pool).await?;

    // 2. Sync the account to discover and import calendars
    let engine = SyncEngine::new(SqlitePool::clone(pool));
    let sync_result = engine.sync_account(account.id).await;

    Ok(CreateAccountResult {
        account,
        sync_result,
    })
}

/// Test a CalDAV connection and discover endpoints.
pub async fn test_account_connection(
    server_url: &str,
    username: &str,
    password: &str,
) -> Result<AccountTestResult> {
    let client = CalDavClient::new(server_url, username, password)
        .map_err(|e| CoreError::CalDav(e.to_string()))?;

    Ok(client.test_connection().await)
}

// --- Row type for sqlx mapping ---

pub(crate) struct AccountRow {
    id: i64,
    name: String,
    server_url: String,
    username: String,
    password: String,
    principal_url: Option<String>,
    calendar_home_url: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<AccountRow> for Account {
    fn from(row: AccountRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            server_url: row.server_url,
            username: row.username,
            password: row.password,
            principal_url: row.principal_url,
            calendar_home_url: row.calendar_home_url,
            created_at: parse_datetime(&row.created_at),
            updated_at: parse_datetime(&row.updated_at),
        }
    }
}
