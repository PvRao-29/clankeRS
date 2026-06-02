use thiserror::Error;

#[derive(Debug, Error)]
pub enum RobotError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("topic error: {0}")]
    Topic(String),

    #[error("model error: {0}")]
    Model(String),

    #[error("data error: {0}")]
    Data(String),

    #[error("ros2 error: {0}")]
    Ros2(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] toml::de::Error),

    #[error("{0}")]
    Other(String),
}

pub type RobotResult<T> = Result<T, RobotError>;
