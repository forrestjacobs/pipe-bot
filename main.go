package main

import (
	"bufio"
	"io"
	"log"
	"os"

	"github.com/bwmarrin/discordgo"
	"github.com/forrestjacobs/pipe-bot/internal/handler"
	"github.com/forrestjacobs/pipe-bot/internal/token"
)

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
		} else if err != nil {
			log.Println(err)
			continue
		}

		err = handler.HandleCommand(discord, line)
		if err != nil {
			log.Println(err)
		}
	}
}
