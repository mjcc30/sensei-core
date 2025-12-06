# 📦 Sensei Common

Shared library containing Protocol Types, Traits, and Utilities used by both the Client and the Server.

## 🧱 Key Structures

### API Protocol

*   **`AskRequest`**: The JSON payload sent by the client.
    ```rust
    pub struct AskRequest {
        pub prompt: String,
    }
    ```

*   **`AskResponse`**: The JSON payload returned by the server.
    ```rust
    pub struct AskResponse {
        pub content: String,
    }
    ```

*   **`Health`**: Health check status.
    ```rust
    pub struct Health {
        pub status: String,
    }
    ```

## 🛠️ Integration

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
sensei-common = { path = "../sensei-common" }
```
