//! Native text-only NuExtract3 adapter for the benchmark transport.

use serde_json::{Value, json};

use crate::{
    ModelProposal, ModelRequest,
    client::{AttemptError, extract_json_object, facts_from_text},
    error::EngineErrorCode,
};

const BEGIN: &str = "--- BEGIN EVIDENCE ---\n";
const END: &str = "\n--- END EVIDENCE ---";

const INSTRUCTIONS: &str = "Extract only facts supported by the evidence. For each *_ids or ids field, copy the supporting line's bracketed ID. Use null for an unknown value. Type is the document's specific own title, without date or identifier. Date is this document's defining date: prefer its explicit effective date over signature, and never pick a due date, deadline, end date, or a date of a referenced document. Date roles: effective, execution, notice, termination, amendment, invoice, filing, issuance, other. Name up to three parties in their own words, keeping organizations and leaving out staff, people merely mentioned, and copies. Include issuer and customer for an invoice. Party roles: client, contractor, employer, employee, buyer, seller, landlord, tenant, issuer, recipient, vendor, customer, borrower, lender, licensor, licensee, sender, addressee, other. Subject is the specific work, goods, premises, position or matter in up to eight words. For amount_id cite the line with the main total, if any.";

fn evidence_lines(prompt: &str) -> Option<&str> {
    let (_, tail) = prompt.split_once(BEGIN)?;
    let (lines, _) = tail.rsplit_once(END)?;
    Some(lines)
}

pub(super) fn request(original: &Value) -> Result<Value, String> {
    let prompt = original["messages"]
        .as_array()
        .and_then(|messages| messages.last())
        .and_then(|turn| turn["content"].as_str())
        .and_then(evidence_lines)
        .ok_or("nuextract3 requires an Intern evidence prompt")?;
    let template = json!({
        "type": "verbatim-string", "type_ids": "verbatim-string",
        "date": "date", "date_role": "string", "date_ids": "verbatim-string",
        "parties": [{"name": "verbatim-string", "role": "string", "ids": "verbatim-string"}],
        "subject": "verbatim-string", "subject_ids": "verbatim-string",
        "amount_id": "verbatim-string"
    });
    let mut body = original.clone();
    let fields = body.as_object_mut().ok_or("request must be an object")?;
    for key in ["grammar", "logprobs", "top_logprobs"] {
        fields.remove(key);
    }
    body["messages"] = json!([{"role": "user", "content": prompt}]);
    body["chat_template_kwargs"] = json!({
        "enable_thinking": false,
        "template": template.to_string(),
        "instructions": INSTRUCTIONS
    });
    Ok(body)
}

pub(crate) fn proposal(
    content: &str,
    request: &ModelRequest,
) -> Result<ModelProposal, AttemptError> {
    let invalid = || AttemptError(EngineErrorCode::ModelResponseInvalid);
    let handles = request.evidence.as_ref().ok_or_else(invalid)?;
    let lines = evidence_lines(&request.prompt).ok_or_else(invalid)?;
    let mut value: Value = serde_json::from_str(extract_json_object(content).ok_or_else(invalid)?)
        .map_err(|_| invalid())?;
    let fields = value.as_object_mut().ok_or_else(invalid)?;
    if !["type", "date", "parties", "subject"]
        .iter()
        .any(|key| fields.contains_key(*key))
    {
        return Err(invalid());
    }
    for field in ["type_ids", "date_ids", "subject_ids"] {
        if let Some(id) = fields.get_mut(field) {
            normalize_citation(id, lines, handles);
        }
    }
    let mut amount = fields.remove("amount_id").unwrap_or(Value::Null);
    normalize_citation(&mut amount, lines, handles);
    let parties = fields.entry("parties").or_insert(Value::Null);
    if parties.is_null() {
        *parties = json!([]);
    }
    for party in parties.as_array_mut().ok_or_else(invalid)? {
        if let Some(id) = party.get_mut("ids") {
            normalize_citation(id, lines, handles);
        }
    }
    // Like a compact Qwen reply: this is a structural value for the existing
    // validator. The adapter has no calibrated token-confidence signal.
    fields.insert("confidence".into(), json!(1.0));
    let mut proposal = facts_from_text(&value.to_string(), handles)?;
    if let Some(facts) = &mut proposal.facts {
        if let Some(id) = amount.as_str().and_then(|id| handles.resolve(id)) {
            facts.amount_evidence.push(id.to_owned());
        }
    }
    Ok(proposal)
}

/// Prefer the copied handle. If the model instead copied a line exactly,
/// recover its ID only when a unique line in the sent evidence matches it.
fn normalize_citation(value: &mut Value, lines: &str, handles: &crate::EvidenceHandles) {
    let Some(citation) = value.as_str().map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };
    if handles.resolve(citation).is_some() {
        return;
    }
    let mut matching = lines.lines().filter_map(|line| {
        let (id, text) = line.strip_prefix('[')?.split_once("] ")?;
        (text == citation && handles.resolve(id).is_some()).then_some(id)
    });
    if let Some(id) = matching.next() {
        if matching.next().is_none() {
            *value = Value::String(id.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_template_keeps_the_evidence_and_drops_the_qwen_grammar() {
        let original = json!({
            "messages": [{"role":"system","content":"original"}, {
                "role":"user","content":"excerpts\n--- BEGIN EVIDENCE ---\n[p1.b1] Invoice\n--- END EVIDENCE ---\nJSON only."
            }],
            "grammar":"rule", "logprobs":true, "top_logprobs":1, "max_tokens":1024
        });
        let sent = request(&original).unwrap();
        assert_eq!(
            sent["messages"],
            json!([{"role":"user","content":"[p1.b1] Invoice"}])
        );
        assert!(sent.get("grammar").is_none());
        assert!(sent.get("logprobs").is_none());
        assert_eq!(sent["max_tokens"], 1024);
        let template = sent["chat_template_kwargs"]["template"].as_str().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(template).unwrap()["date"],
            "date"
        );
    }

    #[test]
    fn handles_and_unique_quotes_resolve_but_ambiguous_quotes_stay_unsupported() {
        let mut request = ModelRequest::new(format!(
            "{BEGIN}[1] Invoice\n[2] Invoice date: May 4, 2026\n[3] North Ltd\n[4] North Ltd{END}"
        ));
        request.evidence = Some(crate::EvidenceHandles::new(
            (1..=4)
                .map(|n| (n.to_string(), format!("p1.b{n}")))
                .collect(),
        ));
        let reply = proposal(
            r#"{"type":"Invoice","type_ids":"[1]","date":"2026-05-04","date_role":"invoice",
               "date_ids":"Invoice date: May 4, 2026","parties":[{"name":"North Ltd","role":"issuer","ids":"North Ltd"}]}"#,
            &request,
        ).unwrap();
        let facts = reply.facts.unwrap();
        assert_eq!(facts.type_evidence, ["p1.b1"]);
        assert_eq!(facts.date_evidence, ["p1.b2"]);
        assert!(facts.parties[0].evidence.is_empty());
        assert_eq!(facts.unknown_evidence, ["North Ltd"]);
        assert!(proposal(r#"{"unrelated":"json"}"#, &request).is_err());
    }
}
