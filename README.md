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

 4. On the server you want to control, download pipe-bot from [GitHub Releases](https://github.com/forrestjacobs/pipe-bot/releases) (or build the project from source using `go build`).

 5. Set up a named pipe, and then run the bot with the token from step 2. (Once you have this working, you'll probably want to set it up as a systemd service. You can use the [systemd example files](./systemd/system/), which sets up a named pipe in `/run/discord`)

    ```sh
    mkfifo discord_pipe
    pipe-bot -t $DISCORD_TOKEN < discord_pipe
    ```

 6. You can now pipe status updates and messages into the named pipe!

    ```sh
    echo "message $CHANNEL_ID Hello there!" > discord_pipe
    echo "playing my guitar" > discord_pipe
    ```
