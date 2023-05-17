# Golem API

This document describes the Golem API. Note that the API is still in development and there may be many major changes in the future.

## Connecting to the chat

- Golem uses [websockets].
- The URL to connect is `/api/ws`.

### Messages

- All messages are sent as JSON objects.
  - They are [externally tagged].

[websockets]: https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API
[externally tagged]: https://serde.rs/enum-representations.html#externally-tagged

#### Client Messages

| Message                   | Description                                     |
| ------------------------- | ----------------------------------------------- |
| AuthenticateToken(u64)    | Authenticate with a token.                      |
| Authenticate(PartialUser) | Authenticate with the required parts of a user. |
| Message(SendMessage)      | Send a message.                                 |
| LoadAllMessages           | Load all messages.                              |

#### Server Message

| Message                        | Description                                  |
| ------------------------------ | -------------------------------------------- |
| Authenticate { success: bool } | Whether or not the authentication succeeded. |
| NewMessage(Message)            | A new message was sent.                      |
| Error                          | There was an internal server error.          |
| Messages(Vec&lt;Message&gt;)   | The messages that were previously requested. |
| Duplicate(String)              | A duplicate message was sent. String=dup_id  |
| Join(User)                     | A user has just joined.                      |
