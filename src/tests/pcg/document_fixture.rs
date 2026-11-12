use crate::pcg::documents::{Request, Session};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

pub(crate) struct Fixture {
    pub _dir: tempfile::TempDir,
    pub root: PathBuf,
    pub profile: PathBuf,
}
impl Fixture {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        fs::create_dir(root.join("docs")).unwrap();
        fs::write(root.join("evidence.txt"), "Exact evidence.\n").unwrap();
        let profile = root.join("documents.json");
        fs::write(&profile,serde_json::to_vec(&json!({"schema_version":1,"workspace_root":root,
            "database_path":root.join("documents.sqlite"),"document_roots":["docs"],
            "max_files":128,"max_nodes":128,"max_edges":512,"max_bytes":1048576,"token_budget":32768})).unwrap()).unwrap();
        Self {
            _dir: dir,
            root,
            profile,
        }
    }
    pub fn declaration(id: &str, status: &str) -> Value {
        json!({"node":id,"kind":"MISSION","status":status,"title":format!("Architecture {id}"),
            "track":"ARCHITECTURE","artifact":"evidence.txt","relationships":[]})
    }
    pub fn write(&self, name: &str, value: &Value) {
        let mut text = String::from("# Architectural evidence\n\n```architecture-graph\n");
        for (key, value) in value.as_object().unwrap() {
            if key != "relationships" {
                text.push_str(&format!(
                    "{key}: {}\n",
                    value
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| value.to_string())
                ));
            }
        }
        let mut fields = std::collections::BTreeMap::<String, Vec<String>>::new();
        for edge in value["relationships"].as_array().unwrap() {
            fields
                .entry(edge["kind"].as_str().unwrap().to_ascii_lowercase())
                .or_default()
                .push(edge["target"].as_str().unwrap().into());
        }
        for (kind, targets) in fields {
            text.push_str(&format!("{kind}:\n"));
            for target in targets {
                text.push_str(&format!("  - {target}\n"));
            }
        }
        text.push_str("```\n");
        fs::write(self.root.join("docs").join(name), text).unwrap();
    }
    pub fn request(&self, mode: &str, id: Option<&str>) -> Request {
        Request {
            workspace_root: self.root.clone(),
            mode: mode.into(),
            id: id.map(str::to_owned),
            track: None,
            text: None,
            tokenizer: "o200k".into(),
            token_budget: 32768,
            max_nodes: None,
            max_edges: None,
            response_format: None,
        }
    }
    pub fn payload(&self, mode: &str, id: Option<&str>) -> Value {
        let response = Session::new(self.profile.clone())
            .query(&self.request(mode, id))
            .unwrap();
        Self::decode(&response.serialized_result)
    }
    pub fn decode(result: &str) -> Value {
        let result: Value = serde_json::from_str(result).unwrap();
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap()
    }
}
