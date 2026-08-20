use moviebox_tui::tui::action::Action;
use moviebox_tui::tui::app::App;
use moviebox_tui::tui::overlay::update_modal_layout;
use moviebox_tui::updater::{Release, ReleaseAsset, TargetPlatform};
use ratatui::layout::Rect;

#[tokio::test]
async fn test_update_check_single_flight() {
    let mut app = App::new();
    assert!(!app.state().is_checking_updates);

    app.handle_action(Action::CheckForUpdates).await;
    assert!(app.state().is_checking_updates);

    app.handle_action(Action::CheckForUpdates).await;
    assert!(app.state().is_checking_updates);

    app.handle_action(Action::UpdateAvailable(Ok(None))).await;
    assert!(!app.state().is_checking_updates);
}

#[tokio::test]
async fn test_update_check_guard_clears_on_error() {
    let mut app = App::new();
    app.handle_action(Action::CheckForUpdates).await;
    assert!(app.state().is_checking_updates);

    app.handle_action(Action::UpdateAvailable(Err(
        "GitHub API rate limited (403)".to_string(),
    )))
    .await;
    assert!(!app.state().is_checking_updates);
}

#[tokio::test]
async fn test_update_check_guard_clears_on_success() {
    let mut app = App::new();
    app.handle_action(Action::CheckForUpdates).await;
    assert!(app.state().is_checking_updates);

    app.handle_action(Action::UpdateAvailable(Ok(Some((
        "0.1.13".to_string(),
        "Release notes content".to_string(),
    )))))
    .await;

    assert!(!app.state().is_checking_updates);
    assert_eq!(
        app.state().update_available,
        Some(("0.1.13".to_string(), "Release notes content".to_string()))
    );
}

#[tokio::test]
async fn test_update_modal_mouse_hitbox_matches_rendered_geometry() {
    let area = Rect::new(0, 0, 80, 24);
    let notes = "• Feature 1\n• Feature 2\n• Feature 3";
    let layout = update_modal_layout(area, notes);

    assert_eq!(layout.popup_area.width, 72);
    assert_eq!(layout.display_count, 3);
    assert!(!layout.has_more);
    assert_eq!(layout.popup_area.height, 11);

    assert_eq!(layout.button_row_y, layout.popup_area.y + 9);
    assert_eq!(layout.open_button_midpoint_x, layout.popup_area.x + 36);
}

#[tokio::test]
async fn test_update_modal_open_release_click() {
    let mut app = App::new();
    app.state_mut().update_available = Some((
        "0.1.13".to_string(),
        "### Notes\n• Major performance improvements".to_string(),
    ));

    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let area = Rect::new(0, 0, cols, rows);
    let layout = update_modal_layout(area, &app.state().update_available.as_ref().unwrap().1);

    let click_x = layout.popup_area.x + 5;
    let click_y = layout.button_row_y;

    app.handle_action(Action::MouseClick(click_x, click_y))
        .await;

    assert!(app.state().update_available.is_none());
}

#[tokio::test]
async fn test_update_modal_dismiss_click() {
    let mut app = App::new();
    app.state_mut().update_available = Some(("0.1.13".to_string(), "• Bug fix".to_string()));

    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let area = Rect::new(0, 0, cols, rows);
    let layout = update_modal_layout(area, &app.state().update_available.as_ref().unwrap().1);

    let click_x = layout.open_button_midpoint_x + 5;
    let click_y = layout.button_row_y;

    app.handle_action(Action::MouseClick(click_x, click_y))
        .await;

    assert!(app.state().update_available.is_none());
}

#[test]
fn test_update_asset_selection_for_current_platform() {
    let release = Release {
        version: "0.1.13".to_string(),
        tag_name: "v0.1.13".to_string(),
        notes: "Notes".to_string(),
        assets: vec![
            ReleaseAsset {
                name: "MovieBox_macOS_Universal.tar.gz".to_string(),
                download_url: "https://.../MovieBox_macOS_Universal.tar.gz".to_string(),
                size: Some(15000),
            },
            ReleaseAsset {
                name: "MovieBox_Linux_x64.tar.gz".to_string(),
                download_url: "https://.../MovieBox_Linux_x64.tar.gz".to_string(),
                size: Some(12000),
            },
            ReleaseAsset {
                name: "MovieBox_Linux_arm64.tar.gz".to_string(),
                download_url: "https://.../MovieBox_Linux_arm64.tar.gz".to_string(),
                size: Some(11000),
            },
            ReleaseAsset {
                name: "MovieBox_Windows_x64.zip".to_string(),
                download_url: "https://.../MovieBox_Windows_x64.zip".to_string(),
                size: Some(13000),
            },
            ReleaseAsset {
                name: "MovieBox_Windows_arm64.zip".to_string(),
                download_url: "https://.../MovieBox_Windows_arm64.zip".to_string(),
                size: Some(12000),
            },
            ReleaseAsset {
                name: "SHA256SUMS".to_string(),
                download_url: "https://.../SHA256SUMS".to_string(),
                size: Some(512),
            },
        ],
    };

    let mac = TargetPlatform::detect("macos", "arm64", false).unwrap();
    assert_eq!(
        release.find_compatible_asset(mac).unwrap().name,
        "MovieBox_macOS_Universal.tar.gz"
    );

    let linux = TargetPlatform::detect("linux", "x86_64", false).unwrap();
    assert_eq!(
        release.find_compatible_asset(linux).unwrap().name,
        "MovieBox_Linux_x64.tar.gz"
    );

    let win = TargetPlatform::detect("windows", "x86_64", false).unwrap();
    assert_eq!(
        release.find_compatible_asset(win).unwrap().name,
        "MovieBox_Windows_x64.zip"
    );
}

#[test]
fn test_update_asset_missing_for_current_platform() {
    let release = Release {
        version: "0.1.13".to_string(),
        tag_name: "v0.1.13".to_string(),
        notes: "Notes".to_string(),
        assets: vec![ReleaseAsset {
            name: "MovieBox_Linux_x64.tar.gz".to_string(),
            download_url: "https://...".to_string(),
            size: Some(1000),
        }],
    };

    let mac = TargetPlatform::detect("macos", "arm64", false).unwrap();
    assert!(release.find_compatible_asset(mac).is_none());
}

#[test]
fn test_update_asset_rejects_wrong_architecture() {
    assert!(TargetPlatform::detect("linux", "ppc64", false).is_none());
    assert!(TargetPlatform::detect("windows", "arm", false).is_none());
}

#[test]
fn test_update_asset_rejects_wrong_platform() {
    assert!(TargetPlatform::detect("solaris", "x86_64", false).is_none());
    assert!(TargetPlatform::detect("netbsd", "x86_64", false).is_none());
}
