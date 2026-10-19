//! Opt-in laboratory request transformation and immutable wire capture.
//! Not enabled by the desktop application. No code loading or network listener.
use serde_json::Value;

pub fn transform(original: &Value, profile: &Value) -> Result<Value, String> {
    let p = profile.as_object().ok_or("profile must be an object")?;
    if p.keys()
        .any(|k| !["settings", "prompt"].contains(&k.as_str()))
    {
        return Err("unknown benchmark profile field".into());
    }
    let settings = p
        .get("settings")
        .and_then(Value::as_object)
        .ok_or("settings must be an object")?;
    let mut body = original.clone();
    for (key, value) in settings {
        if ![
            "temperature",
            "top_k",
            "top_p",
            "min_p",
            "presence_penalty",
            "repeat_penalty",
            "repeat_last_n",
            "max_tokens",
            "cache_prompt",
            "chat_template_kwargs",
            "seed",
        ]
        .contains(&key.as_str())
        {
            return Err(format!("unsupported benchmark setting {key}"));
        }
        match key.as_str() {
            "cache_prompt" if value.is_boolean() => {}
            "chat_template_kwargs" if value == &serde_json::json!({"enable_thinking":false}) => {}
            "seed" | "top_k" | "repeat_last_n"
                if value.as_u64().is_some_and(|v| v <= 1_000_000) => {}
            "max_tokens" if value.as_u64().is_some_and(|v| v > 0 && v <= 1024) => {}
            "temperature" | "top_p" | "min_p" | "presence_penalty" | "repeat_penalty"
                if value
                    .as_f64()
                    .is_some_and(|v| v.is_finite() && (0.0..=2.0).contains(&v)) => {}
            _ => return Err(format!("invalid benchmark setting {key}")),
        }
        body[key] = value.clone();
    }
    if let Some(prompt) = p.get("prompt") {
        let p = prompt.as_object().ok_or("prompt must be an object")?;
        if p.keys()
            .any(|k| !["system", "examples", "prefix", "suffix"].contains(&k.as_str()))
        {
            return Err("unknown prompt field".into());
        }
        let system = p
            .get("system")
            .and_then(Value::as_str)
            .ok_or("system must be text")?;
        let examples = p
            .get("examples")
            .and_then(Value::as_array)
            .ok_or("examples must be an array")?;
        if examples.len() > 4 {
            return Err("too many example turns".into());
        }
        for (i, e) in examples.iter().enumerate() {
            let role = if i % 2 == 0 { "user" } else { "assistant" };
            if e.as_object().is_none_or(|o| o.len() != 2)
                || e["role"] != role
                || !e["content"].is_string()
            {
                return Err("invalid example turn".into());
            }
        }
        if examples.len() % 2 != 0 {
            return Err("example needs a complete pair".into());
        }
        let user = body["messages"]
            .as_array()
            .and_then(|m| m.last())
            .and_then(|m| m["content"].as_str())
            .ok_or("missing user turn")?;
        let prefix = p
            .get("prefix")
            .and_then(Value::as_str)
            .ok_or("prefix must be text")?;
        let suffix = p
            .get("suffix")
            .and_then(Value::as_str)
            .ok_or("suffix must be text")?;
        let mut messages = vec![serde_json::json!({"role":"system","content":system})];
        messages.extend(examples.iter().cloned());
        messages
            .push(serde_json::json!({"role":"user","content":format!("{prefix}{user}{suffix}")}));
        body["messages"] = Value::Array(messages);
    }
    Ok(body)
}

