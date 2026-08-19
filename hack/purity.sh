#!/bin/sh
set -eu

fail=0

BANNED_CRATES='^windows|^tokio|^reqwest|^arboard|^eframe|^egui|^hidapi|^rusb'

if grep -nE "$BANNED_CRATES" Cargo.toml; then
    echo "opends-core does I/O only if a crate lets it. None of the above belongs here." >&2
    fail=1
fi

BANNED_CALLS='std::fs|std::net|std::process|std::io::stdin|std::io::stdout|File::open|File::create|TcpStream|UdpSocket|SystemTime::now|Instant::now'

if grep -rnE "$BANNED_CALLS" src/; then
    echo "" >&2
    echo "opends-core is pure. No filesystem, no socket, no process, no clock." >&2
    echo "A caller passes time in. Move this to opends-app." >&2
    fail=1
fi

if [ "$fail" -eq 0 ]; then
    echo "opends-core is free of I/O"
fi

exit "$fail"
