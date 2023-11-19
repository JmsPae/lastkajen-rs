use std::error::Error;
use std::fs::File;
use std::{env, println};

use dotenv::dotenv;
use lastkajen::types::{DataPackageFolder, DownloadCategory};
use lastkajen::Lastkajen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Create an API instance, getting a bearer token using the supplied credentials.
    let api = Lastkajen::new(env::var("USERNAME")?, env::var("PASSWORD")?).await?;

    // ------------------------------- Published products -------------------------------
    // Data Packages published by the Swedish traffic authority directly.

    // Get a listing of published packages and their information.
    // You'll need their IDs for further exploration.
    let packages = api.get_published_packages().await?;

    // The API doesn't have any querying capabilities, so we have to do that ourselves.
    let gavle_result: Vec<DataPackageFolder> = packages
        .into_iter()
        .filter(|package| package.source_folder == "Datapaket\\Länsfiler NVDB-data\\Gävleborgs län")
        .collect();
    let gavle_result = gavle_result
        .first()
        .expect("Couldn't find the right package!");
    // println!("{:?}", gavle_result);

    let files = api.get_package_files(&gavle_result).await?;

    // Most files outside the inspire folder have multiple file types
    let gpkg: Vec<lastkajen::types::DataPackageFile> = files
        .into_iter()
        .filter(|file| file.name == "Gävleborgs_län_GeoPackage.zip")
        .collect();
    let gpkg = gpkg.first().expect("Couldn't find the right file!");

    println!("{:?}", gpkg);

    // Downloading a published package requires both the package ID and the file name.
    // let mut file = File::open(&gpkg.name)?;
    // api.download_file(DownloadCategory::Published { id: &gavle_result.id, file: &gpkg.name }, &mut file).await?;

    // ------------------------------- User 'orders' -------------------------------
    // Custom extracts from the national road database created via the Lastkajen website. The
    // Lastkajen API does not support creation of custom orders.

    let user_files = api.get_user_files().await?;

    if let Some(user_file) = user_files.last() {
        println!("{:?}", user_file);

        // An ID of any sort is not required for user files.
        // let mut file = File::open(&user_file.name)?;
        // api.download_file(DownloadCategory::User { file: &user_file.name }, &mut file).await?;
    } else {
        println!("No User files.");
    }

    Ok(())
}
