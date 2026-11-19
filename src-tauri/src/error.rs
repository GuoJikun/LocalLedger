use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("数据库连接锁已被占用")]
    LockPoisoned,

    #[error("参数无效: {0}")]
    Invalid(String),

    #[error("找不到 ID 为 {0} 的记录")]
    NotFound(String),

    #[error("设备标识尚未初始化")]
    DeviceNotInitialized,
}

pub type AppResult<T> = Result<T, AppError>;

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}
