// Invoked by the Studio editor harness; ordinary gates do not fabricate client
// execution when no external payload directory is supplied.
#[test]
fn original_map_actual_editor_lifecycle() {
    crate::tests::support::client::injured_original::run();
}
