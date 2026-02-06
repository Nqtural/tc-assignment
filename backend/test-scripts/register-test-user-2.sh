#!/bin/env sh
curl -X POST 0.0.0.0:3000/auth/register \
	-H "Content-Type: application/json" \
	-d '
{
	"name": "First 2",
	"surname": "Last 2",
	"password": "Pass123",
	"email": "email2@example.com"
}
'
