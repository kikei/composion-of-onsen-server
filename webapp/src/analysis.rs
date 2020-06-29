use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::template::{Template, Render, render_template};

/**
 * Resources:
 * - Serde https://serde.rs/
 */

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum CellValue {
    Number(f64),
    Text(String),
    Null
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MgMvalMmol {
    pub mg: CellValue,
    pub mval: CellValue,
    pub mval_percent: CellValue,
    pub mmol: CellValue
}

impl MgMvalMmol {
    pub fn new() -> Self {
        MgMvalMmol {
            mg: CellValue::Null,
            mval: CellValue::Null,
            mval_percent: CellValue::Null,
            mmol: CellValue::Null
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
pub struct ComponentTable {
    #[serde(flatten)]
    pub components: HashMap<String, MgMvalMmol>
}

impl ComponentTable {
    pub fn new() -> Self {
        ComponentTable {
            components: HashMap::new()
        }
    }
}

impl Default for ComponentTable {
    fn default() -> Self {
        debug!("default()->ComponentTable");
        ComponentTable::new()
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
pub struct Analysis {
    pub id: Option<String>,                   // ID. None for new analysis.

    #[serde(rename = "name")]

    pub name: String,                         // 源泉名

    #[serde(rename = "yield")]
    pub gensen_yield: CellValue,              // 湧出量

    pub temperature: Option<f64>,             // 泉温

    #[serde(rename = "pH")]
    pub ph: CellValue,                        // 湧出地におけるpH値

    #[serde(rename = "positiveIon", default)]
    pub positive_ion: ComponentTable,         // 陽イオン計

    #[serde(rename = "negativeIon", default)]
    pub negative_ion: ComponentTable,         // 陽イオン計

    #[serde(default)]
    pub undissociated: ComponentTable,        // 非解離成分

    #[serde(default)]
    pub gas: ComponentTable,                  // 溶存ガス

    #[serde(default)]
    pub minor: ComponentTable,                // 微量成分

    #[serde(rename = "totalPositiveIon")]
    pub total_positive_ion: MgMvalMmol,       // 陽イオン計

    #[serde(rename = "totalNegativeIon")]
    pub total_negative_ion: MgMvalMmol,       // 陽イオン計

    #[serde(rename = "totalUndissociated")]
    pub total_undissociated: MgMvalMmol,      // 非解離成分計

    #[serde(rename = "totalGas")]
    pub total_gas: MgMvalMmol,                // 溶存ガス計

    #[serde(rename = "totalMinor")]
    pub total_minor: MgMvalMmol,              // 微量成分計

    #[serde(rename = "totalMelt")]
    pub total_melt: MgMvalMmol,               // 溶存物質量 (ガス性のものを除く)

    #[serde(rename = "total")]
    pub total: MgMvalMmol,                    // 成分総計

    #[serde(flatten)]
    pub meta: HashMap<String, String>,

    #[serde(rename = "lastModified")]
    pub last_modified: Option<f64>,          // Timestamp [ms]

    #[serde(rename = "createdAt")]
    pub created_at: Option<f64>              // Timestamp [ms]
}

/**
 * Conversions from JSON to MgMvalMmol.
 */
// impl TryFrom<&Value> for MgMvalMmol {
//     type Error = &'static str;
//     fn try_from(v: &Value) -> Result<Self, Self::Error> {
//         Ok(MgMvalMmol {
//             mg: json_number(v, KEY_MG)
//                 .map_or(CellValue::Null, CellValue::Number),
//             mval: json_number(v, KEY_MVAL)
//                 .map_or(CellValue::Null, CellValue::Number),
//             mval_percent: json_number(v, KEY_MVAL_PERCENT)
//                 .map_or(CellValue::Null, CellValue::Number),
//             mmol: json_number(v, KEY_MMOL)
//                 .map_or(CellValue::Null, CellValue::Number)
//         })
//     }
// }
// 
// impl TryFrom<Value> for MgMvalMmol {
//     type Error = &'static str;
//     fn try_from(v: Value) -> Result<Self, Self::Error> {
//         MgMvalMmol::try_from(&v)
//     }
// }

/**
 * Conversions from JSON to CellValue.
 */
// impl TryFrom<&Value> for CellValue {
//     type Error = &'static str;
//     fn try_from(v: &Value) -> Result<Self, Self::Error> {
//         match v {
//             Value::Number(x) => x.as_f64()
//                 .map_or(Err("Number cast error"),
//                         |x| Ok(CellValue::Number(x))),
//             Value::String(x) => Ok(CellValue::Text(x.to_string())),
//             Value::Null => Ok(CellValue::Null),
//             _ => Err("Unsupported cell value")
//         }
//     }
// }

// impl TryFrom<&Value> for ComponentTable {
//     type Error = &'static str;
//     fn try_from(v: &Value) -> Result<Self, Self::Error> {
//         let mut table = HashMap::new();
//         match v.as_object() {
//             Some(m) => {
//                 for (key, value) in m {
//                     if let Ok(cv) = MgMvalMmol::try_from(value) {
//                         &table.insert(key.to_string(), cv);
//                     }
//                 }
//                 Ok(ComponentTable { components: table })
//             },
//             None => Err("Unsupported component table")
//         }
//     }
// }

// impl TryFrom<Value> for Analysis {
//     type Error = &'static str;
//     fn try_from(v: Value) -> Result<Self, Self::Error> {
//         Analysis::try_from(&v)
//     }
// }

/**
 * Conversions from JSON object to Analysis.
 */
// impl TryFrom<&Value> for Analysis {
//     type Error = &'static str;
//     fn try_from(v: &Value) -> Result<Self, Self::Error> {
//         let id = json_str(v, KEY_ID);
//         let name = json_str(v, KEY_GENSEN_NAME)
//             .ok_or("Gensen name must be set")?;
//         let gensen_yield = json_option(v, KEY_YIELD).unwrap_or(CellValue::Null);
//         let temperature = json_number(v, KEY_TEMPERATURE);
//         let ph = json_option(v, KEY_PH).unwrap_or(CellValue::Null);
//         // Component
//         let positive_ion =
//             json_option(v, KEY_POSITIVE_ION)
//             .unwrap_or(ComponentTable::new());
//         let negative_ion =
//             json_option(v, KEY_NEGATIVE_ION)
//             .unwrap_or(ComponentTable::new());
//         let undissociated =
//             json_option(v, KEY_UNDISSOCIATED)
//             .unwrap_or(ComponentTable::new());
//         let gas =
//             json_option(v, KEY_GAS)
//             .unwrap_or(ComponentTable::new());
//         let minor =
//             json_option(v, KEY_MINOR)
//             .unwrap_or(ComponentTable::new());
//         // Total component
//         let total_positive_ion =
//             json_option(v, KEY_TOTAL_POSITIVE_ION)
//             .unwrap_or(MgMvalMmol::new());
//         let total_negative_ion =
//             json_option(v, KEY_TOTAL_NEGATIVE_ION)
//             .unwrap_or(MgMvalMmol::new());
//         let total_undissociated =
//             json_option(v, KEY_TOTAL_UNDISSOCIATED)
//             .unwrap_or(MgMvalMmol::new());
//         let total_gas =
//             json_option(v, KEY_TOTAL_GAS)
//             .unwrap_or(MgMvalMmol::new());
//         let total_minor =
//             json_option(v, KEY_TOTAL_MINOR)
//             .unwrap_or(MgMvalMmol::new());
//         let mut meta = HashMap::new();
//         if let Some(m) = v.as_object() {
//             for (key, value) in m {
//                 match value.as_str() {
//                     Some(t) => meta.insert(key.to_string(), String::from(t)),
//                     None => meta.insert(key.to_string(), value.to_string())
//                 };
//             }
//         }
//         Ok(Analysis {
//             id: id,
//             name: name,
//             gensen_yield: gensen_yield,
//             temperature: temperature,
//             ph: ph,
//             positive_ion: positive_ion,
//             negative_ion: negative_ion,
//             undissociated: undissociated,
//             gas: gas,
//             minor: minor,
//             total_positive_ion: total_positive_ion,
//             total_negative_ion: total_negative_ion,
//             total_undissociated: total_undissociated,
//             total_gas: total_gas,
//             total_minor: total_minor,
//             meta: meta
//         })
//     }
// }

// fn json_str(value: &Value, key: &str) -> Option<String> {
//     value.get(key).and_then(|v| v.as_str()).map(String::from)
// }
// 
// fn json_number(value: &Value, key: &str) -> Option<f64> {
//     value.get(key).and_then(|v| v.as_f64())
// }
// 
// fn json_option<'a, T: TryFrom<&'a Value>>(value: &'a Value, key: &str)
//                                           -> Option<T> {
//     value.get(key).and_then(|v| T::try_from(v).ok())
// }

// impl Serialize for Analysis {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where S: Serializer
//     {
//         let mut s = serializer.serialize_map(Some(1))?;
//         // Insert earlier than other records to be overwritten
//         for (key, value) in &self.meta {
//             s.serialize_entry(key, value)?;
//         }
//         if let Some(t) = &self.id {
//             s.serialize_entry(KEY_ID, t)?;
//         }
//         s.serialize_entry(KEY_GENSEN_NAME, &self.name)?;
//         if let Some(t) = &self.temperature {
//             s.serialize_entry(KEY_TEMPERATURE, t)?;
//         }
//         s.serialize_entry(KEY_PH, &self.ph)?;
//         s.serialize_entry(KEY_POSITIVE_ION, &self.positive_ion)?;
//         s.serialize_entry(KEY_NEGATIVE_ION, &self.negative_ion)?;
//         s.serialize_entry(KEY_UNDISSOCIATED, &self.undissociated)?;
//         s.serialize_entry(KEY_GAS, &self.gas)?;
//         s.serialize_entry(KEY_MINOR, &self.minor)?;
//         s.serialize_entry(KEY_TOTAL_POSITIVE_ION, &self.total_positive_ion)?;
//         s.serialize_entry(KEY_TOTAL_NEGATIVE_ION, &self.total_negative_ion)?;
//         s.serialize_entry(KEY_TOTAL_UNDISSOCIATED, &self.total_undissociated)?;
//         s.serialize_entry(KEY_TOTAL_GAS, &self.total_gas)?;
//         s.serialize_entry(KEY_TOTAL_MINOR, &self.total_minor)?;
//         s.end()
//     }
// }

/**
 * Deserializing into Analysis
 */
// struct AnalysisVisitor {
//     marker: PhantomData<fn () -> Analysis>
// }
// 
// impl AnalysisVisitor {
//     fn new() -> Self {
//         AnalysisVisitor {
//             marker: PhantomData
//         }
//     }
// }
// 
// impl<'de> Visitor<'de> for AnalysisVisitor {
//     type Value = Analysis;
// 
//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("analysis")
//     }
// 
//     fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
//     where
//         M: MapAccess<'de>,
//     {
//         let mut a = Analysis {
//             id: None,
//             name: "".to_string(),
//             temperature: None,
//             ph: CellValue::Null,
//             positive_ion: ComponentTable(HashMap::new()),
//             negative_ion: ComponentTable(HashMap::new()),
//             undissociated: ComponentTable(HashMap::new()),
//             gas: ComponentTable(HashMap::new()),
//             minor: ComponentTable(HashMap::new()),
//             total_positive_ion: MgMvalMmol::new(),
//             total_negative_ion: MgMvalMmol::new(),
//             total_undissociated: MgMvalMmol::new(),
//             total_gas: MgMvalMmol::new(),
//             total_minor: MgMvalMmol::new(),
//             meta: HashMap::new()
//         };
//         while let Some(key) = access.next_key()? {
//             println!("AnavisisVisitor visit_map, key: {}", &key);
//             match key { // -> Result<(), MapEntry::Error)> 
//                 KEY_ID =>
//                     Ok(a.id = access.next_value::<String>().ok()),
//                 KEY_GENSEN_NAME =>
//                     access.next_value::<String>().map(|x| a.name = x),
//                 KEY_TEMPERATURE =>
//                     Ok(a.temperature = access.next_value::<f64>().ok()),
//                 KEY_PH =>
//                     access.next_value::<CellValue>().map(|x| a.ph = x),
//                 KEY_POSITIVE_ION =>
//                     access.next_value::<Value>()
//                     .and_then(|x| match x.as_object() {
//                         Some(x) => Ok(a.positive_ion = ComponentTable(x)),
//                         None => Err(de::Error::missing_field("TODO"))
//                     }),
//                     /*
//                     access.next_value::<ComponentTable>()
//                         .map(|x| a.positive_ion = x),
//                     */
//                 KEY_NEGATIVE_ION =>
//                     access.next_value::<ComponentTable>()
//                         .map(|x| a.negative_ion = x),
//                 KEY_UNDISSOCIATED =>
//                     access.next_value::<ComponentTable>()
//                         .map(|x| a.undissociated = x),
//                 KEY_GAS =>
//                     access.next_value::<ComponentTable>()
//                         .map(|x| a.gas = x),
//                 KEY_MINOR =>
//                     access.next_value::<ComponentTable>()
//                         .map(|x| a.minor = x),
//                 KEY_TOTAL_POSITIVE_ION =>
//                     access.next_value::<MgMvalMmol>()
//                         .map(|x| a.total_positive_ion = x),
//                 KEY_TOTAL_NEGATIVE_ION =>
//                     access.next_value::<MgMvalMmol>()
//                         .map(|x| a.total_negative_ion = x),
//                 KEY_TOTAL_UNDISSOCIATED =>
//                     access.next_value::<MgMvalMmol>()
//                         .map(|x| a.total_undissociated = x),
//                 KEY_TOTAL_GAS =>
//                     access.next_value::<MgMvalMmol>()
//                         .map(|x| a.total_gas = x),
//                 KEY_TOTAL_MINOR =>
//                     access.next_value::<MgMvalMmol>()
//                         .map(|x| a.total_minor = x),
//                 _ =>
//                     access.next_value::<String>()
//                         .map(|x| { a.meta.insert(key.to_string(), x); })
//             }?
//         };
//         Ok(a)
//     }
// }

// Resource:
// Implement Deserialize for custom map type
// https://serde.rs/deserialize-map.html
// impl<'de> Deserialize<'de> for Analysis {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//         where D: Deserializer<'de>
//     {
//         deserializer.deserialize_map(AnalysisVisitor::new())
//     }
// }

/**
 * Rendering with template
 */
impl Render for Analysis {
    fn render(&self, t: &Template) -> Result<String, String> {
        match render_template(self, t) {
            Ok(t) => Ok(t),
            Err(e) => Err(format!("Template error, {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serialize_cellvalue_number() {
        let it = CellValue::Number(63.8);
        let r = serde_json::to_string(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), "63.8");
    }

    #[test]
    fn it_deserialize_cellvalue_number() {
        let it = "63.8";
        let r = serde_json::from_str::<CellValue>(it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), CellValue::Number(63.8));
    }

    #[test]
    fn it_serialize_cellvalue_text() {
        let it = CellValue::Text("微量".to_string());
        let r = serde_json::to_string(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), r#""微量""#);
    }

    #[test]
    fn it_deserialize_cellvalue_text() {
        let it = r#""微量""#;
        let r = serde_json::from_str::<CellValue>(it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), CellValue::Text("微量".to_string()));
    }

    #[test]
    fn it_serialize_cellvalue_null() {
        let it = CellValue::Null;
        let r = serde_json::to_string(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), "null");
    }

    #[test]
    fn it_deserialize_cellvalue_null() {
        let it = "null";
        let r = serde_json::from_str::<CellValue>(it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), CellValue::Null);
    }

    #[test]
    fn it_serialize_mgmvalmmol() {
        let it = MgMvalMmol {
            mg: CellValue::Number(484.),
            mval: CellValue::Number(21.1),
            mval_percent: CellValue::Text("微量".to_string()),
            mmol: CellValue::Null
        };
        let r = serde_json::to_string(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(),
                   r#"{"mg":484.0,"mval":21.1,"mvalPercent":"微量","mmol":null}"#);
    }

    #[test]
    fn it_deserialize_mgmvalmmol() {
        let it = r#"{"mg":484.0,"mval":21.1,"mvalPercent":"微量","mmol":null}"#;
        let r = serde_json::from_str::<MgMvalMmol>(it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(),
                   MgMvalMmol {
                       mg: CellValue::Number(484.),
                       mval: CellValue::Number(21.1),
                       mval_percent: CellValue::Text("微量".to_string()),
                       mmol: CellValue::Null
                   });
    }

    #[test]
    fn it_serialize_empty_componenttable() {
        let it = ComponentTable::new();
        let r = serde_json::to_string(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), "{}");
    }

    #[test]
    fn it_deserialize_empty_componenttable() {
        let it = "{}";
        let r = serde_json::from_str::<ComponentTable>(it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), ComponentTable::new());
    }

    #[test]
    fn it_serialize_componenttable() {
        let mut it = HashMap::new();
        it.insert("Na".to_string(), MgMvalMmol {
            mg: CellValue::Number(484.),
            mval: CellValue::Number(21.1),
            mval_percent: CellValue::Number(97.01),
            mmol: CellValue::Text("".to_string())
        });
        let it = ComponentTable { components: it };
        let r = serde_json::to_string(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(),
                   r#"{"Na":{"mg":484.0,"mval":21.1,"mvalPercent":97.01,"mmol":""}}"#);
    }

    #[test]
    fn it_deserialize_componenttable() {
        let it = r#"{
            "Na": {
                "mg": 484.0, "mval": 21.1, "mvalPercent": 97.01, "mmol": ""
            }
        }"#;
        let r = serde_json::from_str::<ComponentTable>(it);
        assert!(r.is_ok());

        let mut components = HashMap::new();
        components.insert("Na".to_string(), MgMvalMmol {
            mg: CellValue::Number(484.),
            mval: CellValue::Number(21.1),
            mval_percent: CellValue::Number(97.01),
            mmol: CellValue::Text("".to_string())
        });
        let component_table = ComponentTable { components: components };
        assert_eq!(r.unwrap(), component_table);
    }

    #[test]
    fn it_serialize_analysis() {
        let mut positive_ion = HashMap::new();
        positive_ion.insert("Na".to_string(), MgMvalMmol {
            mg: CellValue::Number(484.),
            mval: CellValue::Number(21.1),
            mval_percent: CellValue::Number(97.01),
            mmol: CellValue::Text("".to_string())
        });
        let total_positive_ion = MgMvalMmol {
            mg: CellValue::Number(484.),
            mval: CellValue::Number(21.1),
            mval_percent: CellValue::Null,
            mmol: CellValue::Null
        };
        let mut negative_ion = HashMap::new();
        negative_ion.insert("Cl".to_string(), MgMvalMmol {
            mg: CellValue::Number(152.0),
            mval: CellValue::Number(4.29),
            mval_percent: CellValue::Number(18.76),
            mmol: CellValue::Null
        });
        let total_negative_ion = MgMvalMmol {
            mg: CellValue::Number(152.0),
            mval: CellValue::Number(4.29),
            mval_percent: CellValue::Null,
            mmol: CellValue::Null
        };
        let mut meta = HashMap::new();
        meta.insert("quality".to_string(),
                    "含硫黄ーナトリウムー塩化物泉".to_string());
        let it = Analysis {
            id: None,
            name: "雨中温泉".to_string(),
            gensen_yield: CellValue::Number(1080.0),
            temperature: Some(44.6),
            ph: CellValue::Number(7.9),
            positive_ion: ComponentTable { components: positive_ion },
            negative_ion: ComponentTable { components: negative_ion },
            undissociated: ComponentTable::new(),
            gas: ComponentTable::new(),
            minor: ComponentTable::new(),
            total_positive_ion: total_positive_ion,
            total_negative_ion: total_negative_ion,
            total_undissociated: MgMvalMmol::new(),
            total_gas: MgMvalMmol::new(),
            total_minor: MgMvalMmol::new(),
            meta: meta
        };
        let r = serde_json::to_string(&it);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(),
                   r#"{"id":null,"name":"雨中温泉","yield":1080.0,"temperature":44.6,"pH":7.9,"positiveIon":{"Na":{"mg":484.0,"mval":21.1,"mvalPercent":97.01,"mmol":""}},"negativeIon":{"Cl":{"mg":152.0,"mval":4.29,"mvalPercent":18.76,"mmol":null}},"undissociated":{},"gas":{},"minor":{},"totalPositiveIon":{"mg":484.0,"mval":21.1,"mvalPercent":null,"mmol":null},"totalNegativeIon":{"mg":152.0,"mval":4.29,"mvalPercent":null,"mmol":null},"totalUndissociated":{"mg":null,"mval":null,"mvalPercent":null,"mmol":null},"totalGas":{"mg":null,"mval":null,"mvalPercent":null,"mmol":null},"totalMinor":{"mg":null,"mval":null,"mvalPercent":null,"mmol":null},"quality":"含硫黄ーナトリウムー塩化物泉"}"#);
    }

    #[test]
    fn it_deserialize_analysis() {
        let it = r#"{
            "id": null,
            "name": "雨中温泉",
            "yield": 1080.0,
            "temperature": 44.6,
            "pH": 7.9,
            "positiveIon": {
                "Na": {
                    "mg": 484.0,  "mval": 21.1,
                    "mvalPercent": 97.01,  "mmol": ""
                }
            },
            "negativeIon": {
                "Cl": {
                    "mg": 152.0, "mval": 4.29,
                    "mvalPercent": 18.76, "mmol": null
                }
            },
            "undissociated": {},
            "gas": {},
            "minor": {},
            "totalPositiveIon": {
                "mg": 484.0, "mval": 21.1, "mvalPercent": null, "mmol": null
            },
            "totalNegativeIon": {
                "mg": 152.0,"mval": 4.29,"mvalPercent": null,"mmol": null
            },
            "totalUndissociated": {
                "mg": null,"mval": null,"mvalPercent": null,"mmol": null
            },
            "totalGas": {
                "mg": null,"mval": null,"mvalPercent": null,"mmol": null
            },
            "totalMinor": {
                "mg": null,"mval": null,"mvalPercent": null,"mmol": null
            },
            "quality": "含硫黄ーナトリウムー塩化物泉"
        }"#;
        let r = serde_json::from_str::<Analysis>(it);
        assert!(r.is_ok());

        let mut positive_ion = HashMap::new();
        positive_ion.insert("Na".to_string(), MgMvalMmol {
            mg: CellValue::Number(484.),
            mval: CellValue::Number(21.1),
            mval_percent: CellValue::Number(97.01),
            mmol: CellValue::Text("".to_string())
        });
        let total_positive_ion = MgMvalMmol {
            mg: CellValue::Number(484.),
            mval: CellValue::Number(21.1),
            mval_percent: CellValue::Null,
            mmol: CellValue::Null
        };
        let mut negative_ion = HashMap::new();
        negative_ion.insert("Cl".to_string(), MgMvalMmol {
            mg: CellValue::Number(152.0),
            mval: CellValue::Number(4.29),
            mval_percent: CellValue::Number(18.76),
            mmol: CellValue::Null
        });
        let total_negative_ion = MgMvalMmol {
            mg: CellValue::Number(152.0),
            mval: CellValue::Number(4.29),
            mval_percent: CellValue::Null,
            mmol: CellValue::Null
        };
        let mut meta = HashMap::new();
        meta.insert("quality".to_string(),
                    "含硫黄ーナトリウムー塩化物泉".to_string());
        let analysis = Analysis {
            id: None,
            name: "雨中温泉".to_string(),
            gensen_yield: CellValue::Number(1080.0),
            temperature: Some(44.6),
            ph: CellValue::Number(7.9),
            positive_ion: ComponentTable { components: positive_ion },
            negative_ion: ComponentTable { components: negative_ion },
            undissociated: ComponentTable::new(),
            gas: ComponentTable::new(),
            minor: ComponentTable::new(),
            total_positive_ion: total_positive_ion,
            total_negative_ion: total_negative_ion,
            total_undissociated: MgMvalMmol::new(),
            total_gas: MgMvalMmol::new(),
            total_minor: MgMvalMmol::new(),
            meta: meta
        };
        assert_eq!(r.unwrap(), analysis);
    }

}
