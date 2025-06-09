# cinema
Toy project in Rust to pass Big Data and Distributed Processing course. 

An on-line system for managing a cinema. It consits of 3 instances of the server that the users connect to via a load balancer. The database has two replicas.

Users can:
- Register new accounts
- Login into an account
- View a list of all movies
- View their reservations
- Make new reservations
- Cancel singular reservations
- Cancel reservations in bulk
- Change reservations

## Quickstart
To start the load balancer, servers and database replicas, run
```shell
docker-compose up --scale app=3
```
