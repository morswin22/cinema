import aiohttp
import requests # For synchronous initial data fetching if needed

BASE_URL = "http://localhost:8080"

async def register_and_login_user(session: aiohttp.ClientSession, username: str):
    """
    Registers a new user and then logs them in using the provided aiohttp session.
    Returns the user's database ID (obtained from registration response) and the session (which now holds the cookie).
    """
    email = f"{username}@example.com"
    password = "test_password_secure"

    register_data = {"email": email, "password": password, "password_confirmation": password}
    print(f"[{username}] Attempting to register user: {email} via /register...")
    try:
        async with session.post(f"{BASE_URL}/register", data=register_data) as response:
            response_text = await response.text()
            if response.status == 200:
                print(f"[{username}] User {email} registered successfully")
            else:
                print(f"[{username}] User registration failed. Status: {response.status}, Response: {response_text[:200]}")

    except aiohttp.ClientError as e:
        print(f"[{username}] Network error during user registration: {e}")
        return None

    # --- Step 2: Login User (via POST /login to establish session cookie) ---
    login_data = {"email": email, "password": password}
    print(f"[{username}] Attempting to login user: {email} via /login...")
    try:
        async with session.post(f"{BASE_URL}/login", data=login_data) as response: # Target /auth/login
            if response.status == 200:
                print(f"[{username}] User login successful. Session cookie should be handled by aiohttp session.")
                return session
            else:
                error_text = await response.text()
                print(f"[{username}] User login failed. Status: {response.status}, Response: {error_text[:200]}")
                return None
    except aiohttp.ClientError as e:
        print(f"[{username}] Network error during user login: {e}")
        return None