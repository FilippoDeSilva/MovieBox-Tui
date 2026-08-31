use moviebox_tui::providers::moviebox::client::MovieBoxClient;

#[tokio::test]
#[ignore = "live network test; run with cargo test --test live_stream_verification -- --ignored"]
async fn test_live_movie_stream_real_urls() {
    let client = MovieBoxClient::new();
    client.init().await.expect("client init successful");

    // Avatar subject_id: 1654274595068805784
    let (items, _) = client
        .fetch_resource_page("1654274595068805784", 0, 1)
        .await
        .expect("fetch resource page");
    assert!(!items.is_empty(), "items should not be empty");

    let notice_hash = "1c7de0bd3393702d9191801f15f88f8d";
    let mut real_streams_found = 0;

    for item in &items {
        if let Some(link) = item.get("resourceLink").and_then(|l| l.as_str()) {
            println!("Movie stream link: {}", link);
            assert!(
                !link.contains(notice_hash),
                "Stream URL should NOT be the legacy upgrade notice video: {}",
                link
            );
            assert!(
                link.contains("bcdn.hakunaymatata.com") || link.contains("sign="),
                "Stream URL should be a signed CDN link: {}",
                link
            );
            real_streams_found += 1;
        }
    }

    assert!(
        real_streams_found > 0,
        "Should find at least one real movie stream"
    );
}

#[tokio::test]
#[ignore = "live network test; run with cargo test --test live_stream_verification -- --ignored"]
async fn test_live_series_resolutions_and_streams() {
    let client = MovieBoxClient::new();
    client.init().await.expect("client init successful");

    // Loki subject_id: 8449792878959887920
    let resolutions = client
        .fetch_collection_resolutions("8449792878959887920")
        .await
        .expect("fetch collection resolutions");

    println!("Series resolutions: {:?}", resolutions);
    assert!(!resolutions.is_empty(), "resolutions should not be empty");
    assert!(
        resolutions.contains(&1080) || resolutions.contains(&720) || resolutions.contains(&480),
        "resolutions should contain standard qualities: {:?}",
        resolutions
    );

    // Fetch season 1 episode 1
    let res = client
        .get_resources("8449792878959887920", 1, 1, 1, None, 20)
        .await
        .expect("fetch episode resources");

    let items = res
        .get("list")
        .and_then(|l| l.as_array())
        .cloned()
        .unwrap_or_default();
    let notice_hash = "1c7de0bd3393702d9191801f15f88f8d";
    let mut episode_streams_found = 0;

    for item in &items {
        if let Some(link) = item.get("resourceLink").and_then(|l| l.as_str()) {
            println!("Episode stream link: {}", link);
            assert!(
                !link.contains(notice_hash),
                "Episode stream URL should NOT be the legacy upgrade notice video: {}",
                link
            );
            episode_streams_found += 1;
        }
    }

    assert!(
        episode_streams_found > 0,
        "Should find at least one real episode stream"
    );
}

#[tokio::test]
#[ignore = "live network test; run with cargo test --test live_stream_verification -- --ignored"]
async fn test_live_fourkhdhub_movie_resolution() {
    let client = moviebox_tui::providers::fourkhdhub::FourKHdHubClient::new()
        .expect("fourkhdhub client creation");
    let items = client.search("Inception").await.expect("search Inception");
    assert!(!items.is_empty(), "Inception search should return results");

    let target = &items[0];
    println!("Found item: {} ({:?})", target.title, target.id);
    let releases = client
        .releases(&target.id.value, 0, 0)
        .await
        .expect("fetch movie releases");
    assert!(!releases.is_empty(), "releases should not be empty");

    for release in &releases {
        println!("Attempting resolve for: {}", release.filename);
        let start = std::time::Instant::now();
        match client.resolve_release(release).await {
            Ok(source) => {
                println!(
                    "Resolved in {:?}: {} [{}]",
                    start.elapsed(),
                    source.url,
                    source.source_label
                );
                assert!(source.url.starts_with("https://"));
                break;
            }
            Err(e) => {
                println!("Failed fast in {:?}: {e}", start.elapsed());
                assert!(start.elapsed() < std::time::Duration::from_secs(8));
            }
        }
    }
}

#[tokio::test]
#[ignore = "live network test; run with cargo test --test live_stream_verification -- --ignored"]
async fn test_live_fourkhdhub_game_of_thrones_resolution() {
    let client = moviebox_tui::providers::fourkhdhub::FourKHdHubClient::new()
        .expect("fourkhdhub client creation");
    let items = client
        .search("Game of Thrones")
        .await
        .expect("search Game of Thrones");
    assert!(!items.is_empty(), "Game of Thrones search results");

    let target = items
        .iter()
        .find(|item| item.title.to_lowercase().contains("thrones"))
        .expect("Game of Thrones entry");
    println!("Found series: {} ({:?})", target.title, target.id);

    let releases = client
        .releases(&target.id.value, 1, 1)
        .await
        .expect("releases for S01E01");
    assert!(!releases.is_empty(), "S01E01 releases found");

    for release in &releases {
        println!(
            "Attempting resolve for S01E01 release: {}",
            release.filename
        );
        let start = std::time::Instant::now();
        match client.resolve_release(release).await {
            Ok(source) => {
                println!(
                    "SUCCESS in {:?}: {} [{}]",
                    start.elapsed(),
                    source.url,
                    source.source_label
                );
                assert!(source.url.starts_with("https://"));
                break;
            }
            Err(e) => {
                println!("Failed fast in {:?}: {e}", start.elapsed());
                assert!(start.elapsed() < std::time::Duration::from_secs(8));
            }
        }
    }
}
