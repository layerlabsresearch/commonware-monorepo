# commonware-broadcast-demo

Simple demonstration of the `commonware-broadcast` primitive.

## Usage (4 Participants)

_To run this example, you must first install [Rust](https://www.rust-lang.org/tools/install)._

Run at least four participants. Exactly one should broadcast a message using the `--broadcast` flag. Others will print the message once received.

### Participant 0 (Bootstrapper and Broadcaster)

```bash
cargo run --release -- --me 0@3000 --participants 0,1,2,3 --broadcast "hello world"
```

### Participant 1

```bash
cargo run --release -- --bootstrappers 0@127.0.0.1:3000 --me 1@3001 --participants 0,1,2,3
```

### Participant 2

```bash
cargo run --release -- --bootstrappers 0@127.0.0.1:3000 --me 2@3002 --participants 0,1,2,3
```

### Participant 3

```bash
cargo run --release -- --bootstrappers 0@127.0.0.1:3000 --me 3@3003 --participants 0,1,2,3
```
