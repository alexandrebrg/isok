# Isok Web Offloader

This binary will expose an API on port 8080, and will allow you to see live results 
from isok probe plane. It should be used for testing purposes only, it isn't
the intended materialization of the results.

It will spawn every component it needs: a broker, an agent, and a kafka server.

**This example will spawn a kafka container, you need to have docker running.**

```bash
cargo run
open http://localhost:8080
```