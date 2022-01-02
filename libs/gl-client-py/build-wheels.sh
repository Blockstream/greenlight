export CIBW_MANYLINUX_I686_IMAGE=quay.io/pypa/manylinux_2_24_i686
export CIBW_MANYLINUX_X86_64_IMAGE=quay.io/pypa/manylinux_2_24_x86_64
export CIBW_MANYLINUX_PYPY_X86_64_IMAGE=quay.io/pypa/manylinux_2_24_x86_64
export CIBW_MANYLINUX_PYPY_AARCH64_IMAGE=quay.io/pypa/manylinux_2_24_x86_64
export CIBW_MUSLLINUX_X86_64_IMAGE=quay.io/pypa/manylinux_2_24_x86_64
export CIBW_BUILD_VERBOSITY=1
export CIBW_ENVIRONMENT_LINUX='PATH=$PATH:$HOME/.cargo/bin DEVELOPER=0'
export CIBW_BEFORE_ALL_LINUX="apt-get update -qq && apt-get install -y clang libgmp-dev pkg-config libsqlite3-dev valgrind && curl -sSf https://sh.rustup.rs | sh -s -- -y"
export CIBW_ARCHS_MACOS="native"
export CIBW_ENVIRONMENT_MACOS="CPATH=/opt/homebrew/include LIBRARY_PATH=/opt/homebrew/lib"
export CIBW_BUILD_FRONTEND="build"

export CIBW_PLATFORM=unknown
if [[ "$(uname)" == "Darwin" ]]; then
  export CIBW_PLATFORM=macos
elif [[ "$(uname)" == "Linux" ]]; then
  export CIBW_PLATFORM="linux"
fi

(
    # Need to run in libs/ so we have access to proto/ and tls/
    cd $(git rev-parse --show-toplevel)
    cibuildwheel libs/gl-client-py
)
