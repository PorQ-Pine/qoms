#!/bin/bash
exec >/dev/null 2>&1
mkdir -p /tmp/qoms/
printf '%s\n' "$GREETD_SOCK" > /tmp/qoms/greetd_sock_path.txt
sleep infinity
