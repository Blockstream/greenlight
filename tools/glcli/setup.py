# -*- coding: utf-8 -*-
from setuptools import setup

packages = \
['glcli']

package_data = \
{'': ['*'], 'glcli': ['bin/*']}

install_requires = \
['click>=7.1.2,<7.2.0', 'protobuf', 'gl-client-py']

entry_points = \
{'console_scripts': ['glcli = glcli.cli:cli']}

setup_kwargs = {
    'name': 'glcli',
    'version': '0.1.1',
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

setup(**setup_kwargs)
