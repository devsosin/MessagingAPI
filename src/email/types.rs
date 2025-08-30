use std::{collections::HashMap, io::Error};

pub trait EmailTemplateLoader {
    async fn get_content(&self) -> Result<String, Error>;
    fn is_html(&self) -> bool;
}

pub trait ReceiverGetter {
    fn get_name(&self) -> &Option<String>;
    fn get_address(&self) -> &str;
}

pub trait ToEmailVariable {
    fn to_map(&self) -> HashMap<String, String>;
}
