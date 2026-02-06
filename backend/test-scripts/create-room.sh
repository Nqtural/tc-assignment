#!/bin/env sh
curl -X POST 0.0.0.0:3000/rooms/create \
	-H "Content-Type: application/json" \
	-b cookies.txt \
	-d '
{
	"name": "Room name",
	"description": "Room description"
}
'
