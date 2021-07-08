use crate::image::Image;
use crate::skopeo::{Skopeo, SkopeoConfiguration};
use crate::umoci::{Umoci, UmociConfiguration, UnpackArgs};
use crate::*;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ImageManagerConfiguration {
    pub oci_manager: UmociConfiguration,
    pub image_puller: SkopeoConfiguration,
}

#[derive(Debug)]
pub struct ImageManager {
    config: ImageManagerConfiguration,
    umoci: Umoci,
    skopeo: Skopeo,
}

/// Implementation of the skopeo library
/// coupled with umoci in order to pull images compatible with cri.
impl ImageManager {
    /// Create a new Puller
    pub fn new(config: ImageManagerConfiguration) -> Result<Self> {
        let umoci = Umoci::new(config.oci_manager.clone())?;
        let skopeo = Skopeo::new(config.image_puller.clone())?;

        debug!("ImageManager initialized.");

        Ok(ImageManager {
            config,
            umoci,
            skopeo,
        })
    }

    /// Format the image for skopeo with the following format:
    /// docker://<IMAGE>
    fn format_image_src(&self, image: &String) -> String {
        format!("docker://{}", image)
    }

    /// Pull image locally
    pub async fn pull(&mut self, image_str: &str) -> Result<Image> {
        let bundle_directory = &self.config.oci_manager.bundles_directory.clone().unwrap();
        let mut image = Image::from(image_str);

        if !image.should_be_pulled(&bundle_directory.clone()) {
            log::info!(
                "Using local image for {} due to IfNotPresent image policy",
                image.oci
            );
            let bundle = format!(
                "{}/{}",
                bundle_directory.to_str().unwrap(),
                image.get_uuid()
            );
            image.set_bundle(&bundle[..]);

            return Ok(image);
        }

        info!("Pulling image {}", image_str);
        let src = self.format_image_src(&image.oci);
        let image_path = self
            .skopeo
            .copy(
                &src,
                &format!("{}", &image.get_hashed_oci()),
                Default::default(),
            )
            .await?;

        debug!("{} copied into {}", image_str, image_path);

        let bundle = self
            .umoci
            .unpack(
                &image.get_uuid(),
                Some(&UnpackArgs {
                    image: PathBuf::from(&format!("{}:{}", image_path, image.tag)),
                    rootless: false,
                    uid_map: None,
                    gid_map: None,
                    keep_dirlinks: false,
                }),
            )
            .await?;

        image.set_bundle(&bundle[..]);

        info!("Successfully pulled image {}", image_str);

        Ok(image)
    }
}
