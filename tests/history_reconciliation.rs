mod common;

use common::{TempTestDir, make_history_item};
use moviebox_tui::history::{HistoryManager, PendingPlaybackState};
use std::fs;

#[test]
fn test_history_identity_rules() {
    let movie1 = make_history_item("moviebox", "mb_10", "Dune", 1, "2021", 0, 0);
    let movie2 = make_history_item("moviebox", "mb_10", "Dune: Part One", 1, "2021", 0, 0);
    assert!(HistoryManager::is_same_show(&movie1, &movie2));

    let series = make_history_item("moviebox", "mb_20", "Dune: Prophecy", 2, "2024", 1, 1);
    assert!(!HistoryManager::is_same_show(&movie1, &series));

    let addon_movie = make_history_item("addons", "tt1160419", "Dune", 1, "2021", 0, 0);
    assert!(!HistoryManager::is_same_show(&movie1, &addon_movie));

    let remake1978 = make_history_item("moviebox", "mb_h1", "Halloween", 1, "1978", 0, 0);
    let remake2018 = make_history_item("moviebox", "mb_h2", "Halloween", 1, "2018", 0, 0);
    assert!(!HistoryManager::is_same_show(&remake1978, &remake2018));
}

#[test]
fn test_history_reconciliation_multi_episode_flow() {
    let temp_dir = TempTestDir::new("hist_reconcile");
    let state_file_ep1 = temp_dir.path.join("moviebox_show1_1_1.json");
    let state_file_ep2 = temp_dir.path.join("moviebox_show1_1_2.json");

    let ep1_state = PendingPlaybackState {
        provider: "moviebox".to_string(),
        subject_id: "show1".to_string(),
        season: 1,
        episode: 1,
        progress_seconds: 3600,
        duration_seconds: Some(3600),
        completed: true,
        timestamp: 2000,
    };
    fs::write(&state_file_ep1, serde_json::to_string(&ep1_state).unwrap()).unwrap();

    let ep2_state = PendingPlaybackState {
        provider: "moviebox".to_string(),
        subject_id: "show1".to_string(),
        season: 1,
        episode: 2,
        progress_seconds: 1200,
        duration_seconds: Some(3600),
        completed: false,
        timestamp: 2500,
    };
    fs::write(&state_file_ep2, serde_json::to_string(&ep2_state).unwrap()).unwrap();

    let mut manager = HistoryManager::default();
    manager.recent.push(make_history_item(
        "moviebox",
        "show1",
        "Epic Series",
        2,
        "2024",
        1,
        2,
    ));

    let modified = manager.reconcile_from_dir(&temp_dir.path);
    assert!(modified);
    assert!(manager.is_watched("moviebox", "show1", 1, 1));
    assert_eq!(manager.recent.first().unwrap().episode, 2);
    assert_eq!(manager.recent.first().unwrap().progress_seconds, 1200);

    assert!(!state_file_ep1.exists());
    assert!(!state_file_ep2.exists());
}
