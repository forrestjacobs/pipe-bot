package handler

import (
	"errors"
	"regexp"

	"github.com/bwmarrin/discordgo"
)

var inputPattern = regexp.MustCompile(`^(\w+)\s*(?:\s(.+?)\s*)?\n`)

var errParse = errors.New("could not parse input")

type UnrecognizedCommandError struct {
	name string
}

func (e *UnrecognizedCommandError) Error() string {
	return "unrecognized command " + e.name
}

func HandleCommand(session *discordgo.Session, line string) error {
	match := inputPattern.FindStringSubmatch(line)
	if match == nil {
		return errParse
	}

	command, args := match[1], match[2]
	if handler, exists := commandHandlers[command]; exists {
		return handler(session, args)
	} else {
		return &UnrecognizedCommandError{name: command}
	}

}
