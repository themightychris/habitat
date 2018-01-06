use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use common::ui::{UI, Status};
use hcore::fs as hfs;
use hcore::os::filesystem;
use handlebars::Handlebars;
use build::BuildRoot;

use error::{Error, Result};
use serde_json;

use super::{Naming};
use util;

const INIT_SH: &'static str = include_str!("../defaults/init.sh.hbs");
lazy_static! {
    /// Absolute path to the tar program
    static ref TAR_PROGRAM: PathBuf = hfs::resolve_cmd_in_pkg(
        "tar",
        include_str!(concat!(env!("OUT_DIR"), "/TAR_PKG_IDENT")),
    );
}


/// A builder used to create a Tarball
pub struct TarBallBuilder {
    /// The base workdir which hosts the root file system.
    workdir: PathBuf,
}

impl TarBallBuilder {
    fn new(workdir: PathBuf) -> Self {
        TarBallBuilder {
            workdir: workdir,
        }
    }

    pub fn build(self) -> Result<TarBall> {
// Take Hart File
// Install in new studio ("bare new" studio) - this will put the application and any dependencies in the /hab directory
// Tar up the /hab director
        let mut cmd = tar_cmd();
        cmd.arg("-cpzf").arg("xyz.tar.gz").arg("-C").arg(self.workdir);

        debug!("Running: {:?}", &cmd);
        let exit_status = cmd.spawn()?.wait()?;

        // TEMPORARY
        Ok(TarBall{
            id: String::from("BiteMe")
        })
    }
}

/// A temporary file system build root for building a tarball, based on Habitat packages.
pub struct TarBuildRoot(BuildRoot);

impl TarBuildRoot {
    pub fn from_build_root(build_root: BuildRoot, ui: &mut UI) -> Result<Self> {
        let root = TarBuildRoot(build_root);
        if cfg!(target_os = "linux") {
            root.add_users_and_groups(ui)?;
            root.create_entrypoint(ui)?;
        }

        root.create_tar_env(ui)?;
        Ok(root)
    }

    fn create_tar_env(&self, ui: &mut UI) -> Result<()> {
        let ctx = self.0.ctx();
        println!("===========");
        println!("ctx {:?}", ctx);
        Ok(())
    }

    fn add_users_and_groups(&self, ui: &mut UI) -> Result<()> {
        let ctx = self.0.ctx();
        let (users, groups) = ctx.svc_users_and_groups()?;
        {
            let file = "etc/passwd";
            let mut f = OpenOptions::new().append(true).open(
                ctx.rootfs().join(&file),
            )?;
            for line in users {
                let user = line.split(":").next().expect(
                    "user line contains first entry",
                );
                ui.status(
                    Status::Creating,
                    format!("user '{}' in /{}", user, &file),
                )?;
                f.write_all(line.as_bytes())?;
            }
        }
        {
            let file = "etc/group";
            let mut f = OpenOptions::new().append(true).open(
                ctx.rootfs().join(&file),
            )?;
            for line in groups {
                let group = line.split(":").next().expect(
                    "group line contains first entry",
                );
                ui.status(
                    Status::Creating,
                    format!("group '{}' in /{}", group, &file),
                )?;
                f.write_all(line.as_bytes())?;
            }
        }
        Ok(())
    }

    fn create_entrypoint(&self, ui: &mut UI) -> Result<()> {
        ui.status(Status::Creating, "entrypoint script")?;
        let ctx = self.0.ctx();
        let busybox_shell = util::pkg_path_for(&util::busybox_ident()?, ctx.rootfs())?
            .join("bin/sh");
        let json = json!({
            "busybox_shell": busybox_shell,
            "path": ctx.env_path(),
            "sup_bin": format!("{} sup", ctx.bin_path().join("hab").display()),
            "primary_svc_ident": ctx.primary_svc_ident().to_string(),
        });
        let init = ctx.rootfs().join("init.sh");
        util::write_file(&init, &Handlebars::new().template_render(INIT_SH, &json)?)?;
        filesystem::chmod(init.to_string_lossy().as_ref(), 0o0755)?;
        Ok(())
    }

    pub fn export(&self, ui: &mut UI, naming: &Naming) -> Result<TarBall> { 
        self.build_tarball(ui, naming)
    }

    fn build_tarball(&self, ui: &mut UI, naming: &Naming) -> Result<TarBall> {
        ui.status(Status::Creating, "Tarball")?;
        let ident = self.0.ctx().installed_primary_svc_ident()?;
        let version = &ident.version.expect("version exists");
        let release = &ident.release.expect("release exists");
        let json = json!({
            "pkg_origin": ident.origin,
            "pkg_name": ident.name,
            "pkg_version": &version,
            "pkg_release": &release,
            "channel": self.0.ctx().channel(),
        });



        let mut tarball_path = PathBuf::new();

        tarball_path.push(self.0.workdir());
        // Unpack package in the tarball_path directory

        let mut tarball = TarBall::new(tarball_path);

        tarball.build()
    } 
}

pub struct TarBall {
    /// The ID for this tarball.
    id: String,
}

impl TarBall {
    /// Returns a new `TarBallBuilder` which is used to build the image.
    pub fn new(workdir: PathBuf) -> TarBallBuilder {
        TarBallBuilder::new(workdir)
    }
}

/// Returns a `Command` for the Tar program.
fn tar_cmd() -> Command {
    Command::new(&*TAR_PROGRAM)
}