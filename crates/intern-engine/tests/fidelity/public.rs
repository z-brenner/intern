// Public regressions derived from the published Windows qualification failures.
#[test]
fn fidelity_public_premises_keeps_the_complete_quantity() {
    let index = index_of("OFFICE LEASE\n\nLandlord: Ash LLC\n\nTenant: Elm Inc.\n\nPremises: Suite 400, fourth floor, approximately 14,860 rentable square feet\n");
    let context = whole(&index);
    let facts = ModelFacts {
        document_type: Some("Office Lease".into()),
        type_evidence: vec![id_of(&index, "OFFICE LEASE")],
        parties: vec![
            party("Ash LLC", Some(PartyRole::Landlord), &[id_of(&index, "Landlord:")]),
            party("Elm Inc.", Some(PartyRole::Tenant), &[id_of(&index, "Tenant:")]),
        ],
        ..ModelFacts::default()
    };
    let outcome = check(facts, &context, &index);
    assert!(outcome.proposal.description.contains("14,860 rentable square feet"), "{}", outcome.proposal.description);
    assert!(outcome.proposal.description.split_whitespace().count() <= crate::validate::MAX_DESCRIPTION_WORDS);
}
