[![CI](https://github.com/hannes-hochreiner/podcast-player/actions/workflows/main.yml/badge.svg)](https://github.com/hannes-hochreiner/podcast-player/actions/workflows/main.yml)
# Podcast Player

A simple offline-first app for listening to podcasts.

## Components

Since podcast feeds can - in general - not be accessed directly from a web application, a backend component was implemented.
Hence, there are three parts to the system:
* the web application (pwa)
* the backend (api)
* an update process (updater)

![component diagram](documentation/build/components.svg)

## License

This work is licensed under the MIT license.

`SPDX-License-Identifier: MIT`
