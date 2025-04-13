package main

import (
	"bufio"
	"log"
	"os"

	"github.com/bwmarrin/discordgo"
	"github.com/forrestjacobs/pipe-bot/internal/handler"
	"github.com/forrestjacobs/pipe-bot/internal/token"
)

type exitErrorCode int

const (
	TokenReadError exitErrorCode = 10

	DiscordCreateSessionError  exitErrorCode = 20
	DiscordOpenConnectionError exitErrorCode = 21
)

func dieOnError(err error, code exitErrorCode) {
	if err != nil {
		log.Println(err)
		os.Exit(int(code))
	}
}

func main() {
	token, err := token.GetToken()
	dieOnError(err, TokenReadError)

	discord, err := discordgo.New("Bot " + token)
	dieOnError(err, DiscordCreateSessionError)

	dieOnError(discord.Open(), DiscordOpenConnectionError)
	defer discord.Close()

	reader := bufio.NewReader(os.Stdin)
	for {
		err = handler.Handle(discord, reader)
		if err != nil {
			log.Println(err)
		}
	}
}
