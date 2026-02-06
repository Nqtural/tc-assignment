#!/bin/env sh
curl -X POST 0.0.0.0:3000/auth/login \
	-c cookies.txt \
	-H "Content-Type: application/json" \
	-d '
{
	"password": "Pass123",
	"email": "email2@example.com"
}
'
