// Independently frozen synthetic fidelity controls. Included inside facts::tests.
// Gold assertions come from the synthetic source text, not model output.

fn heldout_description_from_field(field: &str) -> String {
    let source = format!(
        "INSPECTION REPORT\n\nDate: October 3, 2026\n\nEquipment: {field}\n\nThe inspection concerns the equipment identified above. Further measurements require a separate test."
    );
    let (outcome, _) = facts_for(&source, |index| ModelFacts {
        document_type: Some("Inspection Report".into()),
        type_evidence: vec![id_of(index, "INSPECTION REPORT")],
        ..ModelFacts::default()
    });
    outcome.proposal.description
}

fn heldout_assert_budget(description: &str) {
    assert!(
        description.split_whitespace().count() <= 42,
        "word budget exceeded: {description}"
    );
}

fn heldout_assert_quantity(description: &str, numbers: &[&str], complete: &str) {
    heldout_assert_budget(description);
    if numbers.iter().any(|number| description.contains(number)) {
        assert!(
            description.contains(complete),
            "quantity was partially restated; expected {complete:?} if mentioned: {description}"
        );
    }
}

#[test]
fn heldout_fidelity_field_keeps_pressure_unit_with_value() {
    let description = heldout_description_from_field(
        "replacement of high pressure transfer valve rated 85 kilopascals for chamber testing",
    );
    assert!(description.contains("valve"), "{description}");
    heldout_assert_quantity(&description, &["85"], "85 kilopascals");
}

#[test]
fn heldout_fidelity_field_preserves_entire_temperature_range() {
    let description = heldout_description_from_field(
        "replacement of cooling loop sensor rated for 15 to 35 degrees Celsius during operation",
    );
    assert!(description.contains("sensor"), "{description}");
    heldout_assert_quantity(&description, &["15", "35"], "15 to 35 degrees Celsius");
}

#[test]
fn heldout_fidelity_field_does_not_leave_partial_area_unit() {
    let description = heldout_description_from_field(
        "inspection of warehouse loading platform measuring 240 square meters before resurfacing",
    );
    assert!(description.contains("platform"), "{description}");
    heldout_assert_quantity(&description, &["240"], "240 square meters");
}

#[test]
fn heldout_fidelity_field_keeps_flow_denominator() {
    let description = heldout_description_from_field(
        "replacement coolant pump maximum flow rated 62 litres per minute for thermal conditioning",
    );
    assert!(description.contains("pump"), "{description}");
    heldout_assert_quantity(&description, &["62"], "62 litres per minute");
}

#[test]
fn heldout_fidelity_field_handles_unicode_decimal_digits_atomically() {
    let description = heldout_description_from_field(
        "replacement of laboratory chamber temperature sensor rated ٣٧ degrees Celsius for commissioning",
    );
    assert!(description.contains("sensor"), "{description}");
    heldout_assert_quantity(&description, &["٣٧"], "٣٧ degrees Celsius");
}

#[test]
fn heldout_fidelity_incomplete_source_does_not_supply_a_unit() {
    let description =
        heldout_description_from_field("spare valve assembly nominal setting recorded as 74");
    assert!(description.contains("valve"), "{description}");
    heldout_assert_budget(&description);
    for invented in [
        "74 psi",
        "74 kilopascals",
        "74 degrees",
        "74 meters",
        "74 litres",
        "74 percent",
        "74%",
    ] {
        assert!(
            !description.contains(invented),
            "fabricated unit {invented:?}: {description}"
        );
    }
}

fn heldout_description_from_long_subject(subject: &str) -> String {
    let source = format!("INSPECTION REPORT\n\nDate: October 3, 2026\n\n{subject}");
    let (outcome, _) = facts_for(&source, |index| ModelFacts {
        document_type: Some("Inspection Report".into()),
        type_evidence: vec![id_of(index, "INSPECTION REPORT")],
        subject: Some(subject.into()),
        subject_evidence: vec![id_of(index, "verification")],
        ..ModelFacts::default()
    });
    outcome.proposal.description
}

#[test]
fn heldout_fidelity_long_subject_drops_or_keeps_complete_pressure() {
    let description = heldout_description_from_long_subject(
        "verification of chamber operating pressure 180 kilopascals during repeated cold weather startup trials with independent monitoring of inlet seals outlet valves safety interlocks recording instruments mounting brackets electrical connectors emergency switches and backup power supplies before final release for routine laboratory use by the afternoon inspection team",
    );
    heldout_assert_quantity(&description, &["180"], "180 kilopascals");
}

