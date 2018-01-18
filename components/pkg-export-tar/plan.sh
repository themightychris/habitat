pkg_name=hab-pkg-export-tar
_pkg_distname=$pkg_name
pkg_origin=core
pkg_version=$(cat "$PLAN_CONTEXT/../../VERSION")
pkg_maintainer="The Habitat Maintainers <humans@habitat.sh>"
pkg_license=('Apache-2.0')
pkg_deps=(core/coreutils core/findutils core/gawk core/grep core/bash core/tar core/gzip core/hab)
pkg_build_deps=(
  core/musl core/zlib-musl core/xz-musl core/bzip2-musl core/libarchive-musl
  core/openssl-musl core/libsodium-musl
  core/coreutils core/rust core/gcc core/make
)

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

do_prepare() {
  _common_prepare

  export rustc_target="x86_64-unknown-linux-musl"
  build_line "setting rustc_target=$rustc_target"

  la_ldflags="-l$(pkg_path_for zlib-musl)/lib -lz"
  la_ldflags="$la_ldflags -l$(pkg_path_for xz-musl)/lib -llzma"
  la_ldflags="$la_ldflags -l$(pkg_path_for bzip2-musl)/lib -lbz2"
  la_ldflags="$la_ldflags -l$(pkg_path_for openssl-musl)/lib -lssl -lcrypto"

  export libarchive_lib_dir=$(pkg_path_for libarchive-musl)/lib
  export libarchive_include_dir=$(pkg_path_for libarchive-musl)/include
  export libarchive_ldflags="$la_ldflags"
  export libarchive_static=true
  export openssl_lib_dir=$(pkg_path_for openssl-musl)/lib
  export openssl_include_dir=$(pkg_path_for openssl-musl)/include
  export openssl_static=true
  export sodium_lib_dir=$(pkg_path_for libsodium-musl)/lib
  export sodium_static=true

  # used to find libgcc_s.so.1 when compiling `build.rs` in dependencies. since
  # this used only at build time, we will use the version found in the gcc
  # package proper--it won't find its way into the final binaries.
  export ld_library_path=$(pkg_path_for gcc)/lib
  build_line "setting ld_library_path=$ld_library_path"

  plan_tar_pkg_ident=$(pkg_path_for tar | sed "s,^$hab_pkg_path/,,")
  export plan_tar_pkg_ident
  build_line "setting plan_tar_pkg_ident=$plan_tar_pkg_ident"
}

do_build() {
  pushd $plan_context > /dev/null
  cargo build ${build_type#--debug} --target=$rustc_target --verbose
  popd > /dev/null
}

do_install() {
  install -v -d $cargo_target_dir/$rustc_target/${build_type#--}/$bin \
    $pkg_prefix/bin/$bin
}

do_strip() {
  if [[ "$build_type" != "--debug" ]]; then
    do_default_strip
  fi
}
