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

Then open [`localhost:8080`](http://localhost:8080/) to view the GUI.

## Stress tests
To run the stress tests, using Python 3.12 with installed `requests`, `aiohttp`, and `aiohttp_retry` PyPI packages, in `stress-tests` directory, run
```shell
python test1.py
python test2.py
python test3.py
python test4.py
python test5.py
```
