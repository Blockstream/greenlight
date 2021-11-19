install:
	(cd ../rust/gl-client-py; make install)
	pip install .

test:
	virtualenv .tmpenv
	(cd .tmpenv; bin/pip3 install -U pip setuptools wheel pep517)
	(cd .tmpenv; bin/pip3 install ../dist/glcli-0.0.1.tar.gz)
	(cd .tmpenv; bin/python3 -c "from glcli import libhsmd;import _libhsmd;print(libhsmd.__file__);print(_libhsmd.__file__);print(libhsmd.init('00'*32, 'bitcoin'));req='0017000B48656c6c6f20776f726c64';sig = libhsmd.handle(1024, 0, None, req);assert sig.startswith('007b');assert len(sig) / 2 == 67")
	.tmpenv/bin/python3 -c 'import _libhsmd;print(_libhsmd.__file__)'
	find .tmpenv -name '_libhsmd.*.so' | xargs ldd
	rm -rf .tmpenv

clean:
	rm -rf build dist glcli.egg-info .mypy_cache src tmpenv .tmpenv .tmpenv2

testinstall:
	virtualenv .tmpenv2 --python=/usr/bin/python3 --download --always-copy --clear
	.tmpenv2/bin/python3 -m pip install -U pip
	.tmpenv2/bin/python3 -m pip install --extra-index-url=https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/simple/ glcli
	echo -n "00000000000000000000000000000000" > .tmpenv2/hsm_secret
	cd .tmpenv2 && ./bin/glcli --testenv scheduler ping

sdist:
	python setup.py sdist

publish: sdist
	twine upload --repository-url https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/ dist/glcli-*.tar.gz
