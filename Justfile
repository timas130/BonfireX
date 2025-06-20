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
debugger := "bs"

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
	set -a && source .env && set +a
	{{ cargo }} run --bin {{ name }}

test-one name:
	#!/usr/bin/env bash
	set -a && source .env && set +a
	{{ cargo }} test -p {{ name }}
bench-one name:
	#!/usr/bin/env bash
	set -a && source .env && set +a
	{{ cargo }} bench -p {{ name }}

alias dev := dev-services
# Start PostgreSQL and RabbitMQ for development
dev-services:
    {{ compose }} -f compose-dev.yaml up -d

alias stop := dev-stop
# Stop development services
dev-stop:
    {{ compose }} -f compose-dev.yaml down

# Run all services
everything:
	#!/usr/bin/env bash
	set -e
	set -a && source .env && set +a
	{{ cargo }} build
	PORT=5000 target/debug/bfx-rpc-router &
	sleep 1
	PORT=8000 target/debug/bfx-graphql &
	for app in target/debug/bfx-*; do
	    if [[ "$app" = "bfx-graphql" ]]; then
	    	:
	    elif [[ "$app" = "bfx-rpc-router" ]]; then
	    	:
		elif [[ "$app" = "bfx-translation-writer" ]]; then
			:
		elif [[ "$app" == *.d ]]; then
			:
		else
			$app &
		fi
	done
	trap 'kill -TERM $(jobs -p)' SIGINT
	trap 'kill -TERM $(jobs -p)' SIGTERM
	wait
