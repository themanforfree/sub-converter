use crate::formats::{ClashConfig, SingBoxConfig};

#[derive(Debug, Clone)]
pub enum Template {
    Clash(ClashConfig),
    SingBox(SingBoxConfig),
}

#[derive(Debug, Clone, Copy)]
pub enum OutputEncoding {
    Json,
    Yaml,
}
