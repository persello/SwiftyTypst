use std::io::Read;
use std::path::{Path, PathBuf};

use crate::st_log;
use typst::diag::{PackageError, PackageResult};
use typst::syntax::PackageSpec;

use super::SystemWorld;

impl SystemWorld {
    /// Make a package available in the on-disk cache.
    pub fn check_package(&self, spec: &PackageSpec) -> PackageResult<()> {
        st_log!("Preparing package: {}", spec);

        if self
            .file_manager
            .exists("/".into(), spec.to_string())
            .unwrap_or(false)
        {
            Ok(())
        } else {
            st_log!("Package not found: {}, downloading.", spec);
            if spec.namespace != "preview" {
                st_log!("Only the `preview` namespace supports on-demand fetching.");
                return Err(PackageError::NotFound(spec.clone()));
            }

            self.download_package(spec, &PathBuf::from("/"))
        }
    }

    /// Download a package over the network.
    fn download_package(&self, spec: &PackageSpec, package_dir: &Path) -> PackageResult<()> {
        // The `@preview` namespace is the only namespace that supports on-demand
        // fetching.
        assert_eq!(spec.namespace, "preview");

        let url = format!(
            "https://packages.typst.org/preview/{}-{}.tar.gz",
            spec.name, spec.version
        );

        st_log!("Downloading package from {}.", url);

        let reader = match ureq::get(&url).call() {
            Ok(response) => response.into_reader(),
            Err(ureq::Error::Status(404, _)) => {
                st_log!("404 - Package not found: {}.", spec);
                return Err(PackageError::NotFound(spec.clone()));
            }
            Err(e) => {
                st_log!("Network error: {}.", e);
                return Err(PackageError::NetworkFailed(Some(e.to_string().into())));
            }
        };

        st_log!("Unpacking package to {}.", package_dir.display());

        let decompressed = flate2::read::GzDecoder::new(reader);

        let mut archive = tar::Archive::new(decompressed);
        let entries = archive
            .entries()
            .map_err(|e| PackageError::MalformedArchive(Some(e.to_string().into())))?
            .filter_map(|e| e.ok());

        entries.for_each(|e| {
            let path = e.path().unwrap();
            let path = package_dir.join(path);
            if let Some(parent) = path.parent() {
                let result = self
                    .file_manager
                    .create_directory(
                        parent.to_str().unwrap_or("").into(),
                        format!("{}/{}:{}", spec.namespace, spec.name, spec.version),
                    )
                    .map_err(|e| PackageError::Other(Some(e.to_string().into())));

                if result.is_err() {
                    st_log!("Error creating directory: {}.", result.unwrap_err());
                }
            };

            // Get the data from the entry.
            let data = e.bytes().map(|b| b.unwrap_or(0)).collect::<Vec<u8>>();

            // Write the data to the file.
            let result = self
                .file_manager
                .write(
                    path.to_str().unwrap_or("").into(),
                    format!("{}/{}:{}", spec.namespace, spec.name, spec.version),
                    data,
                )
                .map_err(|e| PackageError::Other(Some(e.to_string().into())));

            if result.is_err() {
                st_log!("Error writing file: {}.", result.unwrap_err());
            }
        });

        Ok(())
    }
}
