use log::{debug, error, info};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

/// MusicBrainz API response
#[derive(Deserialize, Debug)]
struct MusicBrainzResponse {
    #[serde(rename = "release-groups")]
    release_groups: Vec<ReleaseGroup>,
}

#[derive(Deserialize, Debug)]
struct ReleaseGroup {
    id: String,
}

/// Cover Art Archive response
#[derive(Deserialize)]
struct CoverArtResponse {
    images: Vec<CoverImage>,
}

#[derive(Deserialize)]
struct CoverImage {
    image: String,
}

pub fn get_mbid(artist: &str, album: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://musicbrainz.org/ws/2/release-group/?query=artist:{} AND release:{}&fmt=json",
        artist, album
    );
    debug!("Fetching MBID: {}", url);

    let client = Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "cmus-discord-rpc/1.0 ( https://github.com/inevitabby/cmus-discord-rpc )")
        .send()?;
    
    let text = response.text()?;
    debug!("MusicBrainz response: {}", text);

    let response: MusicBrainzResponse = serde_json::from_str(&text)?;

    for release in &response.release_groups {
        debug!("Found release group: ID={} Title={}", release.id, album);
    }

    if let Some(release) = response.release_groups.first() {
        info!("Using MBID: {}", release.id);
        Ok(release.id.clone())
    } else {
        error!("MBID not found for {} - {}", artist, album);
        Err("MBID not found".into())
    }
}

pub fn get_album_art(mbid: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://coverartarchive.org/release-group/{}", mbid);
    debug!("Fetching cover art: {}", url);

    let client = Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "cmus-discord-rpc/1.0 ( https://github.com/inevitabby/cmus-discord-rpc )")
        .send()?;
    let text = response.text()?;  
    debug!("Cover Art Archive response: {}", text);

    let response: CoverArtResponse = serde_json::from_str(&text)?;

    if let Some(cover) = response.images.first() {
        info!("Found album art: {}", cover.image);
        Ok(cover.image.clone())
    } else {
        error!("No album art found for MBID: {}", mbid);
        Err("No album art found".into())
    }
}

pub fn fetch_album_art(artist: &str, album: &str) -> Option<String> {
    match get_mbid(artist, album) {
        Ok(mbid) => match get_album_art(&mbid) {
            Ok(url) => Some(url),
            Err(_) => None,
        },
        Err(_) => None,
    }
}


