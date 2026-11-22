//! A backup from another computer (spec 42.10): a crew's work folder chosen
//! there that is no path here goes back to the crew's own folder; one that
//! is a path here stays chosen, even while it is missing.

mod common;

use botloft_core::ids::CrewId;
use botloftd::service::moved::forget_foreign_folders;
use common::TestDaemon;
use serde_json::json;

#[tokio::test]
async fn a_folder_from_another_system_goes_back_to_the_crews_own() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let chosen = tempfile::tempdir().expect("dir");
    let crew = app
        .call(
            "crews.create",
            json!({ "name": "Ops", "workFolder": chosen.path() }),
        )
        .await
        .expect("crew");
    let id: CrewId = crew["id"].as_str().expect("id").parse().expect("crew id");
    assert_eq!(forget_foreign_folders(&t.daemon).expect("checked"), 0);

    // Missing, but a path here: maybe a drive not plugged in.
    let missing = chosen.path().join("gone");
    let foreign = if cfg!(windows) {
        "/srv/site"
    } else {
        r"C:\Users\owner\site"
    };
    for (folder, moved) in [
        (missing.to_string_lossy().into_owned(), 0),
        (foreign.into(), 1),
    ] {
        {
            let store = t.daemon.store();
            let mut crew = store.crew(&id).expect("read").expect("crew");
            crew.work_folder = folder;
            store.update_crew(&crew).expect("as a backup leaves it");
        }
        assert_eq!(forget_foreign_folders(&t.daemon).expect("checked"), moved);
    }
    let crew = t.daemon.store().crew(&id).expect("read").expect("crew");
    assert!(!crew.work_folder_chosen, "back to its own folder");
    assert!(t.paths.work_folder(&crew).is_dir(), "which is ready");
}
