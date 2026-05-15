use super::{Provider, ProviderInput};
use crate::crypto;
use rusqlite::{params, Connection, Result as SqliteResult};
use uuid::Uuid;

pub fn list_providers(conn: &Connection) -> SqliteResult<Vec<Provider>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, prov_type, base_url, api_key, extra_headers, created_at, updated_at FROM providers ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        let api_key_encrypted: Vec<u8> = row.get(4)?;
        let api_key = crypto::decrypt(&api_key_encrypted).unwrap_or_default();
        let masked_key = mask_api_key(&api_key);
        Ok(Provider {
            id: row.get(0)?,
            name: row.get(1)?,
            prov_type: row.get(2)?,
            base_url: row.get(3)?,
            api_key: masked_key,
            extra_headers: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn get_provider(conn: &Connection, id: &str) -> SqliteResult<Option<Provider>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, prov_type, base_url, api_key, extra_headers, created_at, updated_at FROM providers WHERE id = ?1"
    )?;
    let mut rows = stmt.query_map([id], |row| {
        let api_key_encrypted: Vec<u8> = row.get(4)?;
        let api_key = crypto::decrypt(&api_key_encrypted).unwrap_or_default();
        let masked_key = mask_api_key(&api_key);
        Ok(Provider {
            id: row.get(0)?,
            name: row.get(1)?,
            prov_type: row.get(2)?,
            base_url: row.get(3)?,
            api_key: masked_key,
            extra_headers: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn add_provider(conn: &mut Connection, input: &ProviderInput) -> Result<Provider, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let encrypted_key = crypto::encrypt(&input.api_key)?;
    tx.execute(
        "INSERT INTO providers (id, name, prov_type, base_url, api_key, extra_headers, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            &id,
            &input.name,
            &input.prov_type,
            &input.base_url,
            encrypted_key,
            &input.extra_headers,
            now,
            now
        ],
    ).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(Provider {
        id,
        name: input.name.clone(),
        prov_type: input.prov_type.clone(),
        base_url: input.base_url.clone(),
        api_key: mask_api_key(&input.api_key),
        extra_headers: input.extra_headers.clone(),
        created_at: now,
        updated_at: now,
    })
}

pub fn update_provider(
    conn: &mut Connection,
    id: &str,
    input: &ProviderInput,
) -> Result<Provider, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp_millis();
    let encrypted_key = crypto::encrypt(&input.api_key)?;
    tx.execute(
        "UPDATE providers SET name = ?1, prov_type = ?2, base_url = ?3, api_key = ?4, extra_headers = ?5, updated_at = ?6 WHERE id = ?7",
        params![
            &input.name,
            &input.prov_type,
            &input.base_url,
            encrypted_key,
            &input.extra_headers,
            now,
            id
        ],
    ).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(Provider {
        id: id.to_string(),
        name: input.name.clone(),
        prov_type: input.prov_type.clone(),
        base_url: input.base_url.clone(),
        api_key: mask_api_key(&input.api_key),
        extra_headers: input.extra_headers.clone(),
        created_at: now,
        updated_at: now,
    })
}

pub fn delete_provider(conn: &mut Connection, id: &str) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM providers WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        "****".to_string()
    } else {
        format!("****{}", &key[key.len() - 4..])
    }
}
