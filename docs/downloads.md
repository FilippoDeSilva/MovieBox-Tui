# Downloads

The download engine in `download.rs` streams a video URL to disk with resume, ranges,
and optional segmentation. Orchestration lives in `app/download.rs`.

## How it works

- Single-episode downloads write to `<dest>.part` plus a `<dest>.part.json` metadata
  sidecar (etag, last-modified, total size, segment count).
- **Resume**: on a retry, the engine checks what is already in the `.part` file and
  continues from there using `Range` requests.
- **Segmentation**: files above a size threshold can be downloaded in parallel
  segments (up to a capped count), then stitched.
- **Retries**: a failed attempt is retried a limited number of times; 30s idle
  timeouts apply to streaming reads.
- **Cancel**: an `AtomicBool` cancel flag pauses/resumes cleanly, preserving the
  partial file for a later resume.

## File names

`safe_file_stem` sanitizes titles for all platforms: control/whitespace/illegal
characters are replaced, Windows reserved names (`CON`, `COM1`-`COM9`, …) are avoided,
and length is capped. Files go to the user's download directory. On Android-family
environments the code prefers shared `storage/downloads` when present.

## Seasons

A season download enqueues every episode (`download_queue`) and processes them one at a
time, each resolving its stream and subtitle. Progress is reported through
`Action::UpdateDownload` and the status bar; failures pause and preserve partial data.

## Outcomes

`DownloadCompleted` / `DownloadPaused` / `DownloadFailed` drive the UI status and
notifications. `ClearCache` and stale-file cleanup do not touch in-progress downloads.
