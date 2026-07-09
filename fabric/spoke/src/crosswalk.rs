//! Declarative crosswalk loaders (port of crosswalk.py). Mapping knowledge lives in
//! data/type_crosswalk.yaml + data/predicate_crosswalk.yaml; this module only interprets it.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config;

#[derive(Deserialize, Default)]
struct TypeSpec {
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    subtype_routing: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Default)]
struct PrefSpec {
    #[serde(default)]
    prefer_props: HashMap<String, String>,
}

#[derive(Deserialize, Default)]
struct TypeCrosswalkFile {
    #[serde(default)]
    types: HashMap<String, TypeSpec>,
    #[serde(default)]
    xref_to_identifier: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    label_preferences: HashMap<String, PrefSpec>,
    #[serde(default)]
    name_aliases: HashMap<String, HashMap<String, String>>,
}

pub struct TypeCrosswalk {
    types: HashMap<String, TypeSpec>,
    xref_map: HashMap<String, HashMap<String, String>>,
    preferences: HashMap<String, PrefSpec>,
    // {label: {name (verbatim + lowercased): canonical_spoke_name}}
    name_alias_map: HashMap<String, HashMap<String, String>>,
}

impl TypeCrosswalk {
    pub fn load() -> Result<Self> {
        Self::load_path(&config::data_path("type_crosswalk.yaml"))
    }

    pub fn load_path(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let data: TypeCrosswalkFile =
            serde_yaml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        let mut name_alias_map: HashMap<String, HashMap<String, String>> = HashMap::new();
        for (lab, m) in &data.name_aliases {
            let entry = name_alias_map.entry(lab.clone()).or_default();
            for (k, v) in m {
                entry.insert(k.clone(), v.clone());
                entry.insert(k.to_lowercase(), v.clone());
            }
        }
        Ok(TypeCrosswalk {
            types: data.types,
            xref_map: data.xref_to_identifier,
            preferences: data.label_preferences,
            name_alias_map,
        })
    }

    pub fn type_keys(&self) -> HashSet<String> {
        self.types.keys().cloned().collect()
    }

    /// Extra property equalities to disambiguate duplicate records for `label`, or None.
    pub fn label_preference(&self, label: &str) -> Option<HashMap<String, String>> {
        self.preferences
            .get(label)
            .map(|p| p.prefer_props.clone())
            .filter(|m| !m.is_empty())
    }

    /// Canonical SPOKE name for a BioOKF name that differs (taxonomic reclassification), or None.
    pub fn name_alias(&self, label: &str, name: &str) -> Option<String> {
        let m = self.name_alias_map.get(label)?;
        m.get(name).or_else(|| m.get(&name.to_lowercase())).cloned()
    }

    /// Ordered candidate SPOKE labels for a BioOKF (type, subtype). [] => not_mappable.
    pub fn candidate_labels(&self, btype: Option<&str>, subtype: Option<&str>) -> Vec<String> {
        let btype = match btype {
            Some(t) => t,
            None => return vec![],
        };
        let spec = match self.types.get(btype) {
            Some(s) => s,
            None => return vec![],
        };
        if let Some(st) = subtype {
            let st = st.trim().to_lowercase();
            if !st.is_empty() {
                for (key, mapped) in &spec.subtype_routing {
                    if key.trim().to_lowercase() == st {
                        return mapped.clone();
                    }
                }
            }
        }
        spec.labels.clone()
    }

    /// SPOKE identifier strings derived from BioOKF xref CURIEs for `label`.
    pub fn xref_identifiers(&self, label: &str, xrefs: &[String]) -> Vec<String> {
        let rules = match self.xref_map.get(label) {
            Some(r) => r,
            None => return vec![],
        };
        let mut out = vec![];
        for x in xrefs {
            let (ns, local) = match x.split_once(':') {
                Some((a, b)) => (a.to_string(), b.to_string()),
                None => (String::new(), x.clone()),
            };
            if let Some(tmpl) = rules.get(&ns) {
                out.push(tmpl.replace("{id}", &local).replace("{curie}", x));
            }
        }
        out
    }
}

#[derive(Deserialize, Default)]
struct PredSpec {
    #[serde(default)]
    families: Vec<String>,
    #[serde(default)]
    related_families: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    negatable: bool,
}

#[derive(Deserialize, Default)]
struct PredCrosswalkFile {
    #[serde(default)]
    provenance_only: Vec<String>,
    #[serde(default)]
    predicates: HashMap<String, PredSpec>,
}

pub struct PredicateCrosswalk {
    provenance_only: HashSet<String>,
    predicates: HashMap<String, PredSpec>,
}

impl PredicateCrosswalk {
    pub fn load() -> Result<Self> {
        Self::load_path(&config::data_path("predicate_crosswalk.yaml"))
    }

    pub fn load_path(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let data: PredCrosswalkFile =
            serde_yaml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        Ok(PredicateCrosswalk {
            provenance_only: data.provenance_only.into_iter().collect(),
            predicates: data.predicates,
        })
    }

    pub fn predicate_keys(&self) -> HashSet<String> {
        self.predicates.keys().cloned().collect()
    }

    pub fn provenance_keys(&self) -> HashSet<String> {
        self.provenance_only.clone()
    }

    pub fn is_provenance_only(&self, base: &str) -> bool {
        self.provenance_only.contains(base)
    }

    pub fn expected_families(&self, base: &str) -> Vec<String> {
        self.predicates
            .get(base)
            .map(|s| s.families.clone())
            .unwrap_or_default()
    }

    pub fn related_families(&self, base: &str) -> Vec<String> {
        self.predicates
            .get(base)
            .map(|s| s.related_families.clone())
            .unwrap_or_default()
    }
}
