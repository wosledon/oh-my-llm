use super::{ModelInput, ModelMapping};
use rusqlite::{params, Connection, Result as SqliteResult};
use uuid::Uuid;

pub fn list_models(
    conn: &Connection,
    provider_id: Option<&str>,
) -> SqliteResult<Vec<ModelMapping>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, exposed_name, upstream_name, enabled, input_price, output_price FROM model_mappings WHERE provider_id = COALESCE(?1, provider_id) ORDER BY exposed_name"
    )?;
    let rows = stmt.query_map([provider_id], |row| {
        Ok(ModelMapping {
            id: row.get(0)?,
            provider_id: row.get(1)?,
            exposed_name: row.get(2)?,
            upstream_name: row.get(3)?,
            enabled: row.get::<_, i32>(4)? != 0,
            input_price: row.get(5)?,
            output_price: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn get_model(conn: &Connection, id: &str) -> SqliteResult<Option<ModelMapping>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, exposed_name, upstream_name, enabled, input_price, output_price FROM model_mappings WHERE id = ?1"
    )?;
    let mut rows = stmt.query_map([id], |row| {
        Ok(ModelMapping {
            id: row.get(0)?,
            provider_id: row.get(1)?,
            exposed_name: row.get(2)?,
            upstream_name: row.get(3)?,
            enabled: row.get::<_, i32>(4)? != 0,
            input_price: row.get(5)?,
            output_price: row.get(6)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn add_model(conn: &mut Connection, input: &ModelInput) -> Result<ModelMapping, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO model_mappings (id, provider_id, exposed_name, upstream_name, enabled, input_price, output_price) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &id,
            &input.provider_id,
            &input.exposed_name,
            &input.upstream_name,
            if input.enabled { 1 } else { 0 },
            input.input_price,
            input.output_price,
        ],
    ).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(ModelMapping {
        id,
        provider_id: input.provider_id.clone(),
        exposed_name: input.exposed_name.clone(),
        upstream_name: input.upstream_name.clone(),
        enabled: input.enabled,
        input_price: input.input_price,
        output_price: input.output_price,
    })
}

pub fn update_model(
    conn: &mut Connection,
    id: &str,
    input: &ModelInput,
) -> Result<ModelMapping, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE model_mappings SET provider_id = ?1, exposed_name = ?2, upstream_name = ?3, enabled = ?4, input_price = ?5, output_price = ?6 WHERE id = ?7",
        params![
            &input.provider_id,
            &input.exposed_name,
            &input.upstream_name,
            if input.enabled { 1 } else { 0 },
            input.input_price,
            input.output_price,
            id,
        ],
    ).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(ModelMapping {
        id: id.to_string(),
        provider_id: input.provider_id.clone(),
        exposed_name: input.exposed_name.clone(),
        upstream_name: input.upstream_name.clone(),
        enabled: input.enabled,
        input_price: input.input_price,
        output_price: input.output_price,
    })
}

pub fn delete_model(conn: &mut Connection, id: &str) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM model_mappings WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}
