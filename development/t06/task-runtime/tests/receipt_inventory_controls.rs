//! Exact-reference inventory controls; no workload execution or verdict creation.
use super::*;
use habitat_engine::store::ArtifactStaging;
use std::fs;
use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;
use std::time::Duration;
const MANIFEST: &[u8] = b"[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n";
const SOURCE: &[u8] = b"pub fn repaired() -> bool { true }\n";
struct Area(PathBuf);
impl Area {
    fn new(deadline: Instant) -> Self {
        let id = habitat_engine::app::evidence::fresh_id(deadline).unwrap();
        let p = std::env::temp_dir().join(format!("hee3-receipt-inventory-{}", id.as_str()));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        Self(p)
    }
}
impl Drop for Area {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn fixture(patch: &[u8], check: impl FnOnce(&Evidence<'_>, r::SubjectsV1, Instant)) {
    let deadline = Instant::now() + Duration::from_secs(20);
    let area = Area::new(deadline);
    let source = area.0.join("source");
    let staging = area.0.join("staging");
    for p in [&source, &staging] {
        fs::DirBuilder::new().mode(0o700).create(p).unwrap();
    }
    fs::create_dir(source.join("src")).unwrap();
    fs::write(source.join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(source.join("src/lib.rs"), SOURCE).unwrap();
    let snapshot =
        habitat_engine::worker::workspace::Snapshot::capture(&source, &[], deadline).unwrap();
    let staged = ArtifactStaging::open(&staging, true, deadline).unwrap();
    let mut evidence = Evidence::staged(&staged, deadline);
    let result = subjects::publish(
        &mut evidence,
        &snapshot,
        r::SubjectFileV1Origin::Authored,
        deadline,
    )
    .unwrap();
    let patch = evidence.payload(patch, "text/x-diff").unwrap();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/receipt-import/preparation.json"
    ))
    .unwrap();
    let mut subjects: r::SubjectsV1 = serde_json::from_value(fixture["subjects"].clone()).unwrap();
    subjects.result_subject = r::Maybe::present(result);
    subjects.seed_to_result_patch = r::Maybe::present(patch);
    check(&evidence, subjects, deadline);
}
#[test]
fn repaired_inventory_preserves_patch_and_all_exact_result_content_refs() {
    fixture(
        b"independently specified repair patch",
        |evidence, subjects, deadline| {
            let rows = subject_artifacts(evidence, &subjects, deadline).unwrap();
            assert_eq!(rows.len(), 3);
            assert_eq!(
                rows[0].object,
                *subjects
                    .seed_to_result_patch
                    .value
                    .as_ref()
                    .unwrap()
                    .as_ref()
            );
            for (expected, sha) in [
                (
                    MANIFEST,
                    "sha256:97af2cdd4b15f427e0c7a82bb39763dd1af776e42a280162d7d76c8874715854",
                ),
                (
                    SOURCE,
                    "sha256:8f1a26b50b594eda477261ff2a13bc0780c2539117af22da2c00c6ab762e422f",
                ),
            ] {
                let matches: Vec<_> = rows
                    .iter()
                    .filter(|r| r.object.sha256.as_str() == sha)
                    .collect();
                assert_eq!(matches.len(), 1);
                assert_eq!(bytes(evidence, &matches[0].object).unwrap(), expected);
            }
            assert!(rows.iter().all(|row| row.required
                && !row.truncated
                && row.availability == r::ArtifactV1Availability::Available));
        },
    );
}
#[test]
fn baseline_empty_patch_keeps_its_exact_identity_without_inventing_an_output() {
    fixture(b"", |evidence, subjects, deadline| {
        let rows = subject_artifacts(evidence, &subjects, deadline).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows[0].object,
            *subjects
                .seed_to_result_patch
                .value
                .as_ref()
                .unwrap()
                .as_ref()
        );
        assert_eq!(rows[0].object.byte_length, 0);
        assert!(bytes(evidence, &rows[0].object).unwrap().is_empty());
    });
}
#[test]
fn absent_result_or_patch_cannot_create_a_finalized_inventory() {
    fixture(b"patch", |evidence, subjects, deadline| {
        let mut absent = subjects.clone();
        absent.result_subject.value = None;
        assert!(matches!(
            subject_artifacts(evidence, &absent, deadline)
                .unwrap_err()
                .kind,
            ErrorKind::Binding
        ));
        let mut absent = subjects;
        absent.seed_to_result_patch.value = None;
        assert!(matches!(
            subject_artifacts(evidence, &absent, deadline)
                .unwrap_err()
                .kind,
            ErrorKind::Binding
        ));
    });
}
#[test]
fn foreign_same_digest_patch_identity_is_not_replaced_by_matching_bytes() {
    fixture(b"patch", |evidence, mut subjects, deadline| {
        let mut reference =
            serde_json::to_value(subjects.seed_to_result_patch.value.as_ref().unwrap()).unwrap();
        reference["artifact_id"] = json!("92000000-0000-4000-8000-000000000001");
        subjects.seed_to_result_patch.value = Some(serde_json::from_value(reference).unwrap());
        assert!(matches!(
            subject_artifacts(evidence, &subjects, deadline)
                .unwrap_err()
                .kind,
            ErrorKind::Graph(_)
        ));
    });
}
#[test]
fn expired_inventory_deadline_refuses_without_publication() {
    fixture(b"patch", |evidence, subjects, _| {
        assert!(matches!(
            subject_artifacts(evidence, &subjects, Instant::now())
                .unwrap_err()
                .kind,
            ErrorKind::Deadline
        ));
    });
}
