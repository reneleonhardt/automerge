use automerge as am;
use automerge::transaction::Transactable;

pub(crate) fn initialize_from_json(
    json_value: &serde_json::Value,
) -> anyhow::Result<am::AutoCommit> {
    let mut doc = am::AutoCommit::new();
    match json_value {
        serde_json::Value::Object(m) => {
            import_map(&mut doc, &am::ObjId::Root, m)?;
            Ok(doc)
        }
        _ => anyhow::bail!("expected an object"),
    }
}

fn import_map(
    doc: &mut am::AutoCommit,
    obj: &am::ObjId,
    map: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<()> {
    for (key, value) in map {
        match value {
            serde_json::Value::Null => {
                doc.put(obj, key, ())?;
            }
            serde_json::Value::Bool(b) => {
                doc.put(obj, key, *b)?;
            }
            serde_json::Value::String(s) => {
                doc.put(obj, key, s)?;
            }
            serde_json::Value::Array(vec) => {
                let id = doc.put_object(obj, key, am::ObjType::List)?;
                import_list(doc, &id, vec)?;
            }
            serde_json::Value::Number(n) => {
                if let Some(m) = n.as_i64() {
                    doc.put(obj, key, m)?;
                } else if let Some(m) = n.as_u64() {
                    doc.put(obj, key, m)?;
                } else if let Some(m) = n.as_f64() {
                    doc.put(obj, key, m)?;
                } else {
                    anyhow::bail!("not a number");
                }
            }
            serde_json::Value::Object(map) => {
                let id = doc.put_object(obj, key, am::ObjType::Map)?;
                import_map(doc, &id, map)?;
            }
        }
    }
    Ok(())
}

fn import_list(
    doc: &mut am::AutoCommit,
    obj: &am::ObjId,
    list: &[serde_json::Value],
) -> anyhow::Result<()> {
    for (i, value) in list.iter().enumerate() {
        match value {
            serde_json::Value::Null => {
                doc.insert(obj, i, ())?;
            }
            serde_json::Value::Bool(b) => {
                doc.insert(obj, i, *b)?;
            }
            serde_json::Value::String(s) => {
                doc.insert(obj, i, s)?;
            }
            serde_json::Value::Array(vec) => {
                let id = doc.insert_object(obj, i, am::ObjType::List)?;
                import_list(doc, &id, vec)?;
            }
            serde_json::Value::Number(n) => {
                if let Some(m) = n.as_i64() {
                    doc.insert(obj, i, m)?;
                } else if let Some(m) = n.as_u64() {
                    doc.insert(obj, i, m)?;
                } else if let Some(m) = n.as_f64() {
                    doc.insert(obj, i, m)?;
                } else {
                    anyhow::bail!("not a number");
                }
            }
            serde_json::Value::Object(map) => {
                let id = doc.insert_object(obj, i, am::ObjType::Map)?;
                import_map(doc, &id, map)?;
            }
        }
    }
    Ok(())
}

pub fn import_json(
    mut reader: impl std::io::Read,
    mut writer: impl std::io::Write,
) -> anyhow::Result<()> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    let json_value: serde_json::Value = serde_json::from_str(&buffer)?;
    let mut doc = initialize_from_json(&json_value)?;
    writer.write_all(&doc.save())?;
    Ok(())
}

fn toml_to_json(value: toml::Value) -> anyhow::Result<serde_json::Value> {
    Ok(match value {
        toml::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .map(toml_to_json)
                .collect::<anyhow::Result<_>>()?,
        ),
        toml::Value::Boolean(value) => serde_json::Value::Bool(value),
        toml::Value::Datetime(value) => serde_json::Value::String(value.to_string()),
        toml::Value::Float(value) => serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .ok_or_else(|| anyhow::anyhow!("TOML float is not a JSON number: {value}"))?,
        toml::Value::Integer(value) => serde_json::Value::Number(value.into()),
        toml::Value::String(value) => serde_json::Value::String(value),
        toml::Value::Table(table) => serde_json::Value::Object(
            table
                .into_iter()
                .map(|(key, value)| Ok((key, toml_to_json(value)?)))
                .collect::<anyhow::Result<_>>()?,
        ),
    })
}

pub fn import_toml(
    mut reader: impl std::io::Read,
    mut writer: impl std::io::Write,
) -> anyhow::Result<()> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    let toml_value: toml::Value = toml::from_str(&buffer)?;
    let json_value = toml_to_json(toml_value)?;
    let mut doc = initialize_from_json(&json_value)?;
    writer.write_all(&doc.save())?;
    Ok(())
}
