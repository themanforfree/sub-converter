use crate::{
    OutputFormat,
    formats::{ClashConfig, SingBoxConfig},
};

#[derive(Debug, Clone)]
pub enum Template {
    ClashYaml(ClashConfig),
    ClashJson(ClashConfig),
    SingBoxYaml(SingBoxConfig),
    SingBoxJson(SingBoxConfig),
}

impl Template {
    pub fn target(&self) -> OutputFormat {
        match self {
            Template::ClashYaml(_) => OutputFormat::ClashYaml,
            Template::ClashJson(_) => OutputFormat::ClashJson,
            Template::SingBoxYaml(_) => OutputFormat::SingBoxYaml,
            Template::SingBoxJson(_) => OutputFormat::SingBoxJson,
        }
    }
}
