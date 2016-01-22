pub enum Format {
    Text
}

pub struct Formatted<'a> {
    pub content_type: &'a str,
    pub body: Vec<u8>
}
