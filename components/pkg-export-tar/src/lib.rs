#[macro_use]
extern crate clap;
extern crate habitat_core as hcore;
extern crate url;
extern crate habitat_common as common;
extern crate base64;

extern crate hab;
extern crate handlebars;

extern crate mktemp;
extern crate tempdir;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

mod build;
pub mod cli;
mod error;
mod fs;
pub mod rootfs;
mod tar;
mod util;

use std::process::Command;
pub use cli::{Cli, PkgIdentArgOptions};
pub use error::{Error, Result};
use common::ui::UI;
use hcore::channel;
use hcore::url as hurl;
use mktemp::Temp;

use tar::{TarBuildRoot};

pub use build::BuildSpec;

/// The version of this library and program when built.
pub const VERSION: &'static str = include_str!(concat!(env!("OUT_DIR"), "/VERSION"));

/// The Habitat Package Identifier string for a Busybox paCkage.
const BUSYBOX_IDENT: &'static str = "core/busybox-static";
/// The Habitat Package Identifier string for SSL certificate authorities (CA) certificates package.
const CACERTS_IDENT: &'static str = "core/cacerts";

/// An image naming policy.
///
/// This is a value struct which captures the naming and tagging intentions for an image.
#[derive(Debug)]
pub struct Naming<'a> {
    /// An optional custom image name which would override a computed default value.
    pub custom_image_name: Option<&'a str>,
    /// Whether or not to tag the image with a latest value.
    pub latest_tag: bool,
    /// Whether or not to tag the image with a value containing a version from a Package
    /// Identifier.
    pub version_tag: bool,
    /// Whether or not to tag the image with a value containing a version and release from a
    /// Package Identifier.
    pub version_release_tag: bool,
    /// An optional custom tag value for the image.
    pub custom_tag: Option<&'a str>,
}

impl<'a> Naming<'a> {
    /// Creates a `Naming` from cli arguments.
    pub fn new_from_cli_matches(m: &'a clap::ArgMatches) -> Self {
        Naming {
            custom_image_name: m.value_of("IMAGE_NAME"),
            latest_tag: !m.is_present("NO_TAG_LATEST"),
            version_tag: !m.is_present("NO_TAG_VERSION"),
            version_release_tag: !m.is_present("NO_TAG_VERSION_RELEASE"),
            custom_tag: m.value_of("TAG_CUSTOM"),
        }
    }
}

pub fn export_for_cli_matches(ui: &mut UI, matches: &clap::ArgMatches) -> Result<()> {
    let default_channel = channel::default();
    let default_url = hurl::default_bldr_url();
    let spec = BuildSpec::new_from_cli_matches(&matches, &default_channel, &default_url);
    let naming = Naming::new_from_cli_matches(&matches);
    let tarball = export(ui, spec, &naming)?;

    Ok(())
}

pub fn export(ui: &mut UI, build_spec: BuildSpec, naming: &Naming) -> Result<()> {

   let hart_to_package = build_spec.idents_or_archives.join(", ");
   ui.begin(format!(
        "Building a tarball with: {}",
        hart_to_package
    ))?;

    let temp_dir_path = Temp::new_dir().unwrap().to_path_buf();

    let studio_command = Command::new("hab")
                                    .arg("studio")
                                    .arg("-r")
                                    .arg(&temp_dir_path)
                                    .arg("new")
                                    .output();

    let install_command = Command::new("hab")
                                    .arg("studio")
                                    .arg("-q")
                                    .arg("-r")
                                    .arg(&temp_dir_path)
                                    .arg("run")
                                    .arg("hab")
                                    .arg("install")
                                    .arg(&hart_to_package)
                                    .output();

//    let echo_command = Command::new("echo")  
//                                    .arg(hart_to_package)
//                                    .arg(">")
//                                    .arg(&hab_pkg_path)
//                                    .output();

    let tar_command = Command::new("tar")
                                   .arg("cpzf")
                                   .arg("effit.tar.gz")
                                   .arg("-C")
                                   .arg(&temp_dir_path)
                                   .arg("./hab/pkgs")
                                   .arg("./hab/bin")
                                   .output();
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

