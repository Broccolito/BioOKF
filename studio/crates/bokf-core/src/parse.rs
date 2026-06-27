//! Parse a single concept-document Markdown file into a `Node`, splitting the
//! YAML frontmatter from the body and normalizing deprecated v0.4/earlier forms
//! to v0.5 (title/id -> identifier, *_kind -> subtype, inverse predicates ->
//! forward, provided_by -> a reported_in edge).

use crate::model::*;
use serde_yaml::Value as Yaml;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("no YAML frontmatter (file must start with '---')")]
    NoFrontmatter,
    #[error("unterminated YAML frontmatter (missing closing '---')")]
    UnterminatedFrontmatter,
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("frontmatter is not a mapping")]
    NotMapping,
    #[error("missing required field `type`")]
    MissingType,
    #[error("missing required field `identifier` (or title/id)")]
    MissingIdentifier,
}

/// Split a markdown file into (frontmatter_yaml_text, body).
pub fn split_frontmatter(content: &str) -> Result<(&str, &str), ParseError> {
    let c = content.strip_prefix('\u{feff}').unwrap_or(content); // strip BOM
    let rest = c
        .strip_prefix("---\n")
        .or_else(|| c.strip_prefix("---\r\n"))
        .ok_or(ParseError::NoFrontmatter)?;
    // Find the closing delimiter at the start of a line.
    let mut idx = 0usize;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed == "---" || trimmed == "..." {
            let fm = &rest[..idx];
            let body_start = idx + line.len();
            let body = rest.get(body_start..).unwrap_or("");
            return Ok((fm, body.trim_start_matches('\n')));
        }
        idx += line.len();
    }
    Err(ParseError::UnterminatedFrontmatter)
}

fn yaml_to_json(y: &Yaml) -> serde_json::Value {
    match y {
        Yaml::Null => serde_json::Value::Null,
        Yaml::Bool(b) => serde_json::Value::Bool(*b),
        Yaml::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::json!(i)
            } else if let Some(f) = n.as_f64() {
                serde_json::json!(f)
            } else {
                serde_json::Value::String(n.to_string())
            }
        }
        Yaml::String(s) => serde_json::Value::String(s.clone()),
        Yaml::Sequence(seq) => serde_json::Value::Array(seq.iter().map(yaml_to_json).collect()),
        Yaml::Mapping(m) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in m {
                obj.insert(yaml_key(k), yaml_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        Yaml::Tagged(t) => yaml_to_json(&t.value),
    }
}

fn yaml_key(k: &Yaml) -> String {
    match k {
        Yaml::String(s) => s.clone(),
        other => yaml_to_json(other).to_string(),
    }
}

fn as_string(y: &Yaml) -> Option<String> {
    match y {
        Yaml::String(s) => Some(s.clone()),
        Yaml::Number(n) => Some(n.to_string()),
        Yaml::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Accept either a scalar or a sequence and produce a `Vec<String>`.
fn as_string_list(y: &Yaml) -> Vec<String> {
    match y {
        Yaml::Sequence(seq) => seq.iter().filter_map(as_string).collect(),
        other => as_string(other).into_iter().collect(),
    }
}

fn get<'a>(m: &'a serde_yaml::Mapping, key: &str) -> Option<&'a Yaml> {
    m.get(Yaml::String(key.to_string()))
}

/// Parse YAML frontmatter, retrying with a repair pass if strict parsing fails.
/// The repair quotes plain scalar values that contain an unescaped `": "` — the
/// single most common LLM-authoring mistake (e.g. `description: A: B`).
fn parse_yaml_lenient(fm: &str) -> Result<Yaml, serde_yaml::Error> {
    match serde_yaml::from_str(fm) {
        Ok(v) => Ok(v),
        Err(_) => serde_yaml::from_str(&repair_frontmatter(fm)),
    }
}

