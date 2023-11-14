use crate::{CONTINUATION_EXPIRY_SECONDS, MAX_PAYLOAD_SIZE};
use crate::user::User;
use bc_components::{PublicKeyBase, ARID, PrivateKeyBase};
use bc_envelope::prelude::*;
use mysql_async::{Pool, Result as MySqlResult, Row};
use mysql_async::prelude::*;
use url::Url;
use anyhow::anyhow;

const USER: &str = "root";
const PASSWORD: Option<&str> = None;
const HOST: &str = "localhost";
const PORT: u16 = 3306;

const DATABASE_NAME: &str = "depo";
const USERS_TABLE_NAME: &str = "users";
const RECORDS_TABLE_NAME: &str = "records";
const SETTINGS_TABLE_NAME: &str = "settings";

pub fn server_url() -> Url {
    let mut server_url = Url::parse("mysql://").unwrap();
    server_url.set_host(Some(HOST)).unwrap();
    server_url.set_username(USER).unwrap();
    server_url.set_password(PASSWORD).unwrap();
    server_url.set_port(Some(PORT)).unwrap();
    server_url
}

pub fn database_url() -> Url {
    let mut database_url = server_url();
    database_url.set_path(DATABASE_NAME);
    database_url
}

pub fn server_pool() -> Pool {
    Pool::new(server_url().as_str())
}

pub fn db_pool() -> Pool {
    Pool::new(database_url().as_str())
}

pub async fn drop_db(server_pool: &Pool) -> MySqlResult<()> {
    let query = format!("DROP DATABASE IF EXISTS {}", DATABASE_NAME);
    server_pool.get_conn().await?.query_drop(query).await?;

    Ok(())
}

// user_id:
// ur:arid/hdcxcwbkecfyftvljplpfdrkinpapecacxnlbbtpaxweprgujpashsgwihsofxecdkttsbmekpie

// public_key:
// ur:crypto-pubkeys/lftanshfhdcxdpurmndyfncxbyheoxjyctcwtnnbuogustnbrkoxjpdkgtjlsfgyhefmfspmknottansgrhdcxryidwslaesdkmwjkcmtthslrtdchwnstkkylwsdwbnnnpflpfzmhhhrkeogmclhhjnjemkhh

// private_key:
// ur:crypto-prvkeys/hdcxtouovttbkbhkayaxbahewzbndlswpdehlrfhfphfvlbzoyjtempaecahgrtbjzcxwnrnhpkb

// receipt:
// ur:envelope/lftpcshdcxbgryatktiacpbteycnynsnjywktlbyaxwznskgosbdiskohhtpwybwspglvwadgmoyadtpcsiogmihiaihinjojycwswqdbd

pub async fn create_db(server_pool: &Pool) -> MySqlResult<()> {
    let query = format!("CREATE DATABASE IF NOT EXISTS {}", DATABASE_NAME);
    server_pool.get_conn().await?.query_drop(query).await?;

    let query = format!(r"
        CREATE TABLE IF NOT EXISTS {}.{} (
            user_id VARCHAR(100) NOT NULL,
            public_key VARCHAR(200) NOT NULL,
            recovery VARCHAR(1000),
            PRIMARY KEY (user_id),
            INDEX (public_key),
            INDEX (recovery)
        )
    ", DATABASE_NAME, USERS_TABLE_NAME);
    server_pool.get_conn().await?.query_drop(query).await?;

    let query = format!(r"
        CREATE TABLE IF NOT EXISTS {}.{} (
            receipt VARCHAR(150) NOT NULL,
            user_id VARCHAR(100) NOT NULL,
            data BLOB NOT NULL,
            PRIMARY KEY (receipt),
            INDEX (user_id),
            FOREIGN KEY (user_id) REFERENCES {}.{}(user_id) ON DELETE CASCADE
        )
    ", DATABASE_NAME, RECORDS_TABLE_NAME, DATABASE_NAME, USERS_TABLE_NAME);

    server_pool.get_conn().await?.query_drop(query).await?;
    let query = format!(r"
        CREATE TABLE IF NOT EXISTS {}.{} (
            private_key VARCHAR(120),
            continuation_expiry_seconds INT UNSIGNED,
            max_payload_size INT UNSIGNED
        )
    ", DATABASE_NAME, SETTINGS_TABLE_NAME);
    server_pool.get_conn().await?.query_drop(query).await?;

    let private_key = PrivateKeyBase::new().ur_string();

    let query = format!(r"
        INSERT INTO {}.{}
        (private_key, continuation_expiry_seconds, max_payload_size) VALUES ('{}', {}, {})
    ",
    DATABASE_NAME, SETTINGS_TABLE_NAME,
    private_key, CONTINUATION_EXPIRY_SECONDS, MAX_PAYLOAD_SIZE
);
    server_pool.get_conn().await?.query_drop(query).await?;

    Ok(())
}

pub async fn reset_db() -> MySqlResult<()> {
    let server_pool = server_pool();
    drop_db(&server_pool).await?;
    create_db(&server_pool).await?;

    Ok(())
}

pub async fn key_to_user(pool: &Pool, key: impl AsRef<PublicKeyBase>) -> MySqlResult<Option<User>> {
    let mut conn = pool.get_conn().await?;
    let query = "SELECT user_id, public_key, recovery FROM users WHERE public_key = :key";
    let params = params! {
        "key" => key.as_ref().envelope().ur_string()
    };

    let result: Option<Row> = conn.exec_first(query, params).await?;
    if let Some(row) = result {
        Ok(Some(row_to_user(row)))
    } else {
        Ok(None)
    }
}

pub async fn id_to_user(pool: &Pool, user_id: impl AsRef<ARID>) -> MySqlResult<Option<User>> {
    let mut conn = pool.get_conn().await?;
    let query = "SELECT user_id, public_key, recovery FROM users WHERE user_id = :user_id";
    let params = params! {
        "user_id" => user_id.as_ref().envelope().ur_string()
    };

    let result: Option<Row> = conn.exec_first(query, params).await?;
    if let Some(row) = result {
        Ok(Some(row_to_user(row)))
    } else {
        Ok(None)
    }
}

fn row_to_user(row: Row) -> User {
    let user_id_string: String = row.get("user_id").unwrap();
    let user_id = ARID::from_ur_string(user_id_string).unwrap();
    let public_key_string: String = row.get("public_key").unwrap();
    let public_key = PublicKeyBase::from_ur_string(public_key_string).unwrap();
    let recovery: Option<String> = row.get("recovery");

    User::new_opt(user_id, public_key, recovery)
}

pub async fn get_settings(pool: &Pool) -> anyhow::Result<(PrivateKeyBase, u32, u32)> {
    let mut conn = pool.get_conn().await?;
    let query = format!("SELECT private_key, continuation_expiry_seconds, max_payload_size FROM {}.{}", DATABASE_NAME, SETTINGS_TABLE_NAME);

    let result: Option<Row> = conn.query_first(query).await?;
    match result {
        Some(row) => {
            let private_key_string: String = row.get("private_key").ok_or_else(|| anyhow!("Private key not found"))?;
            let private_key = PrivateKeyBase::from_ur_string(private_key_string)?;
            let continuation_expiry_seconds: u32 = row.get("continuation_expiry_seconds").ok_or_else(|| anyhow!("Continuation expiry seconds not found"))?;
            let max_payload_size: u32 = row.get("max_payload_size").ok_or_else(|| anyhow!("Max payload size not found"))?;

            Ok((private_key, continuation_expiry_seconds, max_payload_size))
        },
        None => Err(anyhow!("Settings not found")),
    }
}
