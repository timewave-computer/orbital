#!/bin/bash

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.15.1

ls local-interchaintest/contracts/orbital/
rm -r local-interchaintest/contracts/orbital/*
ls local-interchaintest/contracts/orbital/
cp -r artifacts/* local-interchaintest/contracts/orbital
