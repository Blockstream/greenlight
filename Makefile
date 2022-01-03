include libs/gl-client-py/Makefile
include libs/gl-client-js/Makefile

PY_REPO_URL=https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/
CIBW_MANYLINUX_I686_IMAGE=quay.io/pypa/manylinux_2_24_i686
CIBW_MANYLINUX_X86_64_IMAGE=quay.io/pypa/manylinux_2_24_x86_64
CIBW_MANYLINUX_PYPY_X86_64_IMAGE=quay.io/pypa/manylinux_2_24_x86_64
CIBW_MANYLINUX_PYPY_AARCH64_IMAGE=quay.io/pypa/manylinux_2_24_x86_64
CIBW_MUSLLINUX_X86_64_IMAGE=quay.io/pypa/musllinux_1_1_x86_64
CIBW_MUSLLINUX_I686_IMAGE=quay.io/pypa/musllinux_1_1_i686
CIBW_BUILD_VERBOSITY=1
CIBW_ENVIRONMENT_LINUX='PATH=$PATH:$HOME/.cargo/bin DEVELOPER=0'
CIBW_BEFORE_ALL_LINUX="apt-get update -qq && apt-get install -y clang libgmp-dev pkg-config libsqlite3-dev valgrind && curl -sSf https://sh.rustup.rs | sh -s -- -y"
CIBW_ARCHS_MACOS=native \
CIBW_ENVIRONMENT_MACOS="CPATH=/opt/homebrew/include LIBRARY_PATH=/opt/homebrew/lib" \

uploadwhl:
	twine upload --repository-url=${PY_REPO_URL} wheelhouse/glapi-*.whl --skip-existing
	twine upload --repository-url=${PY_REPO_URL} wheelhouse/glapi-*.tar.gz --skip-existing
	twine upload --repository-url=${PY_REPO_URL} wheelhouse/gl_client_py-*.whl --skip-existing
	twine upload --repository-url=${PY_REPO_URL} wheelhouse/gl_client_py-*.tar.gz --skip-existing

gl_client_py:
	bash -x libs/gl-client-py/build-wheels.sh

glapi:
	pip wheel --extra-index-url=${PY_REPO_URL}simple/ -w wheelhouse ./python/
	rm -rf python/dist
	(cd python; python setup.py sdist)
	mv python/dist/glapi-*.tar.gz wheelhouse

check: check-rs check-py check-js

check-rs:
	cd libs; cargo test --all -- --test-threads=1

clean-rs:
	cd libs; cargo clean

clean: clean-rs
