# Rust bindings for the Discord Game SDK

### And a mock library for testing (on Linux)

I don't think this currently qualifies as "open source software" as the Discord Game SDK header files are not published under open source licenses. I am not going to redistribute those files, you'll have to put `c/discord_game_sdk.h` in `sys/` yourself, after that, you'll be able to run `cargo build --all && cargo test`.

If you'd like to discuss this, I should be available on the Amethyst Discord (https://discord.gg/amethyst).
