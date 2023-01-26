# Exchangy: Knox Rust Programing Challenge

Implements a small HTTP service to convert amounts from one currency to another.

# How to run locally 

```bush
cargo run
```
# Using the service

The following command will convert 100 USD to Euros:  
```bash
curl -v localhost:8080/currency --header 'Content-Type: application/json' \
-d '{ "to": "France", "from": "USA", "amount": 100 }'


*   Trying 127.0.0.1:8080...
* Connected to localhost (127.0.0.1) port 8080 (#0)
> POST /currency HTTP/1.1
> Host: localhost:8080
> User-Agent: curl/7.87.0
> Accept: */*
> Content-Type: application/json
> Content-Length: 48
> 
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< content-type: application/json
< content-length: 43
< date: Thu, 26 Jan 2023 20:06:34 GMT
< 
* Connection #0 to host localhost left intact
{"from":"USA","to":"France","amount":91.72}%
```
Please note, the `Content-Type` header should be set to the proper type, otherwise the service will not be able to process the request:
```bash
curl -v localhost:8080/currency \                                                                 ✔ 
-d '{ "to": "France", "from": "USA", "amount": 100 }'

*   Trying 127.0.0.1:8080...
* Connected to localhost (127.0.0.1) port 8080 (#0)
> POST /currency HTTP/1.1
> Host: localhost:8080
> User-Agent: curl/7.87.0
> Accept: */*
> Content-Length: 48
> Content-Type: application/x-www-form-urlencoded
> 
* Mark bundle as not supporting multiuse
< HTTP/1.1 415 Unsupported Media Type
< content-type: text/plain; charset=utf-8
< content-length: 54
< date: Thu, 26 Jan 2023 20:15:10 GMT
< 
* Connection #0 to host localhost left intact
Expected request with `Content-Type: application/json`%  
```

The fields `from` and `to` should be valid country names recognized by the Rest Countries API. In cases when the name is not valid, a 500 response will be returned:
```bash
curl -v localhost:8080/currency --header 'Content-Type: application/json' \                       ✔ 
-d '{ "to": "France", "from": "Narnia", "amount": 100 }'

*   Trying 127.0.0.1:8080...
* Connected to localhost (127.0.0.1) port 8080 (#0)
> POST /currency HTTP/1.1
> Host: localhost:8080
> User-Agent: curl/7.87.0
> Accept: */*
> Content-Type: application/json
> Content-Length: 51
> 
* Mark bundle as not supporting multiuse
< HTTP/1.1 500 Internal Server Error
< content-type: text/plain; charset=utf-8
< content-length: 46
< date: Thu, 26 Jan 2023 20:09:57 GMT
< 
* Connection #0 to host localhost left intact
Something went wrong: bad country name: Narnia%        
```

The `amount` field should contain a valid numeric value, otherwise a 400 response will be returned:
```bash
curl -v localhost:8080/currency --header 'Content-Type: application/json' \                       ✔ 
-d '{ "to": "France", "from": "USA", "amount": bla }'   

*   Trying 127.0.0.1:8080...
* Connected to localhost (127.0.0.1) port 8080 (#0)
> POST /currency HTTP/1.1
> Host: localhost:8080
> User-Agent: curl/7.87.0
> Accept: */*
> Content-Type: application/json
> Content-Length: 48
> 
* Mark bundle as not supporting multiuse
< HTTP/1.1 400 Bad Request
< content-type: text/plain; charset=utf-8
< content-length: 84
< date: Thu, 26 Jan 2023 20:11:45 GMT
< 
* Connection #0 to host localhost left intact
Failed to parse the request body as JSON: amount: expected value at line 1 column 44%           
```

