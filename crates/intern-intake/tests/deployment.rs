use intern_intake::SharePointDeployment;

// These GUIDs are synthetic test data. They are not tenant, application, or
// SharePoint identifiers and cannot authenticate against Microsoft services.
const VALID_DEPLOYMENT: &str = r#"
{
  "schema_version": 1,
  "enabled": true,
  "site_url": "https://teamcontoso.sharepoint.com/sites/InternTestSite",
  "library_name": "Files",
  "intake_folder_name": "Inbox",
  "destination_folder_name": "Filed",
  "tenant_id": "11111111-1111-1111-1111-111111111111",
  "client_id": "22222222-2222-2222-2222-222222222222",
  "site_id": "33333333-3333-3333-3333-333333333333",
  "web_id": "44444444-4444-4444-4444-444444444444",
  "list_id": "55555555-5555-5555-5555-555555555555",
  "drive_id": "66666666-6666-6666-6666-666666666666",
  "intake_folder_id": "77777777-7777-7777-7777-777777777777",
  "destination_folder_id": "88888888-8888-8888-8888-888888888888"
}
"#;

#[test]
fn accepts_the_fixed_site_library_and_folders_and_encodes_odopen_values() {
    let deployment = SharePointDeployment::from_slice(VALID_DEPLOYMENT.as_bytes())
        .expect("the documented fixed deployment should be valid");

    assert_eq!(deployment.web_id(), "44444444-4444-4444-4444-444444444444");
    assert_eq!(
        deployment
            .odopen_url("ada+intern@example.com")
            .unwrap()
            .as_str(),
        "odopen://sync/?siteId=%7B33333333-3333-3333-3333-333333333333%7D&webId=%7B44444444-4444-4444-4444-444444444444%7D&listId=%7B55555555-5555-5555-5555-555555555555%7D&userEmail=ada%2Bintern%40example.com&webUrl=https%3A%2F%2Fteamcontoso.sharepoint.com%2Fsites%2FInternTestSite&listTitle=Files"
    );
}

#[test]
fn rejects_lookalike_hosts_and_paths_outside_the_fixed_site() {
    let deployment = SharePointDeployment::from_slice(VALID_DEPLOYMENT.as_bytes()).unwrap();
    let lookalike = "https://teamcontoso.sharepoint.com.evil.example/sites/InternTestSite"
        .parse()
        .unwrap();
    let other_site = "https://teamcontoso.sharepoint.com/sites/OtherSite/Inbox"
        .parse()
        .unwrap();

    assert!(!deployment.contains_web_url(&lookalike));
    assert!(!deployment.contains_web_url(&other_site));
}

#[test]
fn confines_web_urls_to_the_site_path_boundary_and_rejects_traversal() {
    let deployment = SharePointDeployment::from_slice(VALID_DEPLOYMENT.as_bytes()).unwrap();
    let contained = "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox"
        .parse()
        .unwrap();
    let prefix_collision = "https://teamcontoso.sharepoint.com/sites/InternTestSiteBackup"
        .parse()
        .unwrap();
    let traversal = "https://teamcontoso.sharepoint.com/sites/InternTestSite/%2E%2E/OtherSite"
        .parse()
        .unwrap();

    assert!(deployment.contains_web_url(&contained));
    assert!(!deployment.contains_web_url(&prefix_collision));
    assert!(!deployment.contains_web_url(&traversal));
}

#[test]
fn the_fixed_inbox_url_does_not_accept_a_sibling_folder_item_or_query() {
    let deployment = SharePointDeployment::from_slice(VALID_DEPLOYMENT.as_bytes()).unwrap();

    assert!(deployment.is_intake_folder_web_url(
        "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox"
    ));
    for address in [
        "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Filed",
        "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox/agreement.pdf",
        "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox?download=1",
    ] {
        assert!(!deployment.is_intake_folder_web_url(address), "{address}");
    }
}

#[test]
fn rejects_non_simple_library_and_folder_components() {
    for replacement in [
        (
            "\"library_name\": \"Files\"",
            "\"library_name\": \"../Files\"",
        ),
        (
            "\"intake_folder_name\": \"Inbox\"",
            "\"intake_folder_name\": \"In/box\"",
        ),
        (
            "\"destination_folder_name\": \"Filed\"",
            "\"destination_folder_name\": \".\"",
        ),
    ] {
        let invalid = VALID_DEPLOYMENT.replace(replacement.0, replacement.1);
        assert!(SharePointDeployment::from_slice(invalid.as_bytes()).is_err());
    }
}

#[test]
fn rejects_schema_versions_other_than_one() {
    let invalid = VALID_DEPLOYMENT.replace("\"schema_version\": 1", "\"schema_version\": 2");

    assert!(SharePointDeployment::from_slice(invalid.as_bytes()).is_err());
}

#[test]
fn rejects_missing_and_non_guid_public_identifiers() {
    for field in [
        "tenant_id",
        "client_id",
        "site_id",
        "web_id",
        "list_id",
        "drive_id",
        "intake_folder_id",
        "destination_folder_id",
    ] {
        let mut missing: serde_json::Value = serde_json::from_str(VALID_DEPLOYMENT).unwrap();
        missing[field] = serde_json::Value::Null;
        assert!(SharePointDeployment::from_slice(missing.to_string().as_bytes()).is_err());

        let mut invalid: serde_json::Value = serde_json::from_str(VALID_DEPLOYMENT).unwrap();
        invalid[field] = serde_json::Value::String("not-a-guid".to_string());
        assert!(SharePointDeployment::from_slice(invalid.to_string().as_bytes()).is_err());
    }
}

#[test]
fn rejects_secrets_passwords_and_tokens_in_the_public_resource() {
    for field in ["client_secret", "password", "access_token"] {
        let mut invalid: serde_json::Value = serde_json::from_str(VALID_DEPLOYMENT).unwrap();
        invalid[field] = serde_json::Value::String("must-not-be-packaged".to_string());
        let error = SharePointDeployment::from_slice(invalid.to_string().as_bytes()).unwrap_err();
        assert!(error.to_string().contains("secret, password, or token"));
    }
}

#[test]
fn rejects_invalid_emails_for_onedrive_sync() {
    let deployment = SharePointDeployment::from_slice(VALID_DEPLOYMENT.as_bytes()).unwrap();

    for email in ["", "ada", "ada@example", "ada @example.com"] {
        assert!(
            deployment.odopen_url(email).is_err(),
            "{email:?} must be rejected"
        );
    }
}

#[test]
fn disabled_bundled_configuration_reports_that_identifiers_are_unavailable() {
    let error = SharePointDeployment::from_slice(include_bytes!(
        "../../../src-tauri/resources/sharepoint-deployment.json"
    ))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "SharePoint deployment configuration is unavailable: provisioned identifiers are not available in this build."
    );
}

#[test]
fn a_disabled_resource_still_validates_its_fixed_site_boundary() {
    let disabled = include_str!("../../../src-tauri/resources/sharepoint-deployment.json").replace(
        "teamcontoso.sharepoint.com",
        "teamcontoso.sharepoint.com.evil.example",
    );
    let error = SharePointDeployment::from_slice(disabled.as_bytes()).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("site_url must be the fixed HTTPS site")
    );
}
