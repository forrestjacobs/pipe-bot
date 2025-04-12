package main

import (
	"bufio"
	"errors"
	"flag"
	"log"
	"os"
	"regexp"
	"strings"

	"github.com/bwmarrin/discordgo"
)

var inputPattern = regexp.MustCompile(`^(status|message:(\d+))\b\s*(.+)$`)

func dieOnError(err error) {
	if err != nil {
		log.Println(err)
		// TODO: Different exit codes for different errors
		os.Exit(1)
	}
}

func getToken() (string, error) {
	var token string
	flag.StringVar(&token, "token", "", "Discord bot token")
	flag.StringVar(&token, "t", "", "Discord bot token (shorthand)")

	flag.Parse()

	if token != "" {
		return token, nil
	}

	if token, present := os.LookupEnv("PIPEBOT_DISCORD_TOKEN"); present {
		return token, nil
	}

	return "", errors.New("missing token")
}

func main() {
	token, err := getToken()
	dieOnError(err)

	discord, err := discordgo.New("Bot " + token)
	dieOnError(err)

	dieOnError(discord.Open())
	defer discord.Close()

	reader := bufio.NewReader(os.Stdin)
	for {
		line, err := reader.ReadString('\n')
		dieOnError(err)
		match := inputPattern.FindStringSubmatch(strings.TrimSuffix(line, "\n"))
		if match == nil {
			log.Println("Could not parse input")
			continue
		}

		command, channelId, body := match[1], match[2], strings.TrimSpace(match[3])
		if command == "status" {
			dieOnError(discord.UpdateGameStatus(0, body))
		} else if strings.HasPrefix(command, "message") {
			_, err = discord.ChannelMessageSend(channelId, body)
			dieOnError(err)
		}
	}
}
