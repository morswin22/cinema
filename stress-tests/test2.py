import time
import asyncio
import aiohttp
from aiohttp_retry import RetryClient, ExponentialRetry
import random
from collections import defaultdict
from auth_helpers import register_and_login_user, BASE_URL

NUM_CLIENTS = 10
REQUESTS_PER_CLIENT = 500

async def perform_random_request(session: RetryClient):
    """
    A client performs a random request and records which server handled it.
    server_hit_counts is a shared defaultdict to track server distribution.
    """
    successful_requests = 0
    failed_requests = 0

    endpoints = [
        ("GET", "/movies"),
        ("GET", "/reservations"),
        ("GET", "/reservations/new")
    ]

    for _ in range(REQUESTS_PER_CLIENT):
        method, path = random.choice(endpoints)
        url = f"{BASE_URL}{path}"
        status = None

        try:
            if method == "GET":
                async with session.get(url) as response:
                    status = response.status

            if status in [200, 303, 201]: # 200 OK, 303 See Other for redirect, 201 Created
                successful_requests += 1
            else:
                failed_requests += 1
                # print(f"Client {client_id}: Request {method} {url} failed with status {status}")

        except aiohttp.ClientError as e:
            # print(f"Client {client_id}: Request {method} {url} failed due to network error: {e}")
            failed_requests += 1

    return successful_requests, failed_requests

async def stress_test_2():
    """Performs Stress Test 2: Random Concurrent Requests with Load Balancer Testing."""
    print(f"--- Stress Test 2: Random Concurrent Requests (Load Balancer Test) ---")
    print(f"Clients: {NUM_CLIENTS}, Requests per Client: {REQUESTS_PER_CLIENT}")
    print(f"Targeting Load Balancer at: {BASE_URL}")

    # --- Setup Auth Context for each client ---
    print("\nSetting up authentication context for clients (registering and logging in users)...")
    client_sessions = [] # Stores aiohttp.ClientSession for each client
    auth_tasks = [register_and_login_user(aiohttp.ClientSession(), f"client_{i}") for i in range(NUM_CLIENTS)]
    results = await asyncio.gather(*auth_tasks)

    # Store sessions only if registration and login were successful
    for session_obj in results:
        if session_obj: # Only append session if login was successful
            client_sessions.append(session_obj)

    NUM_CLIENTS_ACTUAL = len(client_sessions)
    if NUM_CLIENTS_ACTUAL == 0:
        print("FATAL: No clients successfully authenticated. Cannot proceed with test.")
        return
    print(f"Proceeding with {NUM_CLIENTS_ACTUAL} authenticated clients.")

    print("\nStarting random request phase...")
    start_time = time.time()
    total_successful = 0
    total_failed = 0

    retry_options_main = ExponentialRetry(attempts=3, statuses=[500, 502, 503, 504])

    tasks = []
    for i in range(NUM_CLIENTS_ACTUAL):
        session = client_sessions[i]
        retry_session = RetryClient(client_session=session, raise_for_status=False, retry_options=retry_options_main)
        tasks.append(perform_random_request(retry_session))

    results = await asyncio.gather(*tasks)

    for success, fail in results:
        total_successful += success
        total_failed += fail

    end_time = time.time()
    duration = end_time - start_time
    total_requests_attempted = NUM_CLIENTS_ACTUAL * REQUESTS_PER_CLIENT

    print("\n--- Test Results ---")
    print(f"Total requests attempted: {total_requests_attempted}")
    print(f"Total successful requests: {total_successful}")
    print(f"Total failed requests: {total_failed}")
    print(f"Total duration: {duration:.2f} seconds")
    if duration > 0:
        print(f"Requests per second: {total_requests_attempted / duration:.2f}")
    else:
        print("Requests per second: N/A (duration too short)")

    # Close all client sessions
    await asyncio.gather(*[s.close() for s in client_sessions])

    print("---------------------------------------------------\n")

if __name__ == "__main__":
    asyncio.run(stress_test_2())
