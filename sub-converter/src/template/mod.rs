use crate::{
    OutputFormat,
    formats::{ClashConfig, SingBoxConfig},
};

#[derive(Debug, Clone)]
pub enum Template {
    Clash(ClashConfig),
    SingBox(SingBoxConfig),
}

impl Template {
    pub fn target(&self) -> OutputFormat {
        match self {
            Template::Clash(_) => OutputFormat::Clash,
            Template::SingBox(_) => OutputFormat::SingBox,
        }
    }
}
