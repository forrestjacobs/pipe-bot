package handler

import (
	"errors"
	"regexp"

	"github.com/bwmarrin/discordgo"
)

var inputPattern = regexp.MustCompile(`^(\w+)\s*(?:\s(.+?)\s*)?\n`)

func HandleCommand(session *discordgo.Session, line string) error {
	match := inputPattern.FindStringSubmatch(line)
	if match == nil {
		return errors.New("could not parse input")
	}

	command, args := match[1], match[2]
	if handler, exists := commandHandlers[command]; exists {
		return handler(session, args)
	} else {
		return errors.New("unrecognized command " + command)
	}

}
