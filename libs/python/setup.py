# -*- coding: utf-8 -*-
from setuptools import setup

packages = \
['glapi']

package_data = \
{'': ['*'], 'glapi': ['bin/*']}

install_requires = \
['click>=7.1.2,<7.2.0', 'grpcio>=1.34.0,<1.35.0', 'protobuf>=3.15.8,<3.16.0', 'libhsmd>=0.10.0']

entry_points = \
{'console_scripts': ['glcli = glapi.cli:cli']}

setup_kwargs = {
    'name': 'glapi',
    'version': '0.0.2',
    'description': '',
    'long_description': None,
    'author': 'Christian Decker',
    'author_email': 'decker.christian@gmail.com',
    'maintainer': None,
    'maintainer_email': None,
    'url': None,
    'packages': packages,
    'package_data': package_data,
    'install_requires': install_requires,
    'entry_points': entry_points,
    'python_requires': '>=3.5,<4.0',
}
#from extbuild import *
#build(setup_kwargs)

setup(**setup_kwargs)
