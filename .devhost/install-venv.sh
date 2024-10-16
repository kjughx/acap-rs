#!/usr/bin/env sh
set -eux

VIRTUAL_ENV="${1}"
INIT_ENV="${2:-}"

# Ensure the existence of venv.
# This is used below for local installations of applications and libraries.
python3 -m venv "${VIRTUAL_ENV}"

. "${VIRTUAL_ENV}/bin/activate"

# Install python programs and packages
# As of writing this is needed because:
# - the system version of `jsonschema` on Bookworm is 4.10.3, but the SDK uses 3.2.0, which
#   presumably is incompatible.
PIP_CONSTRAINT=constraints.txt pip install --requirement requirements.txt

# Install `npm` into venv.
# As of writing this is needed because `devcontainer` is a convenient way to test dev containers
# automatically.
curl \
  --location \
  --output /tmp/node-v18.16.1-linux-x64.tar.gz \
  "https://nodejs.org/dist/v18.16.1/node-v18.16.1-linux-x64.tar.gz"

echo "59582f51570d0857de6333620323bdeee5ae36107318f86ce5eca24747cabf5b /tmp/node-v18.16.1-linux-x64.tar.gz" \
| sha256sum -c -

tar -xf "/tmp/node-v18.16.1-linux-x64.tar.gz" --strip-components 1 -C "${VIRTUAL_ENV}"

rm /tmp/node-v18.16.1-linux-x64.tar.gz

# Install `devcontainer` into venv
npm install -g @devcontainers/cli@0.65.0

# Install rust programs
cargo install --locked --root ${VIRTUAL_ENV} --target-dir /tmp/target cargo-about@0.6.2
cargo install --locked --root ${VIRTUAL_ENV} --target-dir /tmp/target mkhelp@0.2.3
cargo install --root ${VIRTUAL_ENV} --target-dir /tmp/target --path ../crates/acap-ssh-utils
cargo install --root ${VIRTUAL_ENV} --target-dir /tmp/target --path ../crates/cargo-acap-build
cargo install --root ${VIRTUAL_ENV} --target-dir /tmp/target --path ../crates/device-manager

rm -r /tmp/target

# Create `init_env.sh` in a location where it can be sourced conveniently.
if [ ! -z "${INIT_ENV}" ];
then
  {
    echo "# Automatically created by install-venv.sh";
    echo ". ${VIRTUAL_ENV}/bin/activate";
    echo "unset -f deactivate";
    echo 'cargo-acap-sdk completions $(basename $SHELL) | . /dev/stdin'
    echo alias asdk=cargo-acap-sdk
    cat environment-setup.sh;
  } > "${INIT_ENV}"
fi
