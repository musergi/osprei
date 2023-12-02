#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StageDefinition {
    pub name: String,
    pub image: String,
    pub environment: Vec<EnvironmentVariable>,
    pub working_dir: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Template {
    pub name: String,
    pub image: String,
    pub environment: Vec<EnvironmentVariable>,
}