#[test]
fn heldout_fidelity_long_subject_never_keeps_one_end_of_range() {
    let description = heldout_description_from_long_subject(
        "verification of chamber operating temperature 12 to 18 degrees Celsius during repeated cold weather startup trials with independent monitoring of inlet seals outlet valves safety interlocks recording instruments mounting brackets electrical connectors emergency switches and backup power supplies before final release for routine laboratory use by the afternoon inspection team",
    );
    heldout_assert_quantity(&description, &["12", "18"], "12 to 18 degrees Celsius");
}

#[test]
fn heldout_fidelity_two_invoice_issuers_are_not_customer_and_vendor() {
    let source = "INVOICE\n\nIssuer: Alder Current Ltd.\n\nIssuer: Blue Quarry LLC\n\nServices: joint equipment inspection\n\nBoth organizations issue this invoice jointly. No recipient is specified.";
    let (outcome, _) = facts_for(source, |index| ModelFacts {
        document_type: Some("Invoice".into()),
        type_evidence: vec![id_of(index, "INVOICE")],
        parties: vec![
            party(
                "Alder Current Ltd.",
                Some(PartyRole::Issuer),
                &[id_of(index, "Issuer: Alder")],
            ),
            party(
                "Blue Quarry LLC",
                Some(PartyRole::Issuer),
                &[id_of(index, "Issuer: Blue")],
            ),
        ],
        ..ModelFacts::default()
    });
    let description = &outcome.proposal.description;
    heldout_assert_budget(description);
    assert!(
        !description.contains(" to Alder Current") && !description.contains(" to Blue Quarry"),
        "a joint issuer became the recipient: {description}"
    );
}

#[test]
fn heldout_fidelity_two_memo_senders_are_not_each_others_addressee() {
    let source = "MEMORANDUM\n\nSender: Cobalt Field LLC\n\nSender: Dune Basin Ltd.\n\nProject: joint reservoir survey\n\nThis memorandum is jointly submitted by its two senders. Distribution will be determined separately.";
    let (outcome, _) = facts_for(source, |index| ModelFacts {
        document_type: Some("Memorandum".into()),
        type_evidence: vec![id_of(index, "MEMORANDUM")],
        parties: vec![
            party(
                "Cobalt Field LLC",
                Some(PartyRole::Sender),
                &[id_of(index, "Sender: Cobalt")],
            ),
            party(
                "Dune Basin Ltd.",
                Some(PartyRole::Sender),
                &[id_of(index, "Sender: Dune")],
            ),
        ],
        ..ModelFacts::default()
    });
    let description = &outcome.proposal.description;
    heldout_assert_budget(description);
    assert!(
        !description.contains(" to Cobalt Field") && !description.contains(" to Dune Basin"),
        "a co-sender became the recipient: {description}"
    );
}

#[test]
fn heldout_fidelity_letter_observer_is_not_invented_as_sender() {
    let source = "LETTER\n\nTo: Elm Horizon LLC\n\nThe public test was observed by Fjord Signal Ltd. The observer has no responsibility for drafting or sending this letter.\n\nProject: vibration monitoring";
    let (outcome, _) = facts_for(source, |index| ModelFacts {
        document_type: Some("Letter".into()),
        type_evidence: vec![id_of(index, "LETTER")],
        parties: vec![
            party(
                "Elm Horizon LLC",
                Some(PartyRole::Addressee),
                &[id_of(index, "To: Elm")],
            ),
            party(
                "Fjord Signal Ltd.",
                Some(PartyRole::Other),
                &[id_of(index, "observed by")],
            ),
        ],
        ..ModelFacts::default()
    });
    let description = &outcome.proposal.description;
    heldout_assert_budget(description);
    assert!(
        description.contains("to Elm Horizon LLC"),
        "true addressee lost: {description}"
    );
    assert!(
        !description.contains("from Fjord Signal"),
        "observer became the sender: {description}"
    );
}

#[test]
fn heldout_fidelity_notice_referenced_party_is_not_addressee() {
    let source = "NOTICE\n\nIssuer: Glen Lantern LLC\n\nThe safety handbook published by Haze Works Ltd. is cited for background only. This notice does not identify a recipient.\n\nEquipment: auxiliary lifting frame";
    let (outcome, _) = facts_for(source, |index| ModelFacts {
        document_type: Some("Notice".into()),
        type_evidence: vec![id_of(index, "NOTICE")],
        parties: vec![
            party(
                "Glen Lantern LLC",
                Some(PartyRole::Issuer),
                &[id_of(index, "Issuer:")],
            ),
            party(
                "Haze Works Ltd.",
                Some(PartyRole::Other),
                &[id_of(index, "handbook")],
            ),
        ],
        ..ModelFacts::default()
    });
    let description = &outcome.proposal.description;
    heldout_assert_budget(description);
    assert!(
        !description.contains("to Haze Works"),
        "reference publisher became a recipient: {description}"
    );
}

