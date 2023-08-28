#! /usr/bin/bash

# Starting script for the dummy-plugin
# It looks if an environment variable named PYTHON exists.
# If not it initializes the variable to python3 and runs the plugin

: "${PYTHON:=python3}"
$PYTHON $(dirname $(realpath $0))/dummy_lsps_plugin.py