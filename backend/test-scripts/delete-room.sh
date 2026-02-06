#!/bin/env sh
if [ "$1" != "" ]; then
	curl -X DELETE 0.0.0.0:3000/rooms/$1/delete \
		-H "Content-Type: application/json" \
		-b cookies.txt
else
	echo "Usage: $0 <room id>"
fi
