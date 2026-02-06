#!/bin/env sh
curl -X POST 0.0.0.0:3000/auth/login \
	-c cookies.txt \
	-H "Content-Type: application/json" \
	-d '
{
	"password": "NOT-THE-TEST-USER-PASSWORD",
	"email": "email@example.com"
}
'
