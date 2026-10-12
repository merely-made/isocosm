// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "native_founding/fixture.rs"]
mod fixture;
#[path = "native_setup/fixture.rs"]
mod setup_fixture;
use isocosm::{Command, Execution, site_authoring::Attachment};
use isometry_campaign::founding::{Campaign, Setup};

#[test]
fn named_setup_attaches_native_sites_retains_table_metadata_and_replays_both_modes() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let setup = setup_fixture::setup(mode);
        let bytes = serde_json::to_vec(&setup).unwrap();
        assert_eq!(Setup::decode(&bytes).unwrap(), setup);
        let record = fixture::record();
        let prepared = Campaign::from_setup(&setup, &record, 16).unwrap();
        let campaign = prepared.campaign;
        let reading = campaign.reading();
        assert_eq!(reading.authored_site("ford").unwrap().0, 0, "configured existing site was ignored");
        assert_eq!(reading.authored_site("shrine").unwrap().0, 1000, "configured new site was ignored");
        assert!(reading.state().sites[&0].routes.iter().any(|r| r.border.is_some()),
            "attaching an authored label discarded native topology");
        assert_eq!(reading.authored_entity("mira").unwrap().1.authored.as_ref().unwrap().table.as_ref().unwrap().cell,
            [-4, 2, 11], "table cell metadata disappeared");
        assert!(reading.entries().iter().any(|e| matches!(&e.command,
            Command::AttachSite { attachment: Attachment::Existing(0), .. })),
            "existing attachment was not native history");
        let saved = campaign.save();
        for replay in [Execution::Individuals, Execution::Grouped] {
            assert_eq!(Campaign::load(saved.clone(), replay).unwrap().save(), saved);
        }
        assert_eq!(campaign.origin().unwrap().entropy, record.entropy);
        assert_ne!(campaign.reading().genesis().seed, record.entropy);
        assert_eq!(prepared.private.secret("secret-oath").unwrap().text, "HOST PRIVATE OATH");
    }
}

#[test]
fn setup_refuses_missing_or_ambiguous_attachments_and_retains_native_phase() {
    let record = fixture::record();
    let mut setup = setup_fixture::setup(Execution::Grouped);
    setup.attachments.remove("shrine");
    let error = Campaign::from_setup(&setup, &record, 16).unwrap_err().to_string();
    assert!(error.contains("attach every authored place exactly once"), "missing attachment was ignored");
    let mut setup = setup_fixture::setup(Execution::Grouped);
    setup.attachments.insert("shrine".into(), Attachment::Existing(0));
    let error = Campaign::from_setup(&setup, &record, 16).unwrap_err().to_string();
    assert!(error.contains("Assertion") && error.contains("another authored place"),
        "late native attachment refusal lost its assertion index: {error}");
    let mut setup = setup_fixture::setup(Execution::Grouped);
    setup.name.clear();
    assert!(Campaign::from_setup(&setup, &record, 16).unwrap_err().to_string().contains("has no name"));
}

#[test]
fn attachment_file_override_is_validated_as_the_exact_selected_configuration() {
    let mut setup = setup_fixture::setup(Execution::Individuals);
    let attachments = serde_json::to_vec(&setup.attachments).unwrap();
    setup.attachments.clear(); setup.replace_attachments(&attachments).unwrap();
    let prepared = Campaign::from_setup(&setup, &fixture::record(), 16).unwrap();
    assert_eq!(prepared.campaign.reading().authored_site("shrine").unwrap().0, 1000);
    let before = setup.clone();
    assert!(setup.replace_attachments(b"{bad").is_err());
    assert_eq!(setup, before, "malformed attachment override replaced the selected configuration");
}