#[test]
fn heldout_fidelity_composer_does_not_use_unsupported_recipient_role() {
    // Composition receives a validated issuer and a named party whose
    // model-supplied recipient role the source did not establish.
    let relation = Relation {
        relation: PartyRelation::From,
        parties: vec!["Indigo Plain LLC".into()],
        basis: "source issuer label".into(),
    };
    let cast = vec![
        CastMember {
            name: "Indigo Plain LLC".into(),
            role: Some(PartyRole::Issuer),
            role_supported: true,
        },
        CastMember {
            name: "Juniper Crest Ltd.".into(),
            role: Some(PartyRole::Recipient),
            role_supported: false,
        },
    ];
    let description = describe(&DescriptionFacts {
        class: DocumentClass::Email,
        document_type: Some("Email"),
        relation: Some(&relation),
        parties: &cast,
        subject: Some("equipment survey arrangements"),
        ..DescriptionFacts::default()
    });
    heldout_assert_budget(&description);
    assert!(
        !description.contains("to Juniper Crest"),
        "unsupported role caused a recipient assertion: {description}"
    );
}

#[test]
fn heldout_fidelity_invoice_retains_stated_billing_direction() {
    let source = "INVOICE\n\nIssuer: Kestrel Vale LLC\n\nBill To: Linden Cove Ltd.\n\nServices: inspection of bridge cable anchors";
    let (outcome, _) = facts_for(source, |index| ModelFacts {
        document_type: Some("Invoice".into()),
        type_evidence: vec![id_of(index, "INVOICE")],
        parties: vec![
            party(
                "Kestrel Vale LLC",
                Some(PartyRole::Issuer),
                &[id_of(index, "Issuer:")],
            ),
            party(
                "Linden Cove Ltd.",
                Some(PartyRole::Customer),
                &[id_of(index, "Bill To:")],
            ),
        ],
        ..ModelFacts::default()
    });
    let description = &outcome.proposal.description;
    heldout_assert_budget(description);
    assert!(
        description.contains("from Kestrel Vale LLC")
            && description.contains("to Linden Cove Ltd."),
        "stated direction lost: {description}"
    );
    assert!(
        !description.contains("from Linden Cove") && !description.contains("to Kestrel Vale"),
        "stated direction inverted: {description}"
    );
}

#[test]
fn heldout_fidelity_letter_retains_explicit_sender_and_addressee() {
    let source = "LETTER\n\nFrom: Maple Flint LLC\n\nTo: North Aspen Ltd.\n\nProject: control cabinet inspection";
    let (outcome, _) = facts_for(source, |index| ModelFacts {
        document_type: Some("Letter".into()),
        type_evidence: vec![id_of(index, "LETTER")],
        parties: vec![
            party(
                "Maple Flint LLC",
                Some(PartyRole::Sender),
                &[id_of(index, "From:")],
            ),
            party(
                "North Aspen Ltd.",
                Some(PartyRole::Addressee),
                &[id_of(index, "To:")],
            ),
        ],
        ..ModelFacts::default()
    });
    let description = &outcome.proposal.description;
    heldout_assert_budget(description);
    assert!(
        description.contains("from Maple Flint LLC") && description.contains("to North Aspen Ltd."),
        "explicit sender/addressee lost: {description}"
    );
    assert!(
        !description.contains("from North Aspen") && !description.contains("to Maple Flint"),
        "explicit direction inverted: {description}"
    );
}

#[test]
fn heldout_fidelity_unlabelled_email_parties_allow_neutral_summary() {
    let source = "EMAIL\n\nThe survey discussed Oak Nimbus LLC and Pine Lumen Ltd. Their organizational roles and the message routing are not recorded.\n\nProject: archive ventilation inspection";
    let (outcome, _) = facts_for(source, |index| ModelFacts {
        document_type: Some("Email".into()),
        type_evidence: vec![id_of(index, "EMAIL")],
        parties: vec![
            party(
                "Oak Nimbus LLC",
                Some(PartyRole::Sender),
                &[id_of(index, "survey discussed")],
            ),
            party(
                "Pine Lumen Ltd.",
                Some(PartyRole::Recipient),
                &[id_of(index, "survey discussed")],
            ),
        ],
        ..ModelFacts::default()
    });
    let description = &outcome.proposal.description;
    heldout_assert_budget(description);
    for assertion in [
        "from Oak Nimbus",
        "from Pine Lumen",
        "to Oak Nimbus",
        "to Pine Lumen",
    ] {
        assert!(
            !description.contains(assertion),
            "unproved message routing: {description}"
        );
    }
}
