import requests
import time
import asyncio
import aiohttp
from aiohttp_retry import RetryClient, ExponentialRetry

BASE_URL = "http://localhost:8080"
ENDPOINT = "/movies" # Or "/reservations" for a slightly heavier GET
NUM_REQUESTS = 1000 # Number of times to hit the endpoint

async def fetch_url(session: RetryClient, url: str):
    """Fetches a URL and returns its status."""
    try:
        async with session.get(url) as response:
            return response.status
    except aiohttp.ClientError as e:
        print(f"Request failed: {e}")
        return None

async def stress_test_1():
    """Performs Stress Test 1: Rapid Single Request."""
    print(f"--- Stress Test 1: Rapid Single Request ({NUM_REQUESTS} requests) ---")
    print(f"Targeting: {BASE_URL}{ENDPOINT}")

    start_time = time.time()
    successful_requests = 0
    failed_requests = 0

    # Configure retry for network resilience
    retry_options = ExponentialRetry(attempts=3, statuses=[500, 502, 503, 504])
    async with RetryClient(raise_for_status=False, retry_options=retry_options) as session:
        tasks = [fetch_url(session, f"{BASE_URL}{ENDPOINT}") for _ in range(NUM_REQUESTS)]
        results = await asyncio.gather(*tasks)

    for status in results:
        if status == 200:
            successful_requests += 1
        else:
            failed_requests += 1

    end_time = time.time()
    duration = end_time - start_time

    print("\n--- Results ---")
    print(f"Total requests: {NUM_REQUESTS}")
    print(f"Successful requests: {successful_requests}")
    print(f"Failed requests: {failed_requests}")
    print(f"Total duration: {duration:.2f} seconds")
    print(f"Requests per second: {NUM_REQUESTS / duration:.2f}")
    print("---------------------------------------------------\n")

if __name__ == "__main__":
    asyncio.run(stress_test_1())
