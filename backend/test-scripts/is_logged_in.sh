#!/bin/env sh
curl -X POST 0.0.0.0:3000/auth/is_logged_in \
	-b cookies.txt \
	-H "Content-Type: application/json"
