package main

import (
	"bufio"
	"io"
	"log"
	"os"
	"regexp"
	"strings"

	"github.com/bwmarrin/discordgo"
	"github.com/forrestjacobs/pipe-bot/internal/token"
)

var commandToActivity = map[string]discordgo.ActivityType{
	"playing":      discordgo.ActivityTypeGame,
	"listening_to": discordgo.ActivityTypeListening,
	"watching":     discordgo.ActivityTypeWatching,
	"competing_in": discordgo.ActivityTypeCompeting,
}

var inputPattern = regexp.MustCompile(`^(\w+)(.*)$`)
var messageArgsPattern = regexp.MustCompile(`^:(\d+)\s+(.+)$`)

func dieOnError(err error) {
	if err != nil {
		log.Println(err)
		// TODO: Different exit codes for different errors
		os.Exit(1)
	}
}

func main() {
	token, err := token.GetToken()
	dieOnError(err)

	discord, err := discordgo.New("Bot " + token)
	dieOnError(err)

	dieOnError(discord.Open())
	defer discord.Close()

	reader := bufio.NewReader(os.Stdin)
	for {
		line, err := reader.ReadString('\n')
		// Ignore EOFs so the bot doesn't exit after the first piped command
		if err == io.EOF {
			continue
		}
		dieOnError(err)

		match := inputPattern.FindStringSubmatch(strings.TrimSuffix(line, "\n"))
		if match == nil {
			log.Println("Could not parse input")
			continue
		}

		command, args := match[1], match[2]
		if command == "message" {
			argMatch := messageArgsPattern.FindStringSubmatch(args)
			if argMatch == nil {
				log.Println("Could not parse message")
				continue
			}
			channelId, body := argMatch[1], argMatch[2]

			_, err = discord.ChannelMessageSend(channelId, body)
			dieOnError(err)
		} else if command == "clear_status" {
			if args != "" {
				log.Println("Could not parse clear_status")
				continue
			}
			dieOnError(discord.UpdateStatusComplex(discordgo.UpdateStatusData{
				Status: "online",
			}))
		} else if activityType, present := commandToActivity[command]; present {
			name := strings.TrimSpace(args)
			if name == "" {
				log.Println("Could not parse status")
				continue
			}
			dieOnError(discord.UpdateStatusComplex(discordgo.UpdateStatusData{
				Status: "online",
				Activities: []*discordgo.Activity{{
					Name: strings.TrimSpace(args),
					Type: activityType,
				}},
			}))
		}
	}
}
