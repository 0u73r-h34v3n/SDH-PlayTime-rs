#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub id: String,
    pub name: String,
}

impl Game {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameChecksum {
    pub game: Game,
    pub checksum: String,
    pub algorithm: ChecksumAlgorithm,
    pub chunk_size: usize,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    Sha256,
    Md5,
}

impl std::fmt::Display for ChecksumAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sha256 => write!(f, "sha256"),
            Self::Md5 => write!(f, "md5"),
        }
    }
}
