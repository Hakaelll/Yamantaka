use super::*;

pub(super) fn preferences(payload: &Value) -> Result<Preferences> {
    let Some(extension) = payload.get("rustPreferences") else {
        return Ok(Preferences {
            legacy: serde_json::from_value(payload["settings"].clone()).map_err(storage)?,
            ..Default::default()
        });
    };
    let map = extension
        .as_object()
        .ok_or_else(|| Failure::Invalid("Invalid backup preferences".into()))?;
    if map.len() != 3
        || !["version", "legacy", "rust"]
            .iter()
            .all(|key| map.contains_key(*key))
    {
        return Err(Failure::Invalid(
            "Unsupported backup preferences fields".into(),
        ));
    }
    let preferences: Preferences = serde_json::from_value(extension.clone()).map_err(storage)?;
    if preferences.version != 1 {
        return Err(Failure::Invalid(
            "Unsupported backup preferences version".into(),
        ));
    }
    let mut merged = preferences.legacy.clone();
    merged.extend(preferences.rust.clone());
    if serde_json::to_value(merged).map_err(storage)? != payload["settings"] {
        return Err(Failure::Invalid("Backup preference views disagree".into()));
    }
    Ok(preferences)
}
