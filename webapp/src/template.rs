use handlebars::Handlebars;
use serde::{Serialize, Deserialize};

pub static KEY_ID: &str = "_id";
pub static KEY_NAME: &str = "name";
// static KEY_CONTENT_TYPE: &str = "contentType";
pub static KEY_BODY: &str = "body";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    pub id: Option<String>,
    pub name: String,
    // pub content_type: ContentType,
    pub body: String
}

pub trait Render {
    fn render(&self, template: &Template) -> Result<String, String>;
}

pub fn render_template<T: Serialize>(data: &T, template: &Template)
                                 -> Result<String, String> {
    // TODO reuse template
    let mut reg = Handlebars::new();
    reg.register_template_string(&template.name, &template.body)
        .or_else(|e| Err(format!("{}", e)))?;
    reg.render(&template.name, &data)
        .or_else(|e| Err(format!("{}", e)))
}
