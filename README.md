# pipe-bot: Update Discord from shell scripts

pipe-bot lets you pipe messages and status updates to your Discord from shell scripts.

```sh
# Make a named pipe
mkfifo discord_pipe

# Start the bot in the background
pipe-bot -t $DISCORD_TOKEN < discord_pipe &
botpid=$!

# Send messages and status updates
echo "message $CHANNEL_ID Hello there!" > discord_pipe
echo "playing my guitar" > discord_pipe

# Kill the bot and clean up
kill $botpid
rm discord_pipe
```

## Setup

 1. [Create an application on the Discord Developer Portal](https://discord.com/developers/applications).

 2. Go to _Bot_ > _Add Bot_. Fill it the page, and then jot down the bot's _token_.

 3. Go to _OAuth2_ > _URL Generator_. Check _bot_, then under _Bot Permissions_ check _Send Messages_. Navigate to the URL in the _Generated Link_ at the bottom of the page and follow the prompts in the Discord app.

 4. On the server you want to control, download pipe-bot from [GitHub Releases](https://github.com/forrestjacobs/pipe-bot/releases) and extract it (or build the project from source using `cargo build --release`).

 5. Set up a service using the [example systemd files](./systemd/system/) or [example OpenRC script](./openrc/init.d/pipe-bot). Update the path to the binary, and set the bot token.

 6. You can now pipe status updates and messages into `/run/discord`!

    ```sh
    echo "message $CHANNEL_ID Hello there!" > /run/discord
    echo "playing my guitar" > /run/discord
    ```

## Commands

### Send a message

```sh
# message <CHANNEL ID> <MESSAGE>
echo "message 12345 Hello there!" > discord_pipe
```

* `CHANNEL ID`: ID for the channel receiving the message. You can find this in Discord by right-clicking on the channel, then selecting _Copy Channel ID_.
* `MESSAGE`: message content

### Update status

```sh
# playing|listening_to|watching|competing_in <VALUE>
echo "playing a guitar" > discord_pipe
echo "listening_to the breeze" > discord_pipe
```

* `playing|listening_to|watching|competing_in`: status type
* `VALUE`: status content

### Clear status

```sh
echo "clear_status" > discord_pipe
```

## Config

| CLI flag              | env                     | description                                      |
| --------------------- | ----------------------- | ------------------------------------------------ |
| `-t, --token <TOKEN>` | `PIPEBOT_DISCORD_TOKEN` | Discord bot token, required. See (Setup)[#setup] |
| `-f, --file <FILE>`   | `PIPEBOT_INPUT_FILE`    | Path to input file. Defaults to stdin.           |
