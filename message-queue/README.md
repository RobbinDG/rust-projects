## Feature To-Do List

- [ ] Disconnect senders and receivers when queue is deleted.
- [ ] Persist queues and queue configuration.
- [ ] Support other stream types over `StreamIO`.
- [ ] Add customisation.
    - [x] TTL.
    - [ ] Max capacity.
- [x] Dead lettering.
- [x] Send messages in queue to DLX on deletion.
- [x] Reconnect on disconnect admin panel.
- [x] Filtering on topics on the receiving end, i.e. creating topics and subtopics.
- [ ] Messages with payloads other than strings (JSON, YAML, XML, etc.).
- [x] Logs are (also) messages onto a topic.

## Redacted ideas

### Internal microservices

Some internal "services" of the server, the queue store and subscription manager, are shared
between a relatively large amount of other components, and hence locked behind an `Arc<Mutex<_>>`.
This creates ugly and hard to read code. A possible solution is to separate them into their own (
`tokio`) threads/processes and reach them using asynchronous calls. This mimics a microservice
architecture. Although this is a good idea in theory, it creates a lot of overhead through system
calls. On the other hand, expanding the system to a larger one consisting out of several
independently managed services would be more intuitive after such a change.