use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool, Row};
use chrono::NaiveDateTime;

pub struct SqlManipulator {
    pool: Pool<MySql>
}

impl SqlManipulator {
    pub async fn new(mysql_str: &str) -> Self {
        let db_url = mysql_str;
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await
            .unwrap();

        SqlManipulator {
            pool,
        }
    }

    pub async fn init_file_info(&self, file_name: &str, file_size: i64, file_checksum: u32) -> Result<i32, sqlx::Error> {
        // 开启一个事务
        let mut tx = self.pool.begin().await?;

        // 执行插入操作
        sqlx::query(
            "INSERT INTO file_info (file_name, file_size, file_checksum, file_status) VALUES (?, ?, ?, 0)",
        )
        .bind(file_name)
        .bind(file_size)
        .bind(file_checksum)
        .execute(&mut *tx)
        .await?;

        // 获取插入记录的自增 ID
        let id = sqlx::query("SELECT LAST_INSERT_ID() as id")
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(id.get("id"))
    }

    pub async fn write_block_info(&self, file_id: i32, block_id: i64, block_name: &str, block_size: u32, block_checksum: u32) -> Result<(), sqlx::Error> {
        sqlx::query_scalar!(
            "INSERT INTO file_block (file_id, block_name, block_id, block_checksum, block_size) VALUES (?, ?, ?, ?, ?)",
            file_id,
            block_name,
            block_id,
            block_checksum,
            block_size,
        ).fetch_one(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn finish_file_info(&self, file_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query_scalar!(
            "UPDATE file_info SET file_status = 1 WHERE id = ?",
            file_id,
        ).fetch_one(&self.pool)
        .await?;
        Ok(())
    }
    
    pub async fn delete_file_info(&self, file_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query_scalar!(
            "UPDATE file_info SET file_status = 2 WHERE id = ?",
            file_id,
        ).fetch_one(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn register(&self, user_name: &str, password: &str) -> Result<(), sqlx::Error> {
        sqlx::query_scalar!(
            "INSERT INTO user (user_name, user_password) VALUES (?, ?)",
            user_name,
            password,
        ).fetch_one(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn check_user_exists(&self, user_name: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user WHERE user_name = ?",
            user_name,
        ).fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn login(&self, user_name: &str, password: &str) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user WHERE user_name = ? AND user_password = ?",
            user_name,
            password,
        ).fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn get_file_ids(&self) -> Result<Vec<i32>, sqlx::Error> {
        let file_ids = sqlx::query_scalar!(
            "SELECT id FROM file_info WHERE file_status = 1",
        ).fetch_all(&self.pool)
        .await?;
        Ok(file_ids)
    }

    pub async fn get_file_info_by_id(&self, id: i32) -> Result<FileInfo, sqlx::Error> {
        let file_info = sqlx::query_as!(
            FileInfo,
            "SELECT * FROM file_info WHERE id = ?",
            id,
        ).fetch_one(&self.pool)
        .await?;
        Ok(file_info)
    }

    pub async fn get_file_block_ids_by_file_id(&self, file_id: i32) -> Result<Vec<i32>, sqlx::Error> {
        let block_ids = sqlx::query_scalar!(
            "SELECT id FROM file_block WHERE file_id = ?",
            file_id,
        ).fetch_all(&self.pool)
        .await?;
        Ok(block_ids)
    }

    pub async fn get_block_info_by_id(&self, block_id: i32) -> Result<FileBlock, sqlx::Error> {
        let block_info = sqlx::query_as!(
            FileBlock,
            "SELECT * FROM file_block WHERE id = ?",
            block_id,
        ).fetch_one(&self.pool)
        .await?;
        Ok(block_info)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct FileBlock {
    id: i32,
    file_id: i32,
    block_name: String,
    block_id: i64,
    block_checksum: u32,
    block_size: u32,
    created_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct FileInfo {
    id: i32,
    file_name: String,
    file_size: i64,
    file_checksum: u32,
    file_status: i32,
    created_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    id: i32,
    user_name: String,
    password: String,
    created_at: NaiveDateTime,
}
