use serde::{Deserialize, Serialize};

use crate::{CONFIG_DIR, ConfigError};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct MLConfig {
    pub host: String,
    pub port: u16,
    pub model: String,
    pub version: u32,
    pub dims: u32,
    pub message_size_mb: usize,
    #[serde(default)]
    pub dim_reduction_model: DimensionsReductionModel,
    #[serde(default)]
    pub cluster_model: ClusterModel,
    #[serde(default)]
    pub umap: UmapConfig,
    #[serde(default)]
    pub lda: LdaConfig,
    #[serde(default)]
    pub lsa: LsaConfig,
    #[serde(default)]
    pub hdbscan: HdbscanConfig,
    #[serde(default)]
    pub kmeans: KmeansConfig,
    #[serde(default)]
    pub dbscan: DbscanConfig,
}

#[derive(Deserialize, Serialize)]
pub enum ClusterModel {
    HDBSCAN,
    DBSCAN,
    KMEANS,
}

impl ClusterModel {
    pub fn parse(&self) -> &str {
        match self {
            Self::HDBSCAN => "hdbscan",
            Self::DBSCAN => "dbscan",
            Self::KMEANS => "kmeans",
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum DimensionsReductionModel {
    LDA,
    LSA,
    UMAP,
    NONE,
}

impl DimensionsReductionModel {
    pub fn parse(&self) -> &str {
        match self {
            Self::LDA => "lda",
            Self::LSA => "lsa",
            Self::UMAP => "umap",
            Self::NONE => "none",
        }
    }
}

impl Default for ClusterModel {
    fn default() -> Self {
        Self::HDBSCAN
    }
}

impl Default for DimensionsReductionModel {
    fn default() -> Self {
        Self::UMAP
    }
}

#[derive(Deserialize, Serialize)]
pub struct UmapConfig {
    pub n_components: usize,
    pub n_neighbors: usize,
    pub min_distance: f32,
    pub metric: String,
}

#[derive(Deserialize, Serialize)]
pub struct LdaConfig {
    pub n_components: usize,
    pub max_iter: usize,
    pub learning_method: String,
}

#[derive(Deserialize, Serialize)]
pub struct LsaConfig {
    pub n_components: usize,
}

#[derive(Deserialize, Serialize)]
pub struct HdbscanConfig {
    pub min_samples: Option<usize>,
    pub metric: String,
}

#[derive(Deserialize, Serialize)]
pub struct KmeansConfig {
    pub k_clusters: usize,
    pub init_position: String,
    pub metric: String,
}

#[derive(Deserialize, Serialize)]
pub struct DbscanConfig {
    pub min_pts: usize,
    pub epsilon: f32,
    pub metric: String,
}
impl Default for UmapConfig {
    fn default() -> Self {
        Self {
            n_components: 30,
            n_neighbors: 5,
            min_distance: 0.5,
            metric: String::from("euclidean"),
        }
    }
}
impl Default for LsaConfig {
    fn default() -> Self {
        Self { n_components: 15 }
    }
}
impl Default for LdaConfig {
    fn default() -> Self {
        Self {
            n_components: 15,
            max_iter: 100,
            learning_method: String::from("online"),
        }
    }
}

impl Default for HdbscanConfig {
    fn default() -> Self {
        Self {
            min_samples: Some(10),
            metric: String::from("euclidean"),
        }
    }
}

impl Default for KmeansConfig {
    fn default() -> Self {
        Self {
            k_clusters: 3,
            metric: String::from("cosine"),
            init_position: String::from("kmeans++"),
        }
    }
}

impl Default for DbscanConfig {
    fn default() -> Self {
        Self {
            min_pts: 3,
            metric: String::from("cosine"),
            epsilon: 0.5,
        }
    }
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 50032,
            version: 1,
            model: "BAAI/bge-small-en-v1.5".to_string(),
            dims: 384,
            message_size_mb: 64,
            dim_reduction_model: DimensionsReductionModel::default(),
            cluster_model: ClusterModel::default(),
            umap: UmapConfig::default(),
            lda: LdaConfig::default(),
            lsa: LsaConfig::default(),
            hdbscan: HdbscanConfig::default(),
            dbscan: DbscanConfig::default(),
            kmeans: KmeansConfig::default(),
        }
    }
}

impl MLConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let path = CONFIG_DIR
            .as_ref()
            .ok_or(ConfigError::NoDirsFound)?
            .join("ml.toml");

        if !path.exists() {
            return Ok(MLConfig::default());
        }

        let content = std::fs::read_to_string(path)?;

        Ok(toml::from_str(&content)?)
    }

    pub fn address(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}
