//! Derived knowledge rows share the document publication transaction and revision owner.
use super::{model::Candidate, store::sql};
use crate::pcg::{Error, Result, identity, require};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type SearchRow = (String, String, i64, String, Option<String>, String);

fn components(c: &Connection, revision: &str) -> Result<Vec<Value>> {
    let mut statement=sql(c.prepare("SELECT ordinal,role,label,path,source_sha,start,end,content_sha,selector FROM knowledge_component WHERE revision=? ORDER BY ordinal"))?;
    sql(statement.query_map([revision],|r|Ok(json!({
        "ordinal":r.get::<_,usize>(0)?,"role":r.get::<_,String>(1)?,"label":r.get::<_,Option<String>>(2)?,
        "path":r.get::<_,String>(3)?,"source_sha":r.get::<_,String>(4)?,"start":r.get::<_,usize>(5)?,
        "end":r.get::<_,usize>(6)?,"content_sha":r.get::<_,String>(7)?,"selector":r.get::<_,String>(8)?
    }))))?.collect::<rusqlite::Result<_>>().map_err(|_|Error("document_database"))
}

pub fn publish(
    c: &Connection,
    g: &str,
    nodes: &BTreeMap<String, Candidate>,
    changed: bool,
) -> Result<()> {
    let mut source_keys = BTreeSet::new();
    let mut expected_search = Vec::<SearchRow>::new();
    let mut expected_units = Vec::new();
    for (id, node) in nodes {
        let Some(knowledge) = &node.knowledge else {
            continue;
        };
        if changed {
            sql(c.execute(
                "INSERT OR IGNORE INTO knowledge_unit VALUES(?,?,?,?,?,?)",
                params![
                    node.revision,
                    knowledge.scope,
                    node.declaration.title,
                    node.declaration.kind,
                    node.declaration.status,
                    node.declaration.track
                ],
            ))?;
        }
        let actual: (String, String, String, String, String) = sql(c.query_row(
            "SELECT scope,title,kind,status,track FROM knowledge_unit WHERE revision=?",
            [&node.revision],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        ))?;
        require(
            actual
                == (
                    knowledge.scope.clone(),
                    node.declaration.title.clone(),
                    node.declaration.kind.clone(),
                    node.declaration.status.clone(),
                    node.declaration.track.clone(),
                ),
            "knowledge_corruption",
        )?;
        expected_units.push(node.revision.clone());
        expected_search.push((
            id.clone(),
            node.revision.clone(),
            -1,
            "identity_title".into(),
            None,
            format!("{}\n{}", id, node.declaration.title),
        ));
        let mut expected_components = Vec::new();
        let mut composed = String::new();
        for (ordinal, component) in knowledge.components.iter().enumerate() {
            let binding = &component.binding;
            if source_keys.insert((binding.path.clone(), component.source_sha.clone())) {
                if changed {
                    sql(c.execute(
                        "INSERT OR IGNORE INTO document_source VALUES(?,?,?)",
                        params![
                            binding.path,
                            component.source_sha,
                            component.source.as_slice()
                        ],
                    ))?;
                }
                let body: Vec<u8> = sql(c.query_row(
                    "SELECT body FROM document_source WHERE path=? AND sha=?",
                    params![binding.path, component.source_sha],
                    |r| r.get(0),
                ))?;
                require(
                    body.as_slice() == component.source.as_slice(),
                    "knowledge_corruption",
                )?;
            }
            if changed {
                sql(c.execute(
                    "INSERT OR IGNORE INTO knowledge_component VALUES(?,?,?,?,?,?,?,?,?,?)",
                    params![
                        node.revision,
                        ordinal,
                        binding.role,
                        binding.label,
                        binding.path,
                        component.source_sha,
                        component.start,
                        component.end,
                        component.content_sha,
                        binding.selector
                    ],
                ))?;
            }
            expected_components.push(json!({"ordinal":ordinal,"role":binding.role,"label":binding.label,"path":binding.path,
                "source_sha":component.source_sha,"start":component.start,"end":component.end,"content_sha":component.content_sha,"selector":binding.selector}));
            let text = std::str::from_utf8(&component.source[component.start..component.end])
                .map_err(|_| Error("invalid_utf8"))?;
            composed.push_str(text);
            composed.push('\n');
            expected_search.push((
                id.clone(),
                node.revision.clone(),
                ordinal as i64,
                binding.role.clone(),
                binding.label.clone(),
                text.into(),
            ));
        }
        expected_search.push((
            id.clone(),
            node.revision.clone(),
            -2,
            "composed_knowledge".into(),
            None,
            composed,
        ));
        require(
            components(c, &node.revision)? == expected_components,
            "knowledge_corruption",
        )?;
    }
    expected_units.sort();
    let actual_units = {
        let mut s=sql(c.prepare("SELECT u.revision FROM knowledge_unit u JOIN document_member m USING(revision) WHERE m.g=? ORDER BY u.revision"))?;
        sql(s.query_map([g], |r| r.get::<_, String>(0)))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| Error("document_database"))?
    };
    require(actual_units == expected_units, "knowledge_corruption")?;
    if changed {
        sql(c.execute("DELETE FROM knowledge_search", []))?;
        for (id, revision, ordinal, origin, label, text) in &expected_search {
            sql(c.execute(
                "INSERT INTO knowledge_search VALUES(?,?,?,?,?,?)",
                params![id, revision, ordinal, origin, label, text],
            ))?;
        }
    }
    expected_search.sort();
    let mut s=sql(c.prepare("SELECT id,revision,ordinal,origin,label,text FROM knowledge_search ORDER BY id,revision,ordinal,origin,label,text"))?;
    let actual = sql(s.query_map([], |r| {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get(2)?,
            r.get(3)?,
            r.get(4)?,
            r.get(5)?,
        ))
    }))?
    .collect::<rusqlite::Result<Vec<SearchRow>>>()
    .map_err(|_| Error("document_database"))?;
    require(actual == expected_search, "knowledge_corruption")?;
    Ok(())
}

pub fn content(c: &Connection, revision: &str) -> Result<Vec<Value>> {
    let mut result = components(c, revision)?;
    for component in &mut result {
        let body: Vec<u8> = sql(c.query_row(
            "SELECT substr(body,?3+1,?4-?3) FROM document_source WHERE path=?1 AND sha=?2",
            params![
                component["path"].as_str(),
                component["source_sha"].as_str(),
                component["start"].as_u64(),
                component["end"].as_u64()
            ],
            |r| r.get(0),
        ))?;
        require(
            identity::sha(&body)
                == component["content_sha"]
                    .as_str()
                    .ok_or(Error("knowledge_corruption"))?,
            "knowledge_corruption",
        )?;
        component["text"] = String::from_utf8(body)
            .map_err(|_| Error("knowledge_corruption"))?
            .into();
    }
    Ok(result)
}
