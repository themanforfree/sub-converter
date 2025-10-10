use crate::error::Result;
use crate::ir::Node;

pub mod clash;
pub mod sing_box;
pub mod uri;

pub trait Parser {
    fn parse(&self, input: &str) -> Result<Vec<Node>>;
}
