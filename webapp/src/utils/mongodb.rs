use r2d2_mongodb::mongodb::Document;

pub fn document_str(item: &Document, key: &str) -> Option<String> {
    item.get_str(key).ok().map(|v| String::from(v))
}

pub fn document_number(item: &Document, key: &str) -> Option<f64> {
    item.get_f64(key).ok()
}
