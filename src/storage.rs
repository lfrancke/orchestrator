pub trait Storage {
    fn get(&self, key: &str);
    fn create(&self, key: &str, obj: &Vec<u8>); // TODO: obj should not be a &str but what should it be?
}
