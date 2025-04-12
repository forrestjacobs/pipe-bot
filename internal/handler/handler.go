package handler

import (
	"errors"
	"regexp"
	"strings"

	"github.com/bwmarrin/discordgo"
)

var inputPattern = regexp.MustCompile(`^(\w+)(.*)$`)

func HandleCommand(session *discordgo.Session, line string) error {
	match := inputPattern.FindStringSubmatch(strings.TrimSuffix(line, "\n"))
	if match == nil {
		return errors.New("could not parse input")
	}

	command, args := match[1], match[2]
	if handler, exists := commandHandlers[command]; exists {
		return handler(session, command, args)
	} else {
		return errors.New("unrecognized command " + command)
	}

}
