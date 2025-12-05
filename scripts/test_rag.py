#!/usr/bin/env python3
import subprocess
import time
import os
import sys

SERVER_BIN = "./target/release/sensei-server"
CLIENT_BIN = "./target/release/sensei-client"
SECRET_FILE = "secret_plans.txt"
SECRET_CONTENT = "CONFIDENTIAL: The nuclear launch code is 999-XYZ."

def run_test():
    print("🧪 Starting RAG Integration Test...\n")

    # 1. Setup
    with open(SECRET_FILE, "w") as f:
        f.write(SECRET_CONTENT)

    # Start Server with Logging
    server_log = open("server.log", "w")
    server = subprocess.Popen([SERVER_BIN], stdout=server_log, stderr=server_log)
    time.sleep(2) # Wait for startup

    try:
        # 2. Ingest Document
        print(f"📄 Adding {SECRET_FILE}...")
        res_add = subprocess.run(
            [CLIENT_BIN, "add", SECRET_FILE],
            capture_output=True, text=True
        )

        if res_add.returncode != 0:
            print(f"❌ Ingestion Failed: {res_add.stderr}")
            print(f"   Stdout: {res_add.stdout}")
        else:
            print("✅ Ingestion Command Sent")

        # 3. Ask Question
        print("❓ Asking question...")
        res_ask = subprocess.run(
            [CLIENT_BIN, "ask", "What is the launch code?"],
            capture_output=True, text=True
        )

        print(f"🤖 Answer:\n{res_ask.stdout}")

        if "999-XYZ" in res_ask.stdout:
            print("✅ RAG SUCCESS! Secret found in answer.")
            return True
        else:
            print("❌ RAG FAILURE. Secret not found.")
            return False

    finally:
        server.kill()
        server_log.close()

        if os.path.exists("server.log"):
            print("\n--- SERVER LOGS ---")
            with open("server.log", "r") as f:
                print(f.read())
            print("-------------------")
            os.remove("server.log")

        if os.path.exists(SECRET_FILE): os.remove(SECRET_FILE)

if __name__ == "__main__":
    if not os.path.exists(SERVER_BIN) or not os.path.exists(CLIENT_BIN):
        print("🔨 Building binaries...")
        subprocess.run(["cargo", "build", "--release", "--quiet"])

    success = run_test()
    sys.exit(0 if success else 1)
