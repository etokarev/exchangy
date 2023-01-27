# Exchangy: Knox Rust Programing Challenge

Implements a small HTTP service to convert amounts from one currency to another.

# How to run locally 

Run `cargo run` command in the application folder to start the service: 

```shell
cargo run                                                                                         ✔ 
   Compiling exchangy v0.1.0 (/home/tev/src/exchangy)
    Finished dev [unoptimized + debuginfo] target(s) in 1.91s
     Running `target/debug/exchangy`
2023-01-26T20:28:14.706603Z  INFO exchangy: listening on 127.0.0.1:8080
```
# Using the service

The following command will convert 100 USD to Euros:  
```shell
curl -v localhost:8080/currency --header 'Content-Type: application/json' \                          ✔ 
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
< content-length: 40
< date: Thu, 26 Jan 2023 20:25:31 GMT
< 
* Connection #0 to host localhost left intact
{"from":"USD","to":"EUR","amount":91.72}%    
```
Please note, the `Content-Type` header should be set to the proper type, otherwise the service will not be able to process the request:
```shell
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
```shell
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
```shell
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

# Implementation details and possible bottlenecks

- Error handling. Currently `AppError` struct is used for error handling. In case if any error is encountered, a 500 response will be returned along with the error details. The approach limits to the way how errors are handled and a better approach would be making `AppError` of `enum` type and changing `into_response` method in `AppError`'s implementation to generate a different response depending on the error type. For example, that will allow us to return 400 error when a bad country is sent. 
- Concurrency. The web framework I chose to use in this app, axum, is built on the top of async HTTP implementation in Rust called hyper. As I far as I can tell, web applications built on the top of axum are concurrent by the nature.  They are capable to serve multiple requests concurrently, and the request throughput can be very high. My first impression from Rust is that the developer should be careful with IO blocking operations. As long as they get executed asynchronously the general application performance will not be affected.

  I added a special delay in the web handler to sleep for 30 secs if the `to` field is set to `Russia`.  This proves that one request cannot block serving other requests executed concurrently. 
- Dependency injection. Typically, I prefer to use the classic DI pattern in web applications where the application logic is decoupled from the data access logic via interface usage. So a concrete implementation is injected as a dependency at the initialization step. However, I've run into the limitations of my current understanding of Rust pretty quickly. I understand traits should be used and axum provides ways to share the application state as it is described [here](https://docs.rs/axum/latest/axum/#sharing-state-with-handlers).  I could not get everything work, so I just call static methods for data access in the handler.        
- Caching. I've implemented a simple caching of the country data to save on Rest Country API usage. I picked `dashmap` crate as a good concurrent HahMap implementation. It is pretty simple cache with no limitation on the memory usage and no eviction policy support ( I think). So in the production usage we'd have to think about things like limiting the cache on the total memory usage or total items and implement memory eviction policies.
- Rate Limiting. The best way to implement rate limiting in API would be introducing API keys like ExchangeRate service did. It would allow us to use the API key as a counter and be able to implement different rate limiting policies for different API consumers. From the technical perspective, I'd probably use DashMap again since I  think it can provide concurrent read and write access to the hasp map in the way when different keys can be updated concurrently at the same time.       
- Multi currency countries. In order to handle multi currency countries, we'll have to change our API first. I'd propose changing the `to` field in the response to an array of objects. Something like this:
```json
{
    "from": "USD",
    "to": [
        {
            "currency": "EUR",
            "amount": 91.72
        },
        {
            "currency": "GBP",
            "amount": 85.72
        }
    ]
}
```
This will allow us to convert the same amount into multiple destination currencies used in a given country.

- Further performance improvements. The ExchangeRate API calls are not currently cached so adding caching of the exchange rates data would be the lowest hanging fruit in terms of improving app performance.    