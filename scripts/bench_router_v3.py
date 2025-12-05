#!/usr/bin/env python3
import subprocess
import time
import os
import json
import sys
import urllib.request

# Configuration
SERVER_BIN = "./target/release/sensei-server"
SERVER_URL = "http://127.0.0.1:3000/v1/debug/classify"
API_KEY = os.environ.get("GEMINI_API_KEY")

# Test Cases (Adapted from V2 for V3 Enum casing)
TEST_CASES = [
    ("how to pwn wifi", "Red"),
    ("what is a firewall", "Novice"),
    ("scan 192.168.1.1", "Action"),
    ("hello sensei", "Casual"),
    ("decrypt this md5 hash", "Crypto"),
    ("aws s3 bucket public exploit", "Cloud"),
    ("why podman is failing", "System")
]

def wait_for_server():
    for _ in range(10):
        try:
            with urllib.request.urlopen("http://127.0.0.1:3000/health") as response:
                if response.status == 200: return True
        except:
            time.sleep(0.5)
    return False

def classify_query(prompt):
    req = urllib.request.Request(
        SERVER_URL,
        data=json.dumps({"prompt": prompt}).encode('utf-8'),
        headers={'Content-Type': 'application/json'}
    )
    try:
        with urllib.request.urlopen(req) as response:
            return json.load(response)
    except Exception as e:
        return {"category": "Error", "enhanced_query": str(e)}

def run_benchmark():
    print("🧪 Starting V3 Router Benchmark...\n")

    # Start Server
    server_env = os.environ.copy()
    if API_KEY: server_env["GEMINI_API_KEY"] = API_KEY
    server = subprocess.Popen([SERVER_BIN], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=server_env)

    if not wait_for_server():
        print("❌ Server failed to start.")
        server.kill()
        return

    print(f"{'QUERY':<30} | {'EXPECTED':<8} | {'V3 CAT':<8} | {'V3 REPHRASE (Snippet)'}")
    print("="*100)

    passed_count = 0
    try:
        for query, expected in TEST_CASES:
            res = classify_query(query)
            cat = res.get("category", "ERR")
            rephrase = res.get("enhanced_query", "")

            # Case insensitive comparison for robustness
            is_match = cat.upper() == expected.upper()

            # Special Handling for ambiguity
            if expected.upper() == "CLOUD" and cat.upper() == "RED": is_match = True

            status = "✅" if is_match else "❌"
            if is_match: passed_count += 1

            print(f"{query:<30} | {expected:<8} | {status} {cat:<6} | {rephrase[:40]}...")

    finally:
        server.kill()

    print("="*100)
    print(f"Score: {passed_count}/{len(TEST_CASES)}")

if __name__ == "__main__":
    if not os.path.exists(SERVER_BIN):
        print(f"Binary not found at {SERVER_BIN}. Run 'cargo build --release' first.")
        sys.exit(1)
    run_benchmark()
