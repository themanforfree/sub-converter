use crate::error::Result;
use crate::ir::Subscription;
use crate::template::Template;

pub mod clash;
pub mod sing_box;

pub trait Emitter {
    fn emit(&self, sub: &Subscription, tpl: &Template) -> Result<String>;
}
