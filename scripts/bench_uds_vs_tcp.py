#!/usr/bin/env python3
import subprocess
import time
import os
import sys

SERVER_BIN = "./target/release/sensei-server"
CLIENT_BIN = "./target/release/sensei-client"
UDS_PATH = "/tmp/sensei-bench.sock"
TCP_PORT = "3001"
ITERATIONS = 50

def run_server(mode):
    env = os.environ.copy()
    env["GEMINI_API_KEY"] = "dummy" # Not needed for health check but required for startup
    
    if mode == "UDS":
        env["SENSEI_LISTEN_ADDR"] = f"unix://{UDS_PATH}"
    else:
        env["SENSEI_LISTEN_ADDR"] = f"0.0.0.0:{TCP_PORT}"
        
    # Start server
    server = subprocess.Popen([SERVER_BIN], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env)
    time.sleep(2) # Warmup
    return server

def measure_client(url):
    start = time.time()
    # We invoke the client to ask a simple question.
    # Note: We can't easily hit /health with the CLI client as it's hardcoded to /v1/ask or add.
    # We will assume /v1/ask with a mock LLM would be better, but we don't have mock LLM easily configurable here without env vars.
    # Actually, the server will try to init LLM. 
    # Let's just run a failing command (invalid arg) or similar? No.
    # Let's run a simple ask. It will fail on LLM (dummy key) but the transport will happen.
    # Or better: We assume the handshake overhead is what we want to measure.
    
    # Actually, let's use the CLI's --ask feature. The LLM error "Invalid API Key" comes from Google, 
    # so it means we did the full roundtrip to the server.
    try:
        subprocess.run(
            [CLIENT_BIN, "--url", url, "--ask", "Ping"],
            stdout=subprocess.DEVNULL, 
            stderr=subprocess.DEVNULL,
            timeout=5
        )
    except Exception:
        pass # Expected failure due to dummy key, but transport occurred
        
    return (time.time() - start) * 1000 # ms

def run_benchmark():
    print(f"🏎️  Benchmark: TCP vs UDS ({ITERATIONS} iterations)")
    print("------------------------------------------------")

    # 1. Measure TCP
    print("Testing TCP (127.0.0.1)...")
    srv_tcp = run_server("TCP")
    tcp_times = []
    try:
        for i in range(ITERATIONS):
            t = measure_client(f"http://127.0.0.1:{TCP_PORT}")
            tcp_times.append(t)
            print(f"\rProgress: {i+1}/{ITERATIONS}", end="")
    finally:
        srv_tcp.kill()
    
    avg_tcp = sum(tcp_times) / len(tcp_times)
    print(f"\n✅ TCP Avg Latency (End-to-End): {avg_tcp:.2f} ms")

    # 2. Measure UDS
    print("\nTesting UDS (Unix Socket)...")
    srv_uds = run_server("UDS")
    uds_times = []
    try:
        for i in range(ITERATIONS):
            t = measure_client(f"unix://{UDS_PATH}")
            uds_times.append(t)
            print(f"\rProgress: {i+1}/{ITERATIONS}", end="")
    finally:
        srv_uds.kill()
        if os.path.exists(UDS_PATH): os.remove(UDS_PATH)

    avg_uds = sum(uds_times) / len(uds_times)
    print(f"\n✅ UDS Avg Latency (End-to-End): {avg_uds:.2f} ms")

    # Conclusion
    diff = avg_tcp - avg_uds
    percent = (diff / avg_tcp) * 100
    print("\n------------------------------------------------")
    if diff > 0:
        print(f"🏆 UDS is {diff:.2f} ms faster ({percent:.1f}%) per call!")
    else:
        print(f"TCP was faster by {-diff:.2f} ms (Noise/Overhead dominates)")

if __name__ == "__main__":
    if not os.path.exists(SERVER_BIN):
        print("Build release first!")
        sys.exit(1)
    run_benchmark()
