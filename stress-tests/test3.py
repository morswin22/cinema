import requests
import time
import asyncio
import aiohttp
from aiohttp_retry import RetryClient, ExponentialRetry
import random
import re
from enum import Enum

from auth_helpers import register_and_login_user, BASE_URL

class ResponseType(Enum):
    SUCCESS = 0
    CAPACITY_ERROR = 1
    ALREADY_RESERVED_ERROR = 2
    SERVER_ERROR = 3



TARGET_CAPACITY = 1 # The capacity of the schedule we'll target for this test (assumed to be 1)

async def attempt_reservation(client_id: int, session: RetryClient, schedule_id: int):
    """
    Attempts to make a reservation using the provided authenticated session.
    No user_id is passed in the payload as it's managed by the session on the backend.
    """
    data = {"schedule_id": schedule_id}
    try:
        async with session.post(f"{BASE_URL}/reservations", data=data) as response:
            status = response.status
            response_text = await response.text()
            if status == 200 and not "Room capacity exceeded for schedule" in response_text and not "This user already has a reservation for the selected schedule" in response_text:
                print(f"Client {client_id}: Successfully reserved seat for schedule {schedule_id}. Status: {status}")
                return ResponseType.SUCCESS
            else:
                print(f"Client {client_id}: Failed to reserve seat for schedule {schedule_id}. Status: {status}, Response: {response_text[:200]}...")

                if "Room capacity exceeded for schedule" in response_text:
                    return ResponseType.CAPACITY_ERROR
                elif "This user already has a reservation for the selected schedule" in response_text:
                    return ResponseType.ALREADY_RESERVED_ERROR
                else:
                    return ResponseType.SERVER_ERROR
    except aiohttp.ClientError as e:
        print(f"Client {client_id}: Network error during reservation attempt: {e}")
        return False

async def stress_test_3():
    """Performs Stress Test 3: Immediate Occupancy of All Seats."""
    print(f"--- Stress Test 3: Immediate Occupancy of All Seats (2 clients, capacity={TARGET_CAPACITY}) ---")

    print("Setting up test environment (registering and logging in users)...")
    client_sessions = []

    try:
        auth_tasks = [register_and_login_user(aiohttp.ClientSession(), f"race_client_{i}") for i in range(2)]
        results = await asyncio.gather(*auth_tasks)

        for session_obj in results:
            if session_obj:
                client_sessions.append(session_obj)

        if len(client_sessions) < 2:
            print(f"FATAL: Only {len(client_sessions)} clients registered and logged in. Cannot proceed with race test.")
            await asyncio.gather(*[s.close() for s in client_sessions])
            return

    except requests.exceptions.RequestException as e:
        print(f"FATAL: Failed to set up test environment: {e}. Aborting test.")
        await asyncio.gather(*[s.close() for s in client_sessions])
        return

    schedules_ids = []

    async with client_sessions[0].get(f"{BASE_URL}/reservations/new") as response:
        regex = r"option value=\"(\d+)\""
        response_text = await response.text()
        matches = re.finditer(regex, response_text, re.MULTILINE)
        for matchNum, match in enumerate(matches, start=1):
            for groupNum in range(0, len(match.groups())):
                groupNum = groupNum + 1
            schedules_ids.append(int(match.group(groupNum)))

    print("\nStarting reservation race...")
    start_time = time.time()
    successful_reservations = 0
    capacity_error_reservations = 0
    already_reserved_error_reservations = 0
    server_error_reservation = 0

    # Retries are minimal for this race test, as 400 (capacity exceeded) is an expected outcome.
    retry_options_main = ExponentialRetry(attempts=1, statuses=[500, 502, 503, 504])

    tasks = []
    for i in range(len(client_sessions)):
        for schedule_id in schedules_ids:
            session = client_sessions[i]
            retry_session = RetryClient(client_session=session, raise_for_status=False, retry_options=retry_options_main)
            tasks.append(attempt_reservation(i, retry_session, schedule_id))

    results = await asyncio.gather(*tasks)

    for response in results:
        match response:
            case ResponseType.SUCCESS:
                successful_reservations += 1
            case ResponseType.CAPACITY_ERROR:
                capacity_error_reservations += 1
            case ResponseType.ALREADY_RESERVED_ERROR:
                already_reserved_error_reservations += 1
            case ResponseType.SERVER_ERROR:
                server_error_reservation += 1

    end_time = time.time()
    duration = end_time - start_time

    print("\n--- Results ---")
    print(f"Clients attempting reservations: {len(client_sessions)}")
    print(f"Total successful reservations: {successful_reservations}")
    print(f"Total capacity exceeded errors: {capacity_error_reservations}")
    print(f"Total already reserved reservations: {already_reserved_error_reservations}")
    print(f"Total server errors: {server_error_reservation}")
    print(f"Total duration: {duration:.2f} seconds")

    if successful_reservations == TARGET_CAPACITY:
        print("Outcome: All seats were successfully occupied. Concurrency handled as expected.")
    elif successful_reservations < TARGET_CAPACITY:
        print(f"Outcome: Only {successful_reservations} out of {TARGET_CAPACITY} seats were occupied. This means some clients were correctly blocked due to capacity.")
    else:
        print("Outcome: Unexpected result (more reservations than capacity). This indicates a critical issue.")

    await asyncio.gather(*[s.close() for s in client_sessions])

    print("---------------------------------------------------\n")

if __name__ == "__main__":
    asyncio.run(stress_test_3())
