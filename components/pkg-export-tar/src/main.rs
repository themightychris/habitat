extern crate clap;
extern crate env_logger;
extern crate habitat_core as hcore;
extern crate habitat_common as common;
extern crate habitat_pkg_export_tar as export_tar;
#[macro_use]
extern crate log;

use clap::App;
use common::ui::UI;
use hcore::PROGRAM_NAME;
use export_tar::{Cli, PkgIdentArgOptions, Result};

fn main() {
    env_logger::init().unwrap();
    let mut ui = UI::default_with_env();
    if let Err(e) = start(&mut ui) {
        ui.fatal(e).unwrap();
        std::process::exit(1)
    }
}

fn start(ui: &mut UI) -> Result<()> {
    let cli = cli();
    let m = cli.get_matches();
    debug!("clap cli args: {:?}", m);

    export_tar::export_for_cli_matches(ui, &m)
}

fn cli<'a, 'b>() -> App<'a, 'b> {
    let name: &str = &*PROGRAM_NAME;
    let about = "Creates a tar package from a Habitat package";
    Cli::new(name, about)
        .app

}
