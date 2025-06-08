import requests
import time
import asyncio
import aiohttp
from aiohttp_retry import RetryClient, ExponentialRetry
import random
import json
import re
from enum import Enum

# Import the new authentication helpers
from auth_helpers import register_and_login_user, BASE_URL

NUM_CLIENTS = 5 # Number of concurrent (book, cancel) pairs
ITERATIONS_PER_CLIENT = 100 # How many times each pair performs the cycle

async def attempt_reservation_and_cancellation(client_id: int, session: RetryClient, schedule_id: int):
    """
    Performs a cycle of attempting to book a reservation and then cancelling it.
    Returns a ResponseType indicating the outcome.
    """
    print(f"Pair {client_id} - Attempting to book for schedule {schedule_id}...")

    try:
        book_data = {"schedule_id": schedule_id}
        async with session.post(f"{BASE_URL}/reservations", data=book_data) as response:
            book_status = response.status
            book_response_text = await response.text()

            if book_status == 200:
                if "Room capacity exceeded for schedule" in book_response_text:
                    print(f"Pair {client_id}: Booking for schedule {schedule_id} resulted in CAPACITY_ERROR. Status: {book_status}")
                    return False
                elif "This user already has a reservation for the selected schedule" in book_response_text:
                    print(f"Pair {client_id}: Booking for schedule {schedule_id} resulted in ALREADY_RESERVED_ERROR. Status: {book_status}")
                    return False
                else:
                    print(f"Pair {client_id}: Successfully booked for schedule {schedule_id}. Status: {book_status}")
                    # await asyncio.sleep(0.1) # Give DB a moment to sync

                    try:
                        async with session.get(f"{BASE_URL}/reservations") as get_reservations_response:
                            # TODO: use code from stress test 5 to get list of reservation, to find new reservation
                            if get_reservations_response.status == 200:
                                all_reservations = await get_reservations_response.json()
                                # Find the reservation belonging to this user for this schedule
                                # Sort to pick the latest if multiple exist (though should be 1 for this test)
                                found_res = next(
                                    (res for res in sorted(all_reservations, key=lambda x: x.get('reservation_id', 0), reverse=True) if res.get('schedule_id') == schedule_id),
                                    None
                                )
                                if found_res:
                                    reservation_id_to_delete = found_res.get('reservation_id')
                                    print(f"Pair {client_id}: Found reservation ID {reservation_id_to_delete} for deletion.")
                                else:
                                    print(f"Pair {client_id}: WARNING: Could not find reservation after successful booking for schedule {schedule_id}. Skipping deletion.")
                                    return False
                            else:
                                print(f"Pair {client_id}: ERROR: Failed to fetch all reservations to find ID. Status: {get_reservations_response.status}")
                                return False
                    except (aiohttp.ClientError, json.JSONDecodeError) as e:
                        print(f"Pair {client_id}: Network/JSON error while fetching reservations for deletion: {e}")
                        return False
            else:
                print(f"Pair {client_id}: Booking for schedule {schedule_id} resulted in SERVER_ERROR. Status: {book_status}, Response: {book_response_text[:200]}...")
                return False

        if reservation_id_to_delete is not None:
            print(f"Pair {client_id}: Attempting to delete reservation {reservation_id_to_delete}...")
            async with session.delete(f"{BASE_URL}/reservations/{reservation_id_to_delete}") as delete_response:
                delete_status = delete_response.status
                delete_response_text = await delete_response.text()
                if delete_status == 200:
                    print(f"Pair {client_id}: Successfully deleted reservation {reservation_id_to_delete}. Status: {delete_status}")
                    return True
                else:
                    print(f"Pair {client_id}: ERROR: Failed to delete reservation {reservation_id_to_delete}. Status: {delete_status}, Response: {delete_response_text[:200]}...")
                    return False
        else:
            return False

    except aiohttp.ClientError as e:
        print(f"Pair {client_id}: Network error during booking/cancellation cycle: {e}")
        return False
    except Exception as e:
        print(f"Pair {client_id}: Unexpected error during booking/cancellation cycle: {e}")
        return False

async def stress_test_4():
    """Performs Stress Test 4: Constant Cancellations and Seat Occupancy."""
    print(f"--- Stress Test 4: Constant Cancellations and Seat Occupancy ({NUM_CLIENTS} pairs, {ITERATIONS_PER_CLIENT} iterations/pair) ---")

    print("\nSetting up authentication context for clients (registering and logging in users for pairs)...")
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

    print("\nStarting booking and cancellation cycles...")
    start_time = time.time()

    success_count = 0
    server_error_count = 0

    retry_options_main = ExponentialRetry(attempts=1, statuses=[500, 502, 503, 504])

    tasks = []
    for i in range(NUM_CLIENTS):
        session, user_id = client_sessions[i]
        schedule_id = random.choice(schedules_ids)

        for iteration in range(ITERATIONS_PER_CLIENT):
            retry_session = RetryClient(client_session=session, raise_for_status=False, retry_options=retry_options_main)
            tasks.append(attempt_reservation_and_cancellation(i, retry_session, schedule_id))

    results = await asyncio.gather(*tasks)

    for result_type in results:
        if result_type:
            success_count += 1
        else:
            server_error_count += 1

    end_time = time.time()
    duration = end_time - start_time

    print("\n--- Results ---")
    print(f"Total pairs attempting cycles: {NUM_CLIENTS}")
    print(f"Total cycles attempted: {NUM_CLIENTS * ITERATIONS_PER_CLIENT}")
    print(f"Successful book-and-cancel cycles: {success_count}")
    print(f"Other Server errors: {server_error_count}")
    print(f"Total duration: {duration:.2f} seconds")

    await asyncio.gather(*[s.close() for s, _ in client_sessions])

    print("---------------------------------------------------\n")

if __name__ == "__main__":
    asyncio.run(stress_test_4())
