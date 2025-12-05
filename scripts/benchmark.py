import time
import subprocess
import os
import statistics

# Paths
V2_CMD = ["sensei", "ask", "Hello"] # Assumes v2 is installed in PATH
V3_SERVER_BIN = "./target/release/sensei-server"
V3_CLIENT_BIN = "./target/release/sensei-client"

def measure_execution(cmd, env=None):
    start = time.time()
    try:
        subprocess.run(cmd, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
    except Exception as e:
        return None
    return (time.time() - start) * 1000 # ms

def bench_v3_e2e():
    # Start Server
    server = subprocess.Popen([V3_SERVER_BIN], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env={**os.environ, "GEMINI_API_KEY": "dummy"})
    time.sleep(1) # Wait for startup

    try:
        # Measure Client
        cmd = [V3_CLIENT_BIN, "--ask", "Hello"]
        times = []
        for _ in range(10):
            t = measure_execution(cmd)
            if t: times.append(t)
        return times
    finally:
        server.kill()

def bench_v2():
    times = []
    # Check if sensei v2 exists
    if subprocess.call(["which", "sensei"], stdout=subprocess.DEVNULL) != 0:
        print("⚠️ Sensei v2 not found in PATH. Skipping.")
        return []

    for _ in range(5): # V2 is slower, less runs
        t = measure_execution(V2_CMD)
        if t: times.append(t)
    return times

if __name__ == "__main__":
    print("🏎️  Sensei Benchmark (V2 Python vs V3 Rust)\n")

    # Build V3 Release
    print("🔨 Building V3 Release...")
    subprocess.run(["cargo", "build", "--release", "--quiet"], check=True)

    # Bench V3
    print("MEASURING V3 (Client -> Server)...")
    v3_times = bench_v3_e2e()
    if v3_times:
        print(f"✅ V3 Mean: {statistics.mean(v3_times):.2f} ms")

    # Bench V2
    print("\nMEASURING V2 (Python Monolith)...")
    v2_times = bench_v2()
    if v2_times:
        print(f"✅ V2 Mean: {statistics.mean(v2_times):.2f} ms")

        speedup = statistics.mean(v2_times) / statistics.mean(v3_times)
        print(f"\n🚀 V3 is {speedup:.1f}x faster than V2!")
    else:
        print("\nCould not benchmark V2 (not installed).")
