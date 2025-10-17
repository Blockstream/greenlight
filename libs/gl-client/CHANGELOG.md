# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Fixed

- Addressed an issue with signers being unable to connect to the node, due to an SNI header override that is no longer required
- Parsing an invalid certificate no longer panics, instead returning an error.
- Addressed a deprecation warning in gl-testing regarding PROTOCOL_TLS being renamed to PROTOCOL_TLS_SERVER
