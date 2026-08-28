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
