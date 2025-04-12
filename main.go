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

var inputPattern = regexp.MustCompile(`^(status|message)\b\s*(.*)\n$`)
var messagePattern = regexp.MustCompile(`^(\d+)\s*(.*)$`)

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
		text, err := reader.ReadString('\n')
		dieOnError(err)
		match := inputPattern.FindStringSubmatch(text)
		if match == nil {
			log.Println("Could not parse input")
			continue
		}

		command, body := match[1], match[2]
		switch command {
		case "status":
			dieOnError(discord.UpdateGameStatus(0, strings.TrimSpace(body)))
		case "message":
			messageMatch := messagePattern.FindStringSubmatch(body)
			if messageMatch == nil {
				log.Println("Could not parse message")
				continue
			}

			channelId, message := messageMatch[1], messageMatch[2]
			_, err = discord.ChannelMessageSend(channelId, message)
			dieOnError(err)
		}
	}
}
