use chrono::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadInfo {
    meta: Downloads,
    pub version_downloads: Vec<VersionDownload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Downloads {
    extra_downloads: Vec<BasicDownload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicDownload {
    date: NaiveDate,
    downloads: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDownload {
    pub date: DateTime<UTC>,
    pub downloads: i64,
    pub id: i32,
    pub version: i32,
}
