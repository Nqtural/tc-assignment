#!/bin/env sh
curl -X GET 0.0.0.0:3000/rooms/get \
	-H "Content-Type: application/json" \
	-b cookies.txt
