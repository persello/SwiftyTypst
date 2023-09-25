use std::fs;
use std::path::{Path, PathBuf};

use crate::st_log;
use typst::diag::{PackageError, PackageResult};
use typst::syntax::PackageSpec;

/// Make a package available in the on-disk cache.
pub fn prepare_package(spec: &PackageSpec) -> PackageResult<PathBuf> {
    st_log!("Preparing package: {}", spec);

    let subdir = format!(
        "typst/packages/{}/{}-{}",
        spec.namespace, spec.name, spec.version
    );

    st_log!("Subdirectory for {} is {}.", spec, subdir);

    if let Some(data_dir) = dirs::data_dir() {
        st_log!("Data directory found: {}.", data_dir.display());
        let dir = data_dir.join(&subdir);
        if dir.exists() {
            st_log!("Package directory found in data: {}.", dir.display());
            return Ok(dir);
        }
    }

    if let Some(cache_dir) = dirs::cache_dir() {
        st_log!("Cache directory found: {}.", cache_dir.display());
        let dir = cache_dir.join(&subdir);

        // Download from network if it doesn't exist yet.
        if spec.namespace == "preview" && !dir.exists() {
            st_log!("Namespace is preview, downloading package.");
            download_package(spec, &dir)?;
        }

        if dir.exists() {
            st_log!("Package dir found in cache: {}.", dir.display());
            return Ok(dir);
        }
    }

    st_log!("Package not found: {}.", spec);

    Err(PackageError::NotFound(spec.clone()))
}

/// Download a package over the network.
fn download_package(spec: &PackageSpec, package_dir: &Path) -> PackageResult<()> {
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
    tar::Archive::new(decompressed)
        .unpack(package_dir)
        .map_err(|e| {
            st_log!("Failed to unpack package: {}.", e);
            fs::remove_dir_all(package_dir).ok();
            PackageError::MalformedArchive(Some(e.to_string().into()))
        })
}
