//
// Copyright:: Copyright (c) 2015 Chef Software, Inc.
// License:: Apache License, Version 2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

//! Starts a service from an installed bldr package.
//!
//! Services in bldr support one or more *topologies*, which are state machines that handle the
//! lifecycle of a service; they are members of a *group*, which is a namespace for their
//! configuration and state; and they can *watch* another service group, incorporating that groups
//! configuration and state into their own.
//!
//! # Examples
//!
//! ```bash
//! $ bldr start chef/redis
//! ```
//!
//! Will start the `redis` service in the `default` group, using the `standalone` topology.
//!
//! ```bash
//! $ bldr start chef/redis -g production
//! ```
//!
//! Will do the same, but in the `production` group.
//!
//! ```bash
//! $ bldr start haproxy -w redis.production
//! ```
//!
//! Will start the `haproxy` service, and have it watch the configuration for the `redis`
//! `production` group (note the `.` as the separator.)
//!
//! ```bash
//! $ bldr start chef/redis -t leader
//! ```
//!
//! Will start the `redis` service using the `leader` topology.
//!
//! ```bash
//! $ bldr start chef/redis -t leader -g production -w haproxy.default
//! ```
//!
//! Will start the `redis` service using the `leader` topology in the `production` group, while
//! watching the `haproxy` `default` group's configuration.
//!
//! See the [documentation on topologies](../topology) for a deeper discussion of how they function.
//!

use ansi_term::Colour::Yellow;

use fs::PACKAGE_CACHE;
use error::{BldrResult, ErrorKind};
use config::Config;
use package::Package;
use topology::{self, Topology};
use command::install;
use repo;

static LOGKEY: &'static str = "CS";

/// Creates a [Package](../../pkg/struct.Package.html), then passes it to the run method of the
/// selected [topology](../../topology).
///
/// # Failures
///
/// * Fails if it cannot find a package with the given name
/// * Fails if the `run` method for the topology fails
/// * Fails if an unknown topology was specified on the command line
pub fn package(config: &Config) -> BldrResult<()> {
    match Package::load(config.deriv(), config.package(), None, None, None) {
        Ok(mut package) => {
            if let Some(ref url) = *config.url() {
                outputln!("Checking remote for newer versions...");
                let latest_pkg = try!(repo::client::show_package(&url,
                                                                 &package.derivation,
                                                                 &package.name,
                                                                 None,
                                                                 None));
                if latest_pkg > package {
                    outputln!("Downloading latest version from remote: {}", &latest_pkg);
                    let archive = try!(repo::client::fetch_package_exact(&url,
                                                                         &latest_pkg,
                                                                         PACKAGE_CACHE));
                    try!(archive.verify());
                    package = try!(archive.unpack());
                } else {
                    outputln!("Already running latest.");
                };
            }
            start_package(package, config)
        }
        Err(_) => {
            outputln!("{} not found in local cache",
                      Yellow.bold().paint(config.package()));
            match *config.url() {
                Some(ref url) => {
                    outputln!("Searching for {} in remote {}",
                              Yellow.bold().paint(config.package()),
                              url);
                    let package = try!(install::from_url(url,
                                                         config.deriv(),
                                                         config.package(),
                                                         config.version().clone(),
                                                         config.release().clone()));
                    start_package(package, config)
                }
                None => {
                    Err(bldr_error!(ErrorKind::PackageNotFound(config.deriv().to_string(),
                                                               config.package().to_string(),
                                                               config.release().clone(),
                                                               config.release().clone())))
                }
            }
        }
    }
}

fn start_package(package: Package, config: &Config) -> BldrResult<()> {
    match *config.topology() {
        Topology::Standalone => topology::standalone::run(package, config),
        Topology::Leader => topology::leader::run(package, config),
        Topology::Initializer => topology::initializer::run(package, config),
    }
}
