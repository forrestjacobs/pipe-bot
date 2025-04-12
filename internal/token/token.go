package token

import (
	"errors"
	"flag"
	"os"
)

func GetToken() (string, error) {
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
