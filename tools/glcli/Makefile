install:
	(cd ../rust/gl-client-py; make install)
	pip install .

clean:
	rm -rf build dist glcli.egg-info .mypy_cache src tmpenv .tmpenv .tmpenv2

test:
	virtualenv .tmpenv --python=$(which python3) --download --always-copy --clear
	.tmpenv/bin/python3 -m pip install -U pip
	.tmpenv/bin/python3 -m pip install --extra-index-url=https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/simple/ gl-client-py

	GL_SCHEDULER_GRPC_URI=https://scheduler.testing.gl.blckstrm.com:2601 .tmpenv/bin/python3 -c 'import glclient as gl; tls = gl.TlsConfig(); s = gl.Signer(b"0"*32, "regtest", tls); sched = gl.Scheduler(s.node_id(), "regtest"); print(sched.schedule())'
	.tmpenv/bin/python3 -c 'import _libhsmd;print(_libhsmd.__file__)'

sdist:
	python setup.py sdist

publish: sdist
	twine upload --repository-url https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/ dist/glcli-*.tar.gz