fn repair_frontmatter(fm: &str) -> String {
    let re = regex::Regex::new(r"^(\s*(?:- )?)([A-Za-z0-9_]+):[ \t]+(.*\S)[ \t]*$").unwrap();
    fm.lines()
        .map(|line| {
            if let Some(caps) = re.captures(line) {
                let indent = &caps[1];
                let key = &caps[2];
                let val = caps[3].trim();
                let first = val.chars().next().unwrap_or(' ');
                let already = matches!(first, '[' | '{' | '"' | '\'' | '>' | '|' | '&' | '*' | '#');
                if !already && val.contains(": ") {
                    let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
                    return format!("{indent}{key}: \"{escaped}\"");
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse the frontmatter mapping + body into a `Node`.
pub fn parse_node(content: &str, rel_path: &Path) -> Result<Node, ParseError> {
    let (fm_text, body) = split_frontmatter(content)?;
    let value: Yaml = parse_yaml_lenient(fm_text)?;
    let map = value.as_mapping().ok_or(ParseError::NotMapping)?;

    // --- type (required) ---
    let raw_type = get(map, "type")
        .and_then(as_string)
        .ok_or(ParseError::MissingType)?;
    let node_type = NodeType::parse(&raw_type);

    // --- identifier (merge of v0.4 title + id) ---
    let mut xref: Vec<String> = get(map, "xref").map(as_string_list).unwrap_or_default();
    let identifier = if let Some(id) = get(map, "identifier").and_then(as_string) {
        id
    } else {
        let title = get(map, "title").and_then(as_string);
        let legacy_id = get(map, "id").and_then(as_string);
        match (title, legacy_id) {
            (Some(t), Some(i)) => {
                if !xref.contains(&i) {
                    xref.push(i); // demote the old CURIE id to xref
                }
                t
            }
            (Some(t), None) => t,
            (None, Some(i)) => i,
            (None, None) => return Err(ParseError::MissingIdentifier),
        }
    };

    // --- subtype (+ *_kind / class_basis / Structure `method` aliases) ---
    let subtype = get(map, "subtype")
        .and_then(as_string)
        .or_else(|| get(map, "class_basis").and_then(as_string))
        .or_else(|| get(map, "method").and_then(as_string))
        .or_else(|| {
            // any `<type>_kind` key
            map.iter().find_map(|(k, v)| {
                let key = yaml_key(k);
                if key.ends_with("_kind") {
                    as_string(v)
                } else {
                    None
                }
            })
        });

    let synonyms = get(map, "synonyms").map(as_string_list).unwrap_or_default();
    let in_taxon = get(map, "in_taxon").and_then(as_string);
    let note = get(map, "note").and_then(as_string);
    let description = get(map, "description").and_then(as_string);
    let tags = get(map, "tags").map(as_string_list).unwrap_or_default();
    let raw_source = get(map, "raw_source").map(as_string_list).unwrap_or_default();
    let timestamp = get(map, "timestamp").and_then(as_string);

    // --- edges ---
    let mut edges: Vec<Edge> = Vec::new();
    if let Some(Yaml::Sequence(seq)) = get(map, "edges") {
        for item in seq {
            if let Some(edge) = parse_edge(item) {
                edges.push(edge);
            }
        }
    }

    // --- deprecated `provided_by` -> a reported_in edge (v0.5) ---
    if let Some(pb) = get(map, "provided_by").and_then(as_string) {
        edges.push(Edge {
            predicate: Predicate::ReportedIn,
            raw_predicate: "reported_in".to_string(),
            reversed: false,
            object: pb.clone(),
            knowledge_level: Some("knowledge_assertion".to_string()),
            agent_type: Some("manual_agent".to_string()),
            primary_source: Some(pb),
            negated: false,
            direction: None,
            publications: Vec::new(),
            stats: BTreeMap::new(),
            qualifiers: BTreeMap::new(),
            note: Some("normalized from deprecated provided_by".to_string()),
        });
    }

    // --- preserve any unrecognized top-level keys ---
    let known: [&str; 14] = [
        "type", "identifier", "title", "id", "subtype", "synonyms", "xref", "in_taxon", "note",
        "description", "tags", "raw_source", "timestamp", "edges",
    ];
    let mut extra: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for (k, v) in map {
        let key = yaml_key(k);
        if !known.contains(&key.as_str())
            && key != "provided_by"
            && key != "class_basis"
            && key != "method"
            && !key.ends_with("_kind")
        {
            extra.insert(key, yaml_to_json(v));
        }
    }

    Ok(Node {
        node_type,
        raw_type,
        identifier,
        subtype,
        synonyms,
        xref,
        in_taxon,
        note,
        description,
        tags,
        raw_source,
        timestamp,
        edges,
        body: body.to_string(),
        path: rel_path.to_path_buf(),
        extra,
    })
}

fn parse_edge(item: &Yaml) -> Option<Edge> {
    let m = item.as_mapping()?;
    let raw_predicate = get(m, "predicate").and_then(as_string)?;
    let parsed = Predicate::parse(&raw_predicate);
    let object = get(m, "object").and_then(as_string)?;

    // primary_source: an `infores:`/ontology CURIE is a deprecated v0.4 form; we
    // keep the literal value and let lint flag it (it should name a source node).
    let primary_source = get(m, "primary_source").and_then(as_string);

    let known: [&str; 8] = [
        "predicate",
        "object",
        "knowledge_level",
        "agent_type",
        "primary_source",
        "negated",
        "direction",
        "publications",
    ];
    let mut stats: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut qualifiers: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut note = None;
    for (k, v) in m {
        let key = yaml_key(k);
        if key == "qualifiers" {
            if let Yaml::Mapping(qm) = v {
                for (qk, qv) in qm {
                    qualifiers.insert(yaml_key(qk), yaml_to_json(qv));
                }
            }
        } else if key == "note" {
            note = as_string(v);
        } else if !known.contains(&key.as_str()) {
            stats.insert(key, yaml_to_json(v));
        }
    }

    // negation: a `not_<X>` predicate (parsed above) OR a legacy `negated: true`
    // qualifier on a negatable positive — both normalize to the canonical `not_<X>`.
    let negated_flag = get(m, "negated").and_then(|v| v.as_bool()).unwrap_or(false);
    let mut predicate = parsed.predicate;
    if negated_flag {
        if let Some(neg) = predicate.negated_form() {
            predicate = neg;
        }
    }
    let negated = predicate.is_negative() || negated_flag;

    Some(Edge {
        predicate,
        raw_predicate,
        reversed: parsed.reversed,
        object,
        knowledge_level: get(m, "knowledge_level").and_then(as_string),
        agent_type: get(m, "agent_type").and_then(as_string),
        primary_source,
        negated,
        direction: get(m, "direction").and_then(as_string),
        publications: get(m, "publications").map(as_string_list).unwrap_or_default(),
        stats,
        qualifiers,
        note,
    })
}
