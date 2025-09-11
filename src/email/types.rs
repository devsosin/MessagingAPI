pub trait ReceiverGetter {
    fn get_name(&self) -> &Option<String>;
    fn get_address(&self) -> &str;
}
