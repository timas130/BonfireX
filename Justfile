#!/usr/bin/env -S just --justfile

compose := if shell("sh -c 'podman compose version &> /dev/null; echo $?'") == "0" {
    "podman compose"
} else if shell("sh -c 'docker compose version &> /dev/null; echo $?'") == "0" {
    "docker compose"
} else if shell("sh -c 'docker-compose version &> /dev/null; echo $?'") == "0" {
    "docker-compose"
} else {
    error("could not find `podman compose`, `docker compose`, or `docker-compose`")
}

container-runtime := if compose =~ "podman" {
    "podman"
} else {
    "docker"
}

cargo := require("cargo")

# Show all recipies
default:
    just --list

alias r := router
# Start the RPC router
router:
	PORT=5000 {{ cargo }} run --bin bfx-rpc-router

# Run a service with variables from .env
run name:
	#!/usr/bin/env bash
	export $(grep -v '^#' .env | xargs) && {{ cargo }} run --bin {{ name }}

alias dev := dev-services
# Start PostgreSQL and RabbitMQ for development
dev-services:
    {{ compose }} -f compose-dev.yaml up -d
# Stop development services
dev-stop:
    {{ compose }} -f compose-dev.yaml down
