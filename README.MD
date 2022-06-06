# How Long To Beat Telegram Bot

Telegram bot (hosted by AWS Lambda) that respond with [How Long To Beat](https://howlongtobeat.com/) information by scrapping site itself.

[@htlb_assistant_bot](https://t.me/hltb_assistant_bot)

Here how typical response would look like: 

![Example](./readme/example.png)

## Build

### Using Docker

1. Prerequisite: Docker
2. Run docker compose
```
   docker compose up --build build
```

### Locally
1. Prerequisite: Linux
2. Install Rust https://www.rust-lang.org/tools/install
3. Install musl toolchain
   ```
   rustup target add x86_64-unknown-linux-musl
   ```
4. Install musl-tools
   ```
   sudo apt install musl-tools 
   ``` 
5. Install zip
   ```
   apt-get install zip
   ```
6. Install build essential
   ```
   apt-get install -y build-essential
   ```
7. Build
   ```
   sh build.sh
   ```

## Run

Define env variables:
- `API_KEY` - bot token
- `RUN_MODE` - "Polling" or "WebHook" (case-insensitive). Defaults to "WebHook"
- `LOG_LEVEL` - https://docs.rs/log/latest/log/enum.LevelFilter.html#variants
- `ENTRIES_LIMIT` - how many entries to show (max). Defaults to 5

Bot can function both in long-polling and webhooks mechanism.
To register it to use in  WebHook mode, you can call lambda with following JSON:
```json
{
  "lambda_rq_type": "register_webhook",
  "url": "<webhook url>"
}
```
or following to unregister:
```json
{
  "lambda_rq_type": "remove_webhook"
}
```

Any other JSON would be interpreted as Telegram WebHook message.


## License

MIT