<img width="200" src="https://github.com/briansjensen/grux/blob/0c198a580b6d473bfbef642301e069507429be26/assets/logo.svg">

#  Grux - High performance web server

Grux is a web server focused on high performance and ease of administration. It has a built-in web interface to change settings, add websites etc. No more need for complicated configuration files with weird syntax. Written in high performance Rust. Supports PHP right out of the box.

[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](https://choosealicense.com/licenses/mit/)

## Features

- Serve files in fastest possible manner
- Really low memory footprint and low CPU usage per request
- Easy web interface for administration of everything built right in
- SSL/TLS support for secure https:// sites
- Supports HTTP1.1 and HTTP2.0
- PHP Support (both FPM and php-cgi on Windows (needs to be v7.1+ of PHP for Windows))
- High performance file cache to really speed up
- Gzip of content, to make it as small as possible (cached ofcourse)
- Monitoring of current load and state directly from the admin portal

## Getting started

To get started running Grux, there is a few options:

### Using the binaries

1. Download the release appropriate for the system you want to run it on.
2. This is a ready to go build right after it is extracted.
3. Run the binary and check out http://127.0.0.1 for the default Grux page.
4. To do configuration, go to https://127.0.0.1:8000 and login with the user "admin" and the password given in the output from the server on first run. Save it, as it will NOT be shown again.

Please let us know what you think. Both what is nice about Grux and which problems you had, so we better can improve it.

### With Docker:
Coming soon

## Screenshots


## Documentation

[You can find documentation for Grux web server here](https://grux.eu) (coming soon)


## Help with development

Do you want to help with the development and build Grux locally. It is easy.

### Using Rust framework:

1. Install rust framework - https://rust-lang.org/tools/install/
2. Clone Grux repository with git
3. Build grux by running: "cargo run -- -o DEV" (this will run it in dev mode, with trace log enabled)

If you want admin portal running, you need to build that too.

1. Install node.js
2. Go into /www-admin-src
3. Run "npm run build"

Grux can now be found on http://127.0.0.1 and admin portal on https://127.0.0.1:8000

### Easy mode development with Docker compose:
If you rather want total easy mode development, use the docker solution:

1. Install docker
2. Clone Grux repository with git
3. Go into /development
4. Run "docker compose up -d"
5. After a while, Grux is running on http://127.0.0.1 and admin portal on https://127.0.0.1:8000

Log in to admin portal with "admin" as username and password written in the server output. Only written on first startup.

After your changes is done, make sure it builds and tests are running.
Submit a PR and wait for approval. We appreciate any contribution and improvements.

## Licensing with commercial support

Grux is free to use for everybody (and always will be), but if you need support in a commercial context, let us know and we will figure out a solution. Contact us on <contact@grux.eu>.

## Authors

[Brian Søgård Jensen](https://www.github.com/briansjensen) - Contact info: <contact@grux.eu>
