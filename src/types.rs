use serde::Deserialize;

// -----------------------------------------------------
// See https://lastkajen2-p.ea.trafikverket.se/assets/Lastkajen2_API_Information.pdf for more
// information.

// Bearer token for the Swedish Transport Administration's Lastkajen API.
#[derive(Debug, Clone, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub expires_in: usize,
    pub is_external: bool,
}

// -----------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub enum PackageType {
    Published,
    User
}   

// -----------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct TargetFolder {
    pub id: usize,
    pub name: String,
    pub path: String
}

/// Published Data Package
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPackageFolder {
    pub id: usize,
    pub target_folder: TargetFolder,
    pub source_folder: String,
    pub name: String,
    pub description: String,
    pub published: bool
}

// -----------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileLink {
    pub href: String,
    pub rel: String,
    pub method: String,
    pub is_templated: bool
}

/// File from a published Data Package.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPackageFile {
    pub is_folder: bool,
    pub name: String,
    pub size: String,

    #[cfg(all(feature = "time"))]
    #[serde(with = "time::serde::iso8601")]
    pub date_time: time::OffsetDateTime,

    #[cfg(not(feature = "time"))]
    pub date_time: String,

    pub links: Vec<FileLink>
}

// -----------------------------------------------------

/// User order information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFile {
    pub is_folder: bool,
    pub name: String,
    pub size: String,

    // TODO: No time offset (ie timezone) is specified by the api result, figure 
    // out workaround or a "good guess."
    pub date_time: String
}

// -----------------------------------------------------


pub enum DownloadCategory<'a> {
    Published {
        id: &'a usize,
        file: &'a String
    },
    User {
        file: &'a String
    }
}

pub enum DownloadToken {
    Published(String),
    User(String)
}
