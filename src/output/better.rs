use anyhow::Result;
use serde::Serialize;
use serde_json::{Value, json};

pub fn envelope<T: Serialize>(command: &str, data: T) -> Result<String> {
  let v = json!({
      "ok": true,
      "command": command,
      "data": data,
  });
  Ok(serde_json::to_string_pretty(&v)?)
}

pub fn envelope_with_hints<T: Serialize>(
  command: &str,
  data: T,
  hints: Vec<String>,
  meta: Value,
) -> Result<String> {
  let v = json!({
      "ok": true,
      "command": command,
      "data": data,
      "hints": hints,
      "meta": meta,
  });
  Ok(serde_json::to_string_pretty(&v)?)
}

pub fn error_envelope(command: &str, error: &str, hints: Vec<String>) -> Result<String> {
  let v = json!({
      "ok": false,
      "command": command,
      "error": error,
      "hints": hints,
  });
  Ok(serde_json::to_string_pretty(&v)?)
}
