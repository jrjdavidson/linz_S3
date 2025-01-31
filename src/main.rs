use stac::{Href, Links};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        match stac::io::get_opts::<stac::Item, _, _, _>(
            "s3://nz-elevation/catalog.json",
            [("skip_signature", "true"), ("region", "ap-southeast-2")],
        )
        .await
        {
            Ok(mut item) => {
                item.make_links_absolute().unwrap();
                // Iterate through the links and fetch more details
                for link in item.links {
                    if let Href::Url(url) = link.href {
                        let parsed_url = url.as_str();
                        match stac::io::get_opts::<stac::Item, _, _, _>(
                            parsed_url,
                            [("skip_signature", "true"), ("region", "ap-southeast-2")],
                        )
                        .await
                        {
                            Ok(mut child_item) => {
                                println!("#######");
                                println!("Details for {}", link.title.unwrap_or_default(),);
                                child_item.make_links_absolute().unwrap();
                                // Iterate through the links and fetch more details

                                println!("Item ID: {}", child_item.id);
                                println!("Item Properties: {:?}", child_item.properties);
                                // Iterate through the links and print download URLs
                                for link in child_item.links {
                                    if let Href::Url(child_url) = link.href {
                                        println!("Download URL: {}", child_url);
                                        match stac::io::get_opts::<stac::Item, _, _, _>(
                                            child_url.as_str(),
                                            [
                                                ("skip_signature", "true"),
                                                ("region", "ap-southeast-2"),
                                            ],
                                        )
                                        .await
                                        {
                                            Ok(child_item) => {
                                                for asset in child_item.assets {
                                                    let (asset_string, asset_struct) = asset;
                                                    println!("Asset string: {}", asset_string);
                                                    println!("Asset href: {:?}", asset_struct.href);
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "Failed to get details for {}: {:?}",
                                                    link.title.unwrap_or_default(),
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }
                                for asset in child_item.assets {
                                    let (asset_string, asset_struct) = asset;
                                    println!("Asset string: {}", asset_string);
                                    println!("Asset href: {:?}", asset_struct.href);
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Failed to get details for {}: {:?}",
                                    link.title.unwrap_or_default(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get item: {:?}", e);
            }
        }
    });
}
