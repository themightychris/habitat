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
extern crate serde_json;
extern crate regex;

mod build;
pub mod cli;
mod error;
mod fs;
pub mod rootfs;
mod util;

use std::process::Command;
use std::path::PathBuf;
use std::str::FromStr;
pub use cli::{Cli, PkgIdentArgOptions};
pub use error::{Error, Result};
use common::ui::UI;
use hcore::channel;
use hcore::url as hurl;
use hcore::package::{PackageIdent};
use mktemp::Temp;
use regex::Regex;

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
    export(ui, spec)?;

    Ok(())
}

pub fn export(ui: &mut UI, build_spec: BuildSpec) -> Result<()> {
   let hart_to_package = build_spec.idents_or_archives.join(", ");
   let builder_url = build_spec.url;

   ui.begin(format!(
        "Building a tarball with: {}",
        hart_to_package
    ))?;

    let temp_dir_path = Temp::new_dir().unwrap().to_path_buf();

    initiate_tar_command(&temp_dir_path, &hart_to_package, &builder_url);

    Ok(())
}

fn initiate_tar_command(temp_dir_path: &PathBuf, hart_to_package: &str, builder_url: &str) {
    let status = Command::new("hab")
        .arg("studio")
        .arg("-r")
        .arg(&temp_dir_path)
        .arg("new")
        .status()
        .expect("failed to create studio");

    if status.success() {
        println!("Able to create studio to export package as tarball, proceeding...");
        install_command(&temp_dir_path, &hart_to_package, &builder_url);
    } else {
        println!("Unable to create a studio to export the package as a tarball.")
    }
}

fn install_command(temp_dir_path: &PathBuf, hart_to_package: &str, builder_url: &str) {
    let status = Command::new("hab")
        .arg("studio")
        .arg("-q")
        .arg("-r")
        .arg(&temp_dir_path)
        .arg("run")
        .arg("hab")
        .arg("install")
        .arg("-u")
        .arg(builder_url)
        .arg(&hart_to_package)
        .status()
        .expect("failed to install package in studio");

        if status.success() {
            println!("Hart package is installable in a studio, proceeding with exporting it to a tarball...");
            tar_command(&temp_dir_path, &hart_to_package);
        } else {
            println!("Hart package is NOT installable in a studio and could not be exported into a tarball, please see the above error for more details.");
        }
}

fn tar_command(temp_dir_path: &PathBuf, hart_to_package: &str) {
    let status = Command::new("tar")
        .arg("cpzf")
        .arg(determine_name_of_tarball(&hart_to_package))
        .arg("-C")
        .arg(&temp_dir_path)
        .arg("./hab/pkgs")
        .arg("./hab/bin")
        .status()
        .expect("failed to create tarball");

    if status.success() {
        println!("Tarball export complete!")
    } else {
        println!("Unable to export package to tarball.")
    }
}

fn determine_name_of_tarball(hart_to_package: &str) -> String {
    let slash_syntax = Regex::new(r".+/.+").unwrap();
    
    // Check whether hart_to_package is in "core/origin" format
    // If so, create a PackageIdent, and use that to extract the origin
    // and package name and format it as a string
    if slash_syntax.is_match(&hart_to_package) {
        let package_ident = PackageIdent::from_str(&hart_to_package).unwrap();
        format!("{}-{}.tar.gz", package_ident.origin, package_ident.name)
    } else {
        String::from(hart_to_package)
    }
}