# RGB-20 Library

RGB20 is an RGB schema for fungible digital assets on bitcoin & lightning.

This repository provides rust library and a command-line utility `rgb20` which
can be used alongside RGB Node to generate and parse RGB20 data (schema, issue
assets, interpret contract information returned by RGB Node).

## Command-line utility

### Install with Docker

#### Build

Clone the repository and checkout to the desired version (here `v0.8.0-rc.3`):

```console
$ git clone https://github.com/RGB-WG/rust-rgb20
$ cd rust-rgb20
$ git checkout v0.8.0-rc.3
```

Build and tag the Docker image:

```console
$ docker build -t rgb20:v0.8.0-rc.3 .
```

#### Usage

```console
$ docker run rgb20:v0.8.0-rc.3 --help
```
