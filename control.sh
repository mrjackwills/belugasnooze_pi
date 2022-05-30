#!/bin/bash

# v0.0.5

DOCKER_GUID=$(id -g)
DOCKER_UID=$(id -u)
DOCKER_TIME_CONT="America"
DOCKER_TIME_CITY="New_York"

DOCKER_GPIO=$(getent group gpio | cut -d: -f3)

RED='\033[0;31m'
YELLOW='\033[0;33m'
RESET='\033[0m'

error_close() {
	echo -e "\n${RED}ERROR - EXITED: ${YELLOW}$1${RESET}\n";
	exit 1
}

if ! [ -x "$(command -v dialog)" ]; then
	error_close "dialog is not installed"
fi


production_up () {
	DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	DOCKER_GPIO=${DOCKER_GPIO} \
	DOCKER_BUILDKIT=0 \
	docker compose up -d
}

production_down () {
	DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	DOCKER_GPIO=${DOCKER_GPIO} \
	DOCKER_BUILDKIT=0 \
	docker compose -f docker-compose.yml down
}

production_rebuild () {
	make_all_directories
	DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	DOCKER_GPIO=${DOCKER_GPIO} \
	DOCKER_BUILDKIT=0 \
	docker compose up -d --build
}

main() {
	cmd=(dialog --backtitle "Start container" --radiolist "choose environment" 14 80 16)
	options=(
		1 "up" off
		2 "down" off
		3 "rebuild" off
	)
	choices=$("${cmd[@]}" "${options[@]}" 2>&1 >/dev/tty)
	exitStatus=$?
	clear
	if [ $exitStatus -ne 0 ]; then
		exit
	fi
	for choice in $choices
	do
		case $choice in
			0)
				exit
				break;;
			1)
				production_up
				break;;
			2)
				production_down
				break;;
			3)
				production_rebuild
				break;;
		esac
	done
}

main