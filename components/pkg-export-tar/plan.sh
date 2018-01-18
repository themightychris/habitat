pkg_name=hab-pkg-export-tar
_pkg_distname=$pkg_name
pkg_origin=core
pkg_version=$(cat "$PLAN_CONTEXT/../../VERSION")
pkg_maintainer="The Habitat Maintainers <humans@habitat.sh>"
pkg_license=('Apache-2.0')
pkg_source=nosuchfile.tar.gz
pkg_deps=(core/coreutils core/findutils core/gawk core/grep core/bash core/tar core/gzip core/hab)
pkg_build_deps=()
pkg_bin_dirs=(bin)

bin=$_pkg_distname

_common_prepare() {
  do_default_prepare

  # Can be either `--release` or `--debug` to determine cargo build strategy
  build_type="--release"
  build_line "Building artifacts with \`${build_type#--}' mode"

  # Used by the `build.rs` program to set the version of the binaries
  export PLAN_VERSION="${pkg_version}/${pkg_release}"
  build_line "Setting PLAN_VERSION=$PLAN_VERSION"

  if [ -z "$HAB_CARGO_TARGET_DIR" ]; then
    # Used by Cargo to use a pristine, isolated directory for all compilation
    export CARGO_TARGET_DIR="$HAB_CACHE_SRC_PATH/$pkg_dirname"
  else
    export CARGO_TARGET_DIR="$HAB_CARGO_TARGET_DIR"
  fi
  build_line "Setting CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
}

_common_prepare() {
  do_default_prepare

  # Can be either `--release` or `--debug` to determine cargo build strategy
  build_type="--release"
  build_line "Building artifacts with \`${build_type#--}' mode"

  # Used by the `build.rs` program to set the version of the binaries
  export PLAN_VERSION="${pkg_version}/${pkg_release}"
  build_line "Setting PLAN_VERSION=$PLAN_VERSION"

  if [ -z "$HAB_CARGO_TARGET_DIR" ]; then
    # Used by Cargo to use a pristine, isolated directory for all compilation
    export CARGO_TARGET_DIR="$HAB_CACHE_SRC_PATH/$pkg_dirname"
  else
    export CARGO_TARGET_DIR="$HAB_CARGO_TARGET_DIR"
  fi
  build_line "Setting CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
}

