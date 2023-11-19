use std::fmt;
use std::io::Write;

pub mod types;

// -----------------------------------------------------
// See https://lastkajen2-p.ea.trafikverket.se/assets/Lastkajen2_API_Information.pdf for more
// information.

#[derive(Debug)]
pub enum LastkajenError {
    ReqwestError(reqwest::Error),
    IoError(std::io::Error),
    StatusError(reqwest::StatusCode),
    LastkajenError(String),
}

impl From<reqwest::Error> for LastkajenError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error)
    }
}

impl From<std::io::Error> for LastkajenError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<reqwest::StatusCode> for LastkajenError {
    fn from(status: reqwest::StatusCode) -> Self {
        Self::StatusError(status)
    }
}

impl fmt::Display for LastkajenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReqwestError(err) => write!(f, "reqwest::Error: {}", err),
            Self::StatusError(code) => write!(f, "Api Request Error: HTTP status {}", code),
            Self::IoError(err) => write!(f, "IO Error: {}", err),
            Self::LastkajenError(err) => write!(f, "Lastkajen Error: {}", err), // Add formatting for other error variants
        }
    }
}

impl std::error::Error for LastkajenError {}

pub type Result<T> = std::result::Result<T, LastkajenError>;

// -----------------------------------------------------
/// Api client for Lastkajen.
///
/// ```rust
/// # use tokio_test;
/// # use std::env;
/// # use dotenv::dotenv;
/// # use lastkajen::*;
/// # tokio_test::block_on(async {
///     dotenv().ok(); // For example...
///     let username = env::var("USERNAME").unwrap();
///     let password = env::var("PASSWORD").unwrap();
///     
///
///     let lastkajen = Lastkajen::new(username, password).await;
///     assert!(lastkajen.is_ok());
/// # })
///
/// ```
///
#[derive(Debug)]
pub struct Lastkajen {
    #[cfg(feature = "time")]
    pub expiry_date_time: time::OffsetDateTime,

    pub token: types::Token,
    client: reqwest::Client,
}

impl Lastkajen {
    async fn check_status(response: reqwest::Response) -> Result<reqwest::Response> {
        if response.status() != 200 {
            return Err(match response.text().await {
                Ok(text) => LastkajenError::LastkajenError(text),
                Err(err) => LastkajenError::ReqwestError(err),
            });
        }

        Ok(response)
    }

    /// Create new Lastkajen instance, fetching a bearer token.
    pub async fn new(user_name: String, password: String) -> Result<Self> {
        let token = Lastkajen::retrieve_token(user_name, password).await?;

        Ok(Self {
            #[cfg(feature = "time")]
            expiry_date_time: time::OffsetDateTime::now_utc()
                .saturating_add(time::Duration::seconds(token.expires_in as i64)),

            token,
            client: reqwest::Client::new(),
        })
    }

    /// Manually retrieve a new bearer token.
    /// ```rust
    /// # use tokio_test;
    /// # use std::env;
    /// # use dotenv::dotenv;
    /// # use lastkajen::*;
    /// # tokio_test::block_on(async {
    /// #   dotenv().ok();
    /// #   let username = env::var("USERNAME").unwrap();
    /// #   let password = env::var("PASSWORD").unwrap();
    ///     let token: Result::<types::Token, _> = Lastkajen::retrieve_token(username, password).await;
    ///     assert!(token.is_ok());
    /// # })
    ///
    /// ```
    pub async fn retrieve_token(user_name: String, password: String) -> Result<types::Token> {
        let params = [("UserName", user_name), ("Password", password)];
        let client = reqwest::Client::new();

        let res = client
            .post("https://lastkajen.trafikverket.se/api/Identity/Login")
            .form(&params)
            .send()
            .await?;

        // TODO: de-tangle this.
        Ok(Lastkajen::check_status(res).await?.json().await?)
    }

    /// Get available public data packages.
    pub async fn get_published_packages(&self) -> Result<Vec<types::DataPackageFolder>> {
        let res = self
            .client
            .get("https://lastkajen.trafikverket.se/api/DataPackage/GetPublishedDataPackages")
            .bearer_auth(&self.token.access_token)
            .send()
            .await?;

        Ok(Lastkajen::check_status(res).await?.json().await?)
    }

    /// Get information and download links for a data package.
    pub async fn get_package_files(
        &self,
        package: &types::DataPackageFolder,
    ) -> Result<Vec<types::DataPackageFile>> {
        self.get_package_files_from_id(&package.id).await
    }

    /// Get information and download links for a data package.
    pub async fn get_package_files_from_id(
        &self,
        id: &usize,
    ) -> Result<Vec<types::DataPackageFile>> {
        let res = self
            .client
            .get(format!(
                "https://lastkajen.trafikverket.se/api/DataPackage/GetDataPackageFiles/{}",
                id
            ))
            .bearer_auth(&self.token.access_token)
            .send()
            .await?;

        Ok(Lastkajen::check_status(res).await?.json().await?)
    }

    /// Get information on user orders.
    pub async fn get_user_files(&self) -> Result<Vec<types::UserFile>> {
        let res = self
            .client
            .get("https://lastkajen.trafikverket.se/api/file/GetUserFiles")
            .bearer_auth(&self.token.access_token)
            .send()
            .await?;

        Ok(Lastkajen::check_status(res).await?.json().await?)
    }

    /// Get download token for a file-which are single use and valid for 60 seconds
    pub async fn get_download_token(
        &self,
        category: types::DownloadCategory<'_>,
    ) -> Result<types::DownloadToken> {
        let url: String = match category {
            types::DownloadCategory::User { file } => format!("https://lastkajen.trafikverket.se/api/file/GetUserFileDownloadToken?fileName={}", file),
            types::DownloadCategory::Published { id, file } => format!("https://lastkajen.trafikverket.se/api/file/GetDataPackageDownloadToken?id={}&fileName={}", id, file),
        };

        let res = Lastkajen::check_status(
            self.client
                .get(url)
                .bearer_auth(&self.token.access_token)
                .send()
                .await?,
        )
        .await?;

        match category {
            types::DownloadCategory::User { .. } => {
                Ok(types::DownloadToken::User(res.json().await?))
            }
            types::DownloadCategory::Published { .. } => {
                Ok(types::DownloadToken::Published(res.json().await?))
            }
        }
    }

    /// Download data, expending a download token.
    pub async fn download_with_token(
        &self,
        download_token: types::DownloadToken,
        writable: &mut dyn Write,
    ) -> Result<()> {
        let url: String = match download_token {
            // No, they aren't interchangable for some reason.
            types::DownloadToken::User(dltoken) => format!(
                "https://lastkajen.trafikverket.se/api/file/GetFileStream?token={}",
                dltoken
            ),
            types::DownloadToken::Published(dltoken) => format!(
                "https://lastkajen.trafikverket.se/api/file/GetDataPackageFile?token={}",
                dltoken
            ),
        };

        let mut res = Lastkajen::check_status(self.client.get(url).send().await?).await?;

        while let Some(chunk) = res.chunk().await? {
            writable.write_all(&chunk)?;
        }

        Ok(())
    }

    /// Download data, creating _and_ expending a download token.
    pub async fn download_file(
        &self,
        category: types::DownloadCategory<'_>,
        writable: &mut dyn Write,
    ) -> Result<()> {
        let download_token = self.get_download_token(category).await?;
        self.download_with_token(download_token, writable).await
    }
}
