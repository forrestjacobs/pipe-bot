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

func readLine(reader *bufio.Reader) (string, error) {
	// Ignore EOFs if there's no pending text
	line, err := "", io.EOF
	for line == "" && err == io.EOF {
		line, err = reader.ReadString('\n')
	}
	return line, err
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
		line, err := readLine(reader)

		if err != nil {
			log.Println(err)
			continue
		}

		err = handler.HandleCommand(discord, line)
		if err != nil {
			log.Println(err)
		}
	}
}
