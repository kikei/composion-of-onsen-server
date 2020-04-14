use handlebars::{
    Handlebars, Context, Helper, HelperResult,
    Output, RenderContext, RenderError
};
use serde::{Serialize, Deserialize};
use serde_json::value::{Value};

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

const MAX_FRAC: usize = 2;

fn prec_helper (h: &Helper, _: &Handlebars, _: &Context,
                _rc: &mut RenderContext, out: &mut dyn Output)
                -> HelperResult
{
    // Value to show
    let value = h.param(0);
    // Precision
    let prec = h.param(1)
        .and_then(|p| p.value().as_u64()).unwrap_or(2) as usize;
    let text = match value {
        None => None,
        Some(p) => match p.value() {
            Value::Number(n) => {
                let v = n.as_f64().unwrap();
                let i;
                let f;
                if v >= 1.0 {
                    i = (v.log(10f64).floor().abs() + 1f64) as usize;
                    f = if i > prec { 0 } else {
                        usize::min(prec - i, MAX_FRAC)
                    };
                } else {
                    i = 1;
                    f = MAX_FRAC;
                }
                Some(format!("{v:i$.f$}", v=v, i=i, f=f))
            },
            Value::String(s) => Some(s.to_string()),
            _ => None
        }
    };
    match text {
        Some(t) => {
            out.write(&t).or_else(|e| {
                warn!("prec_helper writing error: {:?}", &e);
                Ok(())
            })
        },
        None => Ok(())
    }
}

fn fixed_helper (h: &Helper, _: &Handlebars, _: &Context,
                _rc: &mut RenderContext, out: &mut dyn Output)
                -> HelperResult
{
    // Value to show
    let value = h.param(0);
    // Precision
    let prec = h.param(1)
        .and_then(|p| p.value().as_u64()).unwrap_or(2) as usize;
    let text = match value {
        None => None,
        Some(p) => match p.value() {
            Value::Number(n) => {
                let v = n.as_f64().unwrap();
                let i = if v >= 1.0 {
                    (v.log(10f64).floor().abs() + 1f64) as usize
                } else {
                    1
                };
                Some(format!("{v:i$.f$}", v=v, i=i, f=prec))
            },
            Value::String(s) => Some(s.to_string()),
            _ => None
        }
    };
    match text {
        Some(t) => {
            out.write(&t).or_else(|e| {
                warn!("prec_helper writing error: {:?}", &e);
                Ok(())
            })
        },
        None => Ok(())
    }
}
pub fn render_template<T: Serialize>(data: &T, template: &Template)
                                 -> Result<String, String> {
    // TODO reuse template
    let mut reg = Handlebars::new();
    reg.register_helper("prec", Box::new(prec_helper));
    reg.register_helper("fixed", Box::new(fixed_helper));
    reg.register_template_string(&template.name, &template.body)
        .or_else(|e| Err(format!("{}", e)))?;
    reg.render(&template.name, &data)
        .or_else(|e| Err(format!("{}", e)))
}
