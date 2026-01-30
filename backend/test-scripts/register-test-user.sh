#!/bin/env sh
curl -X POST 0.0.0.0:3000/auth/register \
	-H "Content-Type: application/json" \
	-d '
{
	"name": "First",
	"surname": "Last",
	"password": "Pass123",
	"email": "email@example.com"
}
'
