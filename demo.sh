#!/usr/bin/env bash

docker run --rm -ti -v .:/opt/paq -e "TERM=xterm-256color" -w /opt/paq rust /bin/bash -c "cargo install --path=. && cargo install --locked --git https://github.com/k9withabone/autocast && cargo install --locked --git https://github.com/asciinema/agg && cargo install --locked --git https://github.com/asciinema/agg && autocast --overwrite demo.yaml demo.cast && agg demo.cast paq.gif"
