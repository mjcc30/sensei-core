#!/usr/bin/env python3
import os
import json
import urllib.request

api_key = os.getenv("GEMINI_API_KEY")
if not api_key:
    print("❌ GEMINI_API_KEY not set")
    exit(1)

url = f"https://generativelanguage.googleapis.com/v1beta/models?key={api_key}"

try:
    with urllib.request.urlopen(url) as response:
        data = json.load(response)

    print("🔍 Available Models:")
    if "models" in data:
        for m in data["models"]:
            name = m["name"].replace("models/", "")
            methods = m.get("supportedGenerationMethods", [])
            print(f"- {name:<30} {methods}")
    else:
        print("⚠️ No models found in response.")

except Exception as e:
    print(f"❌ Error: {e}")
