package handler

import (
	"errors"
	"regexp"

	"github.com/bwmarrin/discordgo"
)

var inputPattern = regexp.MustCompile(`^(\w+)\s*(?:\s(.+?)\s*)?\n`)
var messagePattern = regexp.MustCompile(`^(\d+)\s+(.+)$`)

var errParse = errors.New("could not parse input")
var errArgs = errors.New("could not parse command")

var clearStatusCommand = &StatusCommand{discordgo.UpdateStatusData{Status: "online"}}

type UnrecognizedCommandError struct {
	name string
}

func (e *UnrecognizedCommandError) Error() string {
	return "unrecognized command " + e.name
}

func makeStatusParser(activityType discordgo.ActivityType) func(args string) (command, error) {
	return func(args string) (command, error) {
		if args == "" {
			return nil, errArgs
		}
		return &StatusCommand{
			data: discordgo.UpdateStatusData{
				Status: "online",
				Activities: []*discordgo.Activity{{
					Type: activityType,
					Name: args,
				}},
			},
		}, nil
	}
}

var parsers = map[string]func(args string) (command, error){
	"message": func(args string) (command, error) {
		match := messagePattern.FindStringSubmatch(args)
		if match == nil {
			return nil, errArgs
		}
		return &messageCommand{
			channelId: match[1],
			content:   match[2],
		}, nil
	},
	"clear_status": func(args string) (command, error) {
		if args != "" {
			return nil, errArgs
		}
		return clearStatusCommand, nil
	},
	"playing":      makeStatusParser(discordgo.ActivityTypeGame),
	"listening_to": makeStatusParser(discordgo.ActivityTypeListening),
	"watching":     makeStatusParser(discordgo.ActivityTypeWatching),
	"competing_in": makeStatusParser(discordgo.ActivityTypeCompeting),
}

func parse(str string) (command, error) {
	match := inputPattern.FindStringSubmatch(str)
	if match == nil {
		return nil, errParse
	}

	name, args := match[1], match[2]
	if parser, exists := parsers[name]; exists {
		return parser(args)
	} else {
		return nil, &UnrecognizedCommandError{name}
	}
}
