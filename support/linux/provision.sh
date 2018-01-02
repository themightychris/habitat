#!/bin/bash
set -eux

sudo sh /src/components/hab/install.sh
sudo hab install core/busybox-static core/hab-studio
sudo hab install \
  core/direnv \
  core/wget \
  core/git \
  core/curl -b
# shellcheck disable=SC2016
echo 'eval "$(direnv hook bash)"' | sudo tee --append /root/.bashrc > /dev/null
