use handlebars::{
    Handlebars, Context, Helper, HelperResult,
    Output, RenderContext
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

fn lookup_html_formula(key: &str) -> &str {
    match key {
        "H" => "H<sup>+</sup>",
        "Li" => "Li<sup>+</sup>",
        "B" => "B<sup>3+</sup>",
        "C" => "C<sup>4+</sup>",
        "N" => "N<sup>3-</sup>",
        "O" => "O<sup>2-</sup>",
        "F" => "F<sup>-</sup>",
        "Na" => "Na<sup>+</sup>",
        "Mg" => "Mg<sup>2+</sup>",
        "Al" => "Al<sup>3+</sup>",
        "Si" => "Si<sup>4-</sup>",
        "P" => "P<sup>3-</sup>",
        "S" => "S<sup>2-</sup>",
        "Cl" => "Cl<sup>-</sup>",
        "K" => "K<sup>+</sup>",
        "Ca" => "Ca<sup>2+</sup>",
        "Cr" => "Cr",
        "MnII" => "Mn<sup>2+</sup>",
        "FeII" => "Fe<sup>2+</sup>",
        "FeIII" => "Fe<sup>3+</sup>",
        "Cu" => "Cu",
        "CuII" => "Cu<sup>2+</sup>",
        "Zn" => "Zn",
        "ZnII" => "Zn<sup>2+</sup>",
        "As" => "As",
        "Br" => "Br<sup>-</sup>",
        "Sr" => "Sr<sup>2+</sup>",
        "Ag" => "Ag",
        "Cd" => "Cd",
        "I" => "I<sup>-</sup>",
        "Ba" => "Ba<sup>2+</sup>",
        "Hg" => "Hg",
        "Pb" => "Pb",
        "H2SiO3" => "H<sub>2</sub>SiO<sub>3</sub>",
        "H2S" => "H<sub>2</sub>S",
        "HBO2" => "HBO<sub>2</sub>",
        "HCO3" => "HCO<sub>3</sub><sup>-</sup>",
        "HNO2" => "HNO<sub>2</sub><sup>-</sup>",
        "HSiO3" => "HSiO<sub>3</sub><sup>-</sup>",
        "HPO4" => "HPO<sub>4</sub><sup>2-</sup>",
        "HS" => "HS<sup>-</sup>",
        "HSO4" => "HSO<sub>4</sub><sup>-</sup>",
        "HAsO2" => "HAsO<sub>2</sub>",
        "BO2" => "BO<sub>2</sub>",
        "CO2" => "CO<sub>2</sub>",
        "CO3" => "CO<sub>3</sub><sup>2-</sup>",
        "NH4" => "NH<sub>4</sub><sup>+</sup>",
        "NO3" => "NO<sub>3</sub><sup>-</sup>",
        "OH" => "OH<sup>-</sup>",
        "PO4" => "PO<sub>4</sub><sup>2-</sup>",
        "SO4" => "SO<sub>4</sub><sup>2-</sup>",
        "S2O3" => "S<sub>2</sub>O<sub>3</sub><sup>2-</sup>",
        "AsO2" => "AsO<sub>2</sub><sup>-</sup>",
        _ => "Unknown"
    }
}

fn htmlf_helper(h: &Helper, _: &Handlebars, _: &Context,
                 _rc: &mut RenderContext, out: &mut dyn Output)
                 -> HelperResult
{
    // Key of a component
    let value = h.param(0);
    let text = match value {
        None => None,
        Some(p) => match p.value() {
            Value::String(s) => Some(lookup_html_formula(s.as_str())),
            _ => None
        }
    };
    match text {
        Some(t) => {
            out.write(&t).or_else(|e| {
                warn!("htmlf_helper writing error: {:?}", &e);
                Ok(())
            })
        },
        None => Ok(())
    }
}


// Resources:
// - Handlebars https://handlebarsjs.com/
// - Rust handlebars https://docs.rs/handlebars/3.0.1/handlebars/index.html

pub fn render_template<T: Serialize>(data: &T, template: &Template)
                                 -> Result<String, String> {
    // TODO reuse template
    let mut reg = Handlebars::new();
    reg.register_helper("prec", Box::new(prec_helper));
    reg.register_helper("fixed", Box::new(fixed_helper));
    reg.register_helper("htmlf", Box::new(htmlf_helper));
    reg.register_template_string(&template.name, &template.body)
        .or_else(|e| Err(format!("{}", e)))?;
    reg.render(&template.name, &data)
        .or_else(|e| Err(format!("{}", e)))
}
