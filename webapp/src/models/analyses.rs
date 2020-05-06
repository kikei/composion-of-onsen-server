use std::collections::HashMap;
use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

use r2d2_mongodb::mongodb::{self, bson, Bson, doc, Document};
use r2d2_mongodb::mongodb::coll::Collection;
use r2d2_mongodb::mongodb::coll::options::{FindOptions};
use serde::Deserialize;

use crate::analysis::{Analysis, ComponentTable, CellValue, MgMvalMmol};
use crate::models::{Models, collection_analyses};
use crate::utils::mongodb::{document_str, document_number};
use crate::utils::scrub;

const KEY_ID: &str = "id";
const KEY_NAME: &str = "name";
const KEY_YIELD: &str = "yild";
const KEY_TEMPERATURE: &str = "temp";
const KEY_PH: &str = "ph";
const KEY_POSITIVE_ION: &str = "pion";
const KEY_NEGATIVE_ION: &str = "nion";
const KEY_UNDISSOCIATED: &str = "unds";
const KEY_GAS: &str = "gas";
const KEY_MINOR: &str = "mino";
const KEY_TOTAL_POSITIVE_ION: &str = "topi";
const KEY_TOTAL_NEGATIVE_ION: &str = "toni";
const KEY_TOTAL_UNDISSOCIATED: &str = "toun";
const KEY_TOTAL_GAS: &str = "toga";
const KEY_TOTAL_MINOR: &str = "tomi";
const KEY_TOTAL_MELT: &str = "tome";
const KEY_TOTAL: &str = "totl";
const KEY_MG: &str = "mg";
const KEY_MVAL: &str = "mv";
const KEY_MVAL_PERCENT: &str = "mvp";
const KEY_MMOL: &str = "mm";
const KEY_LAST_MODIFIED: &str = "_lamo";
const KEY_CREATED_AT: &str = "_crat";

/**
 * Conversions from MongoDB object to Analysis.
 */
impl TryFrom<&Bson> for CellValue {
    type Error = String;
    fn try_from(a: &Bson) -> Result<Self, Self::Error> {
        match a {
            Bson::FloatingPoint(a) => Ok(CellValue::Number(*a)),
            Bson::I32(a) => Ok(CellValue::Number(f64::from(*a))),
            Bson::I64(a) => Ok(CellValue::Number(*a as f64)),
            Bson::String(a) => Ok(CellValue::Text(a.to_string())),
            Bson::Null => Ok(CellValue::Null),
            _ => {
                info!("try_from(Bson)->CellValue, unexpected bson: {}", &a);
                Ok(CellValue::Null)
            }
        }
    }
}

impl TryFrom<&Document> for MgMvalMmol {
    type Error = String;
    fn try_from(d: &Document) -> Result<Self, Self::Error> {
        let mg = document_cell_value(d, KEY_MG)?;
        let mval = document_cell_value(d, KEY_MVAL)?;
        let mval_percent = document_cell_value(d, KEY_MVAL_PERCENT)?;
        let mmol = document_cell_value(d, KEY_MMOL)?;
        Ok(MgMvalMmol {
            mg: mg,
            mval: mval,
            mval_percent: mval_percent,
            mmol: mmol
        })
    }
}

impl TryFrom<&Document> for ComponentTable {
    type Error = String;
    fn try_from(item: &Document) -> Result<Self, Self::Error> {
        let mut components = HashMap::new();
        for key in item.keys() {
            let v = document_mg_mval_mmol(item, key.as_str())?;
            components.insert(key.to_string(), v);
        }
        Ok(ComponentTable {
            components: components
        })
    }
}

impl TryFrom<&Document> for Analysis {
    type Error = String;
    fn try_from(item: &Document) -> Result<Self, Self::Error> {
        let id = document_str(item, KEY_ID)
            .ok_or("BUG: The id field in analysis from Document is missed.
                    Perhaps it has been lost when it was stored to DB.")?;
        let name = document_str(item, KEY_NAME)
            .ok_or("BUG: missing name?")?;
        let gensen_yield = document_number(item, KEY_YIELD)
            .map_or(CellValue::Null, CellValue::Number);
        let temperature = document_number(item, KEY_TEMPERATURE);
        let ph = document_number(item, KEY_PH)
            .map_or(CellValue::Null, CellValue::Number);
        let positive_ion = document_component_table(item, KEY_POSITIVE_ION)?;
        let negative_ion = document_component_table(item, KEY_NEGATIVE_ION)?;
        let undissociated = document_component_table(item, KEY_UNDISSOCIATED)?;
        let gas = document_component_table(item, KEY_GAS)?;
        let minor = document_component_table(item, KEY_MINOR)?;
        // Total of components
        let total_positive_ion =
            document_mg_mval_mmol(item, KEY_TOTAL_POSITIVE_ION)?;
        let total_negative_ion =
            document_mg_mval_mmol(item, KEY_TOTAL_NEGATIVE_ION)?;
        let total_undissociated =
            document_mg_mval_mmol(item, KEY_TOTAL_UNDISSOCIATED)?;
        let total_gas =
            document_mg_mval_mmol(item, KEY_TOTAL_GAS)?;
        let total_minor =
            document_mg_mval_mmol(item, KEY_TOTAL_MINOR)?;
        let total_melt =
            document_mg_mval_mmol(item, KEY_TOTAL_MELT)?;
        let total =
            document_mg_mval_mmol(item, KEY_TOTAL)?;
        let mut meta = HashMap::new();
        for (key, value) in item {
            match key.as_str() {
                KEY_ID | KEY_NAME | KEY_YIELD | KEY_TEMPERATURE | KEY_PH | 
                KEY_POSITIVE_ION | KEY_NEGATIVE_ION |
                KEY_UNDISSOCIATED | KEY_GAS | KEY_MINOR |
                KEY_TOTAL_POSITIVE_ION | KEY_TOTAL_NEGATIVE_ION |
                KEY_TOTAL_UNDISSOCIATED | KEY_TOTAL_GAS | KEY_TOTAL_MINOR |
                KEY_TOTAL_MELT | KEY_TOTAL |
                KEY_LAST_MODIFIED | KEY_CREATED_AT => {},
                _ => {
                    meta.insert(key.to_string(), match value.as_str() {
                        Some(v) => v.to_string(),
                        None => value.to_string()
                    });
                }
            }
        }
        let last_modified =
            document_number(item, KEY_LAST_MODIFIED).map(|f| f as f64);
        let created_at =
            document_number(item, KEY_CREATED_AT).map(|f| f as f64);
        Ok(Analysis {
            id: Some(id),
            name: name,
            gensen_yield: gensen_yield,
            temperature: temperature,
            ph: ph,
            positive_ion: positive_ion,
            negative_ion: negative_ion,
            undissociated: undissociated,
            gas: gas,
            minor: minor,
            total_positive_ion: total_positive_ion,
            total_negative_ion: total_negative_ion,
            total_undissociated: total_undissociated,
            total_gas: total_gas,
            total_minor: total_minor,
            total_melt: total_melt,
            total: total,
            meta: meta,
            last_modified: last_modified,
            created_at: created_at
        })
    }
}

fn document_mg_mval_mmol(item: &Document, key: &str)
    -> Result<MgMvalMmol, String>
{
    item.get_document(key)
        .or(Ok(&Document::new())
            as Result<&Document, mongodb::ordered::ValueAccessError>)
        .map_err(|e| e.to_string())
        .and_then(MgMvalMmol::try_from)
}

fn document_cell_value(item: &Document, key: &str) -> Result<CellValue, String>
{
    match item.get(key) {
        Some(a) => CellValue::try_from(a),
        None => Ok(CellValue::Null)
    }
}

fn document_component_table(item: &Document, key: &str)
    -> Result<ComponentTable, String>
{
    item.get_document(key)
        .or(Ok(&Document::new())
            as Result<&Document, mongodb::ordered::ValueAccessError>)
        .map_err(|e| e.to_string())
        .and_then(ComponentTable::try_from)
}

/**
 * Conversions from Analysis to MongoDB object.
 */
impl From<&Analysis> for Document {
    fn from(item: &Analysis) -> Self {
        let mut d = doc! {
            KEY_ID: item.id.as_ref().map_or(Bson::Null, Bson::from),
            KEY_NAME: Bson::from(&item.name),
            KEY_YIELD: Bson::from(&item.gensen_yield),
            KEY_TEMPERATURE: item
                .temperature.map_or(Bson::Null, Bson::from),
            KEY_PH: Bson::from(&item.ph),
            KEY_POSITIVE_ION: Bson::from(&item.positive_ion),
            KEY_NEGATIVE_ION: Bson::from(&item.negative_ion),
            KEY_UNDISSOCIATED: Bson::from(&item.undissociated),
            KEY_GAS: Bson::from(&item.gas),
            KEY_MINOR: Bson::from(&item.minor),
            KEY_TOTAL_POSITIVE_ION:
            Bson::from(&item.total_positive_ion),
            KEY_TOTAL_NEGATIVE_ION:
            Bson::from(&item.total_negative_ion),
            KEY_TOTAL_UNDISSOCIATED:
            Bson::from(&item.total_undissociated),
            KEY_TOTAL_GAS:
            Bson::from(&item.total_gas),
            KEY_TOTAL_MINOR:
            Bson::from(&item.total_minor),
            KEY_TOTAL_MELT:
            Bson::from(&item.total_melt),
            KEY_TOTAL:
            Bson::from(&item.total),
            KEY_CREATED_AT: item
                .created_at.map_or(Bson::Null, Bson::from),
            KEY_LAST_MODIFIED: item
                .last_modified.map_or(Bson::Null, Bson::from)
        };
        for (key, value) in &item.meta {
            if !d.contains_key(&key) {
                d.insert(key.to_string(), Bson::String(value.to_string()));
            }
        }
        d
    }
}

impl From<&ComponentTable> for Bson {
    fn from(item: &ComponentTable) -> Self {
        let ComponentTable { components } = &item;
        let mut doc = doc! {};
        for (key, value) in components {
            println!("doc.insert({}, {})", key, Bson::from(value));
            doc.insert(key.to_string(), Bson::from(value));
        }
        Bson::Document(doc)
    }
}

/**
 * Conversions from CellValue to MongoDB object.
 */
impl From<&CellValue> for Bson {
    fn from(item: &CellValue) -> Self {
        match item {
            CellValue::Number(x) => Bson::FloatingPoint(*x),
            CellValue::Text(x) => Bson::String(x.to_string()),
            CellValue::Null => Bson::Null
        }
    }
}

impl From<CellValue> for Bson {
    fn from(item: CellValue) -> Self {
        Bson::from(&item)
    }
}

/**
 * Conversions from MgMvalMmol to MongoDB object.
 */
impl From<&MgMvalMmol> for Bson {
    fn from(item: &MgMvalMmol) -> Self {
        let doc = doc!{
            KEY_MG: item.mg.clone(),
            KEY_MVAL: item.mval.clone(),
            KEY_MVAL_PERCENT: item.mval_percent.clone(),
            KEY_MMOL: item.mmol.clone()
        };
        Bson::Document(doc)
    }
}

impl From<MgMvalMmol> for Bson {
    fn from(item: MgMvalMmol) -> Self {
        Bson::from(&item)
    }
}

/**
 * Operations for MongoDB.
 */
pub fn select(models: &Models) ->
    Result<impl Iterator<Item=Analysis>, String> {
        let coll: &Collection = collection_analyses(models);
        let result = coll.find(None, None);
        match result {
            Ok(cur) => Ok(cur.filter_map(|row| {
                let item: Document = row.ok()?;
                Analysis::try_from(&item).ok()
                // let a: Analysis = Analysis::from(&item);
                // Some(a)
            }).into_iter()),
            Err(e) => Err(String::from(format!("{}", e)))
        }
}

pub fn by_id(models: &Models, id: &String) -> Result<Option<Analysis>, String>
{
    debug!("analyses::by_id, id: {}", &id);
    let coll: &Collection = collection_analyses(models);
    let result = coll.find_one(Some(doc!{ KEY_ID: id }), None);
    debug!("analyses::by_id, result: {:?}", &result);
    match result {
        Ok(Some(row)) => Analysis::try_from(&row).map(|a| Some(a)),
        Ok(None) => Ok(None),
        Err(e) => Err(String::from(format!("{}", e)))
    }
}

pub fn save(models: &Models, a: &Analysis) -> Result<Analysis, String> {
    let coll = collection_analyses(models);
    let (id, is_new) = match &a.id {
        Some(id) => (id.to_string(), false),
        None => match create_unique_id(models, a) {
            Ok(id) => (id, true),
            Err(e) => return Err(e)
        }
    };
    // Clone analysis
    let mut a: Analysis = a.clone();
    // Update id
    a.id = Some(id.to_string());
    // Update created_at if needed
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or_else(|e| {
            warn!("Failed to calulate epoch!? {}", &e);
            0.0
        });
    a.created_at = a.created_at.or(Some(epoch));
    // Update last_modified
    a.last_modified = Some(epoch);
    // Write to DB
    let filter = doc!{ KEY_ID: &id };
    let d = Document::from(&a);
    debug!("analyses::save, is_new: {}, fileter: {:?}, d: {:?}",
           &is_new, &filter, &d);
    let result = if is_new {
        coll.insert_one(d, None)
            .map(|r| {
                debug!("analyses::save, created, result: {:?}", &r);
                Some(a)
            })
    } else {
        coll.replace_one(filter, d, None)
            .map(|r| {
                debug!("analyses::save, updated, result: {:?}", &r);
                if r.modified_count > 0 {
                    Some(a)
                } else {
                    None
                }
            })
    };
    match result {
        Ok(Some(a)) => Ok(a),
        Ok(None) => Err(String::from(format!("unexpected result in
                                             analyses::save"))),
        Err(e) => Err(String::from(format!("{}", e)))
    }
}

const MAX_ID_SERIAL: usize = 99;

pub fn create_unique_id(models: &Models, a: &Analysis)
                        -> Result<String, String> {
    let coll: &Collection = collection_analyses(models);
    let base = new_id(a);
    let mut i = 0;
    loop {
        if i > MAX_ID_SERIAL {
            break Err(format!("Cannot create unique id, base: {}, i: {}",
                              &base, &i));
        }
        let tryid = if i == 0 {
            base.clone()
        } else {
            format!("{}-{}", &base, i)
        };
        match coll.find_one(Some(doc!{ KEY_ID: &tryid }), None) {
            Ok(Some(_)) => i += 1,
            Ok(None) => break Ok(tryid),
            Err(e) => break Err(format!("Cannot create uniquid id, e: {}", e))
        }
    }
}

fn new_id(a: &Analysis) -> String {
    scrub::scrub(&a.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_from_mg_mval_mmol_empty() {
        let it = doc! {};
        let r = MgMvalMmol::try_from(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), MgMvalMmol {
            mg: CellValue::Null,
            mval: CellValue::Null,
            mmol: CellValue::Null,
            mval_percent: CellValue::Null
        });
    }

    #[test]
    fn test_document_from_mg_mval_mmol_with_null() {
        let it = doc! {
            KEY_MG: Bson::Null,
            KEY_MVAL: Bson::Null,
            KEY_MMOL: Bson::Null,
            KEY_MVAL_PERCENT: Bson::Null
        };
        let r = MgMvalMmol::try_from(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), MgMvalMmol {
            mg: CellValue::Null,
            mval: CellValue::Null,
            mmol: CellValue::Null,
            mval_percent: CellValue::Null
        });
    }

    #[test]
    fn test_document_from_mg_mval_mmol_with_number() {
        let it = doc! {
            KEY_MG: 4078.0,
            KEY_MVAL: 177.4,
            KEY_MMOL: 177.4,
            KEY_MVAL_PERCENT: 87.65
        };
        let r = MgMvalMmol::try_from(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), MgMvalMmol {
            mg: CellValue::Number(4078.0),
            mval: CellValue::Number(177.4),
            mmol: CellValue::Number(177.4),
            mval_percent: CellValue::Number(87.65)
        });
    }

    #[test]
    fn test_document_from_mg_mval_mmol_with_text() {
        let it = doc! {
            KEY_MG: ">0.01",
            KEY_MVAL: ">0.01",
            KEY_MMOL: "--",
            KEY_MVAL_PERCENT: ""
        };
        let r = MgMvalMmol::try_from(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), MgMvalMmol {
            mg: CellValue::Text(">0.01".to_string()),
            mval: CellValue::Text(">0.01".to_string()),
            mmol: CellValue::Text("--".to_string()),
            mval_percent: CellValue::Text("".to_string())
        });
    }

    #[test]
    fn test_document_from_analysis() {
        let mut meta = HashMap::new();
        meta.insert("quality".to_string(),
                    "含硫黄－ナトリウム－塩化物温泉 (硫化水素型)".to_string());
        let it = Analysis {
            id: Some("amenakaonsen".to_string()),
            name: "雨中温泉".to_string(),
            gensen_yield: CellValue::Number(1220.0),
            temperature: Some(48.8),
            ph: CellValue::Number(7.5),
            positive_ion: ComponentTable::new(),
            negative_ion: ComponentTable::new(),
            undissociated: ComponentTable::new(),
            gas: ComponentTable::new(),
            minor: ComponentTable::new(),
            total_positive_ion: MgMvalMmol {
                mg: CellValue::Number(4583.0),
                mval: CellValue::Number(202.3),
                mmol: CellValue::Text("--".to_string()),
                mval_percent: CellValue::Null
            },
            total_negative_ion: MgMvalMmol {
                mg: CellValue::Number(7881.0),
                mval: CellValue::Number(216.6),
                mmol: CellValue::Text("--".to_string()),
                mval_percent: CellValue::Null
            },
            total_undissociated: MgMvalMmol {
                mg: CellValue::Number(164.3),
                mval: CellValue::Number(0.0),
                mmol: CellValue::Number(2.63),
                mval_percent: CellValue::Null,
            },
            total_gas: MgMvalMmol {
                mg: CellValue::Number(29.0),
                mval: CellValue::Number(0.0),
                mmol: CellValue::Number(0.7),
                mval_percent: CellValue::Null
            },
            total_minor: MgMvalMmol {
                mg: CellValue::Null,
                mval: CellValue::Null,
                mmol: CellValue::Null,
                mval_percent: CellValue::Null
            },
            meta: meta
        };
        let r = Document::try_from(&it);
        if !r.is_ok() {
            println!("Error {:?}", &r);
        }
        assert!(r.is_ok());
        let d = doc! {
            KEY_ID: "amenakaonsen",
            KEY_NAME: "雨中温泉",
            KEY_YIELD: 1220.0,
            KEY_TEMPERATURE: 48.8,
            KEY_PH: 7.5,
            KEY_POSITIVE_ION: {},
            KEY_NEGATIVE_ION: {},
            KEY_UNDISSOCIATED: {},
            KEY_GAS: {},
            KEY_MINOR: {},
            KEY_TOTAL_POSITIVE_ION: {
                KEY_MG: 4583.0,
                KEY_MVAL: 202.3,
                KEY_MVAL_PERCENT: Bson::Null,
                KEY_MMOL: "--"
            },
            KEY_TOTAL_NEGATIVE_ION: {
                KEY_MG: 7881.0,
                KEY_MVAL: 216.6,
                KEY_MVAL_PERCENT: Bson::Null,
                KEY_MMOL: "--"
            },
            KEY_TOTAL_UNDISSOCIATED: {
                KEY_MG: 164.3,
                KEY_MVAL: 0.0,
                KEY_MVAL_PERCENT: Bson::Null,
                KEY_MMOL: 2.63
            },
            KEY_TOTAL_GAS: {
                KEY_MG: 29.0,
                KEY_MVAL: 0.0,
                KEY_MVAL_PERCENT: Bson::Null,
                KEY_MMOL: 0.7
            },
            KEY_TOTAL_MINOR: {
                KEY_MG: Bson::Null,
                KEY_MVAL: Bson::Null,
                KEY_MVAL_PERCENT: Bson::Null,
                KEY_MMOL: Bson::Null
            },
            "quality": "含硫黄－ナトリウム－塩化物温泉 (硫化水素型)"
        };
        assert_eq!(r.unwrap(), d);
    }

    #[test]
    fn test_analysis_from_document() {
        let it = doc! {
            KEY_ID: "amenakaonsen",
            KEY_POSITIVE_ION: {},
            KEY_NEGATIVE_ION: {},
            KEY_UNDISSOCIATED: {},
            KEY_GAS: {},
            KEY_MINOR: {},
            KEY_TOTAL_POSITIVE_ION: {
                KEY_MG: 4583.0,
                KEY_MVAL: 202.3,
                KEY_MMOL: "--"
            },
            KEY_TOTAL_NEGATIVE_ION: {
                KEY_MG: 7881.0,
                KEY_MVAL: 216.6,
                KEY_MMOL: "--"
            },
            KEY_TOTAL_UNDISSOCIATED: {
                KEY_MG: 164.3,
                KEY_MVAL: 0.0,
                KEY_MMOL: 2.63
            },
            KEY_TOTAL_GAS: {
                KEY_MG: 29.0,
                KEY_MVAL: 0.0,
                KEY_MMOL: 0.7
            },
            KEY_NAME: "雨中温泉",
            KEY_PH: 7.5,
            KEY_YIELD: 1220.0,
            KEY_TEMPERATURE: 48.8,
            "quality": "含硫黄－ナトリウム－塩化物温泉 (硫化水素型)"
        };
        let r = Analysis::try_from(&it);
        if !r.is_ok() {
            println!("Error {:?}", &r);
        }
        assert!(r.is_ok());
        let mut meta = HashMap::new();
        meta.insert("quality".to_string(),
                    "含硫黄－ナトリウム－塩化物温泉 (硫化水素型)".to_string());
        assert_eq!(r.unwrap(), Analysis {
            id: Some("amenakaonsen".to_string()),
            name: "雨中温泉".to_string(),
            gensen_yield: CellValue::Number(1220.0),
            temperature: Some(48.8),
            ph: CellValue::Number(7.5),
            positive_ion: ComponentTable::new(),
            negative_ion: ComponentTable::new(),
            undissociated: ComponentTable::new(),
            gas: ComponentTable::new(),
            minor: ComponentTable::new(),
            total_positive_ion: MgMvalMmol {
                mg: CellValue::Number(4583.0),
                mval: CellValue::Number(202.3),
                mmol: CellValue::Text("--".to_string()),
                mval_percent: CellValue::Null
            },
            total_negative_ion: MgMvalMmol {
                mg: CellValue::Number(7881.0),
                mval: CellValue::Number(216.6),
                mmol: CellValue::Text("--".to_string()),
                mval_percent: CellValue::Null
            },
            total_undissociated: MgMvalMmol {
                mg: CellValue::Number(164.3),
                mval: CellValue::Number(0.0),
                mmol: CellValue::Number(2.63),
                mval_percent: CellValue::Null,
            },
            total_gas: MgMvalMmol {
                mg: CellValue::Number(29.0),
                mval: CellValue::Number(0.0),
                mmol: CellValue::Number(0.7),
                mval_percent: CellValue::Null
            },
            total_minor: MgMvalMmol {
                mg: CellValue::Null,
                mval: CellValue::Null,
                mmol: CellValue::Null,
                mval_percent: CellValue::Null
            },
            meta: meta
        });
    }
}
