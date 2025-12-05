# Sensei Core (v3) Behavior Specification

This document defines the expected behaviors of the Sensei AI Mentor. It serves as the "Golden Master" for testing the Rust (v3) rewrite.

---

## 1. Core Interaction & Personality

| Scenario | User Input | Expected Behavior | Internal Route |
| :--- | :--- | :--- | :--- |
| **Greeting** | "Hello", "Hi Sensei" | Responds politely, briefly, and professionally. Hacker-chic tone. | `CASUAL` -> `SpecializedAgent` |
| **Off-topic** | "Tell me a joke" | Answers concisely or politely redirects to cyber topics. | `CASUAL` -> `SpecializedAgent` |
| **Education** | "What is a buffer overflow?" | Explains concepts simply using analogies. | `NOVICE` -> `SpecializedAgent` |
| **Deep Dive** | "Analyze CVE-2024-3094" | Provides deep technical analysis, timelines, and code snippets. | `RED` / `BLUE` -> `SpecializedAgent` |

## 2. Domain Expertise (Routing)

| Scenario | User Input | Expected Behavior | Internal Route |
| :--- | :--- | :--- | :--- |
| **Offensive** | "How to create a reverse shell?" | Provides technical, operational details. **God Mode** if `--raw` is used. | `RED` -> `SpecializedAgent` (Smart Model) |
| **Defensive** | "Analyze firewall logs" | Focuses on detection, mitigation, and forensics. | `BLUE` -> `SpecializedAgent` (Smart Model) |
| **Cloud** | "AWS S3 enumeration" | Discusses cloud-specific vectors (S3, IAM). | `CLOUD` -> `SpecializedAgent` (Smart Model) |
| **Crypto** | "Crack this hash" | Analyzes ciphers, entropy, provides scripts. | `CRYPTO` -> `SpecializedAgent` (Smart Model) |
| **OSINT** | "Find emails for domain.com" | Discusses reconnaissance techniques. | `OSINT` -> `SpecializedAgent` (Smart Model) |

## 3. Tool Execution (Action & System)

| Scenario | User Input | Expected Behavior | Internal Route | Triggered Tool |
| :--- | :--- | :--- | :--- | :--- |
| **Scanning** | "Scan localhost" | Executes Nmap and returns output. | `ACTION` -> `ToolExecutorAgent` | `NmapTool` |
| **System Diag** | "Check RAM usage" | Runs `free -h` and reports usage. | `SYSTEM` -> `ToolExecutorAgent` | `SystemTool` |
| **Disk** | "Disk space?" | Runs `df -h` and analyzes output. | `SYSTEM` -> `ToolExecutorAgent` | `SystemTool` |

## 4. Context & Memory (RAG)

| Scenario | User Input | Expected Behavior | Internal Mechanism |
| :--- | :--- | :--- | :--- |
| **Persistence** | "My name is Max" -> ... -> "Who am I?" | Remembers user name across sessions. | `SQLite` (Table `messages`) |
| **Knowledge** | "What is the launch code?" | Retrieves `secret.txt` content (if ingested) and answers. | `RAG` (`sqlite-vec`, `vec_items`) |

---

## 5. Safety & Filters

*   **Query Optimization:** Queries like "How to hack" are rephrased by `RouterAgent` into "Technical audit of..." to bypass soft filters.
*   **God Mode:** `Red` agent can bypass Safety Filters (`BLOCK_NONE`) if triggered with `--raw`.
*   **System Safety:** `ToolExecutorAgent` uses a strict **Allowlist** for system commands (`uptime`, `free`, `df`). No destructive commands (`rm`).
