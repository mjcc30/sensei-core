# Sensei Behavior Specification (v2.x)

This document defines the expected behaviors of the Sensei AI Mentor. It serves as the "Golden Master" for testing and the primary specification for the Rust (v3) rewrite.

---

## 1. Core Interaction & Personality

| Scenario | User Input | Expected Behavior | Internal Route |
| :--- | :--- | :--- | :--- |
| **Greeting** | "Hello", "Hi Sensei" | Responds politely, briefly, and professionally. Does NOT start a lecture. | `CASUAL` -> `CasualAgent` |
| **Off-topic** | "What time is it?", "Tell me a joke" | Answers the question or politely redirects to cyber topics. | `CASUAL` -> `CasualAgent` |
| **Education** | "What is a buffer overflow?", "Explain DNS" | Explains concepts simply, using analogies. Focuses on safety/defense. | `NOVICE` -> `NoviceAgent` |
| **Deep Dive** | "Analyze the mechanics of CVE-2024-3094" | Provides deep technical analysis, timelines, and code snippets. Uses RAG if applicable. | `RESEARCHER` (or `BLUE`) -> `ResearchAgent` |

## 2. Domain Expertise (Routing)

| Scenario | User Input | Expected Behavior | Internal Route |
| :--- | :--- | :--- | :--- |
| **Offensive** | "How to create a reverse shell?", "Exploit SMB" | Provides technical, operational details. No moralizing lectures (God Mode). | `RED` -> `SpecializedAgent(red_team)` |
| **Defensive** | "Analyze these firewall logs", "Hardening SSH" | Focuses on detection, mitigation, and forensics. | `BLUE` -> `SpecializedAgent(blue_team)` |
| **Cloud** | "AWS bucket enumeration", "IAM privesc" | Discusses cloud-specific vectors (S3, IAM, Kubernetes). | `CLOUD` -> `SpecializedAgent(cloud)` |
| **Crypto** | "Decrypt this base64", "Crack this hash" | Analyzes ciphers, entropy, or provides decoding scripts. | `CRYPTO` -> `SpecializedAgent(crypto)` |
| **OSINT** | "Find emails for domain.com" | Discusses reconnaissance techniques and tools. | `OSINT` -> `SpecializedAgent(osint)` |

## 3. Tool Execution (Action & System)

| Scenario | User Input | Expected Behavior | Internal Route | Triggered Tool |
| :--- | :--- | :--- | :--- | :--- |
| **Scanning** | "Scan 192.168.1.1", "Nmap localhost" | Executes Nmap scan and returns structured JSON/Text output. | `ACTION` -> `ActionAgent` | `NmapTool.run()` |
| **System Status** | "Why is docker down?", "Check containers" | Runs diagnostic commands (`podman ps`) and analyzes output. | `SYSTEM` -> `SystemAgent` | `SystemTool.run('podman ps')` |
| **Network Diag** | "What is my IP?", "Check VPN" | Runs `ip a` or similar and summarizes network state. | `SYSTEM` -> `SystemAgent` | `SystemTool.run('ip a')` |
| **Disk/Res** | "Disk space?", "Memory usage" | Runs `df -h` or `free -m` and reports usage. | `SYSTEM` -> `SystemAgent` | `SystemTool.run('df/free')` |

## 4. Context & Memory (RAG)

| Scenario | User Input | Expected Behavior | Internal Mechanism |
| :--- | :--- | :--- | :--- |
| **Persistence** | "My name is Max" -> (Restart) -> "Who am I?" | Remembers the user's name across sessions. | `Memory` (SQLite) |
| **Local Docs** | "How do I use stealth mode?" | Retrieves info from `CHEATSHEET.md` (Blackfin docs) and explains `just stealth-mode-on`. | `LocalKnowledge` (sqlite-vec) |
| **Self-Update** | `sensei update` | Checks GitHub Releases, downloads binary, updates `~/.local/bin/sensei`. | `main.py` -> `update()` |

---

## 5. Safety & Filters

*   **Reframing:** Queries like "How to hack" must be internally rephrased by the `RouterAgent` into "Technical audit of..." to avoid LLM refusals, while maintaining the user's intent.
*   **System Safety:** `SystemAgent` must NEVER execute destructive commands (`rm`, `dd`, etc.). Only commands in the Allowlist are permitted.