use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
/// One benchmark process owns one empty output directory. Captures never contain authorization headers.
pub struct BenchmarkTransport {
    profile: Value,
    output: PathBuf,
    sequence: AtomicU64,
}
impl BenchmarkTransport {
    pub fn load(profile: &Path, output: &Path) -> Result<Self, String> {
        let mut bytes = Vec::new();
        std::fs::File::open(profile)
            .map_err(|e| e.to_string())?
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 1_048_576 {
            return Err("profile too large".into());
        }
        let profile: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        transform(
            &serde_json::json!({"messages":[{"role":"user","content":"validation"}]}),
            &profile,
        )?;
        std::fs::create_dir(output).map_err(|e| format!("capture directory must be new: {e}"))?;
        Ok(Self {
            profile,
            output: output.into(),
            sequence: AtomicU64::new(0),
        })
    }
    pub fn prepare(&self, original: &Value, request_sha: &str) -> Result<(Vec<u8>, u64), String> {
        let forwarded = transform(original, &self.profile)?;
        let wire = serde_json::to_vec(&forwarded).map_err(|e| e.to_string())?;
        let id = self.sequence.fetch_add(1, Ordering::SeqCst);
        self.save(id,"request.json",&serde_json::to_vec_pretty(&serde_json::json!({"sequence":id,"native_request_sha256":request_sha,"original":original,"forwarded":forwarded,"effective_wire_sha256":format!("{:x}",Sha256::digest(&wire))})).map_err(|e|e.to_string())?)?;
        self.save(id, "model-wire.json", &wire)?;
        Ok((wire, id))
    }
    pub fn response(&self, id: u64, status: u16, bytes: &[u8], seconds: f64) -> Result<(), String> {
        self.save(id, "response.json", bytes)?;
        self.save(id,"transport.json",&serde_json::to_vec_pretty(&serde_json::json!({"sequence":id,"http_status":status,"model_request_seconds":seconds,"response_sha256":format!("{:x}",Sha256::digest(bytes))})).map_err(|e|e.to_string())?)
    }
    pub fn failed(&self, id: u64, error: &str) -> Result<(), String> {
        self.save(
            id,
            "transport-error.json",
            &serde_json::to_vec(&serde_json::json!({"error":error})).map_err(|e| e.to_string())?,
        )
    }
    fn save(&self, id: u64, suffix: &str, bytes: &[u8]) -> Result<(), String> {
        let mut f = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(self.output.join(format!("{id:04}-{suffix}")))
            .map_err(|e| e.to_string())?;
        f.write_all(bytes)
            .and_then(|_| f.sync_all())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn frozen_profile_preserves_evidence_grammar_and_adds_only_declared_example() {
        let body = json!({"model":"intern-local","messages":[{"role":"system","content":"rules"},{"role":"user","content":"evidence"}],"grammar":"grammar","temperature":0,"max_tokens":1024});
        let profile = json!({"settings":{"seed":17,"top_k":1},"prompt":{"system":"rules","examples":[{"role":"user","content":"example"},{"role":"assistant","content":"answer"}],"prefix":"","suffix":""}});
        let got = transform(&body, &profile).unwrap();
        assert_eq!(
            got,
            json!({"model":"intern-local","messages":[{"role":"system","content":"rules"},{"role":"user","content":"example"},{"role":"assistant","content":"answer"},{"role":"user","content":"evidence"}],"grammar":"grammar","temperature":0,"max_tokens":1024,"seed":17,"top_k":1})
        );
    }
    #[test]
    fn refuses_endpoint_and_grammar_overrides() {
        for key in ["endpoint", "grammar", "messages", "model", "api_key"] {
            let p = json!({"settings":{key:"untrusted"}});
            assert!(transform(&json!({"messages":[]}), &p).is_err());
        }
    }
    #[test]
    fn empty_profile_preserves_original_request_exactly() {
        let b = json!({"model":"x","messages":[{"role":"user","content":"a"}],"grammar":"g","max_tokens":1024});
        assert_eq!(transform(&b, &json!({"settings":{}})).unwrap(), b);
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    #[test]
    fn matches_preserved_effective_requests_before_and_after_fewshot_transport() {
        let rows: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/benchmark-request-parity.json"
        ))
        .unwrap();
        let prompt: Value =
            serde_json::from_str(include_str!("../tests/fixtures/benchmark-profile.json")).unwrap();
        let profile = serde_json::json!({"settings":{"temperature":0,"top_k":1,"max_tokens":1024,"cache_prompt":true,"chat_template_kwargs":{"enable_thinking":false},"seed":17},"prompt":prompt});
        for row in rows.as_array().unwrap() {
            assert_eq!(
                transform(&row["original"], &profile).unwrap(),
                row["expected"]
            );
        }
    }
    #[test]
    fn capture_is_exclusive_and_response_bytes_are_unmodified() {
        let dir = tempfile::tempdir().unwrap();
        let profile = dir.path().join("profile.json");
        std::fs::write(&profile, br#"{"settings":{}}"#).unwrap();
        let t = BenchmarkTransport::load(&profile, &dir.path().join("wire")).unwrap();
        let (_, id) = t
            .prepare(&serde_json::json!({"messages":[]}), "source-id")
            .unwrap();
        let raw = b"{\"choices\":[]}";
        t.response(id, 200, raw, 1.25).unwrap();
        assert_eq!(
            std::fs::read(dir.path().join("wire/0000-response.json")).unwrap(),
            raw
        );
        assert!(t.response(id, 200, b"replacement", 1.0).is_err());
        assert!(BenchmarkTransport::load(&profile, &dir.path().join("wire")).is_err());
    }
}
