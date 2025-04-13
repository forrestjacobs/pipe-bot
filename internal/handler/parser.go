package handler

import (
	"errors"
	"regexp"

	"github.com/bwmarrin/discordgo"
)

var messagePattern = regexp.MustCompile(`^(\d+)\s+(.+)$`)

var errArgs = errors.New("could not parse command")

func makeStatusParser(activityType discordgo.ActivityType) func(args string) (Command, error) {
	return func(args string) (Command, error) {
		if args == "" {
			return nil, errArgs
		}
		return &StatusCommand{
			Name: args,
			Type: activityType,
		}, nil
	}
}

var parsers = map[string]func(args string) (Command, error){
	"message": func(args string) (Command, error) {
		match := messagePattern.FindStringSubmatch(args)
		if match == nil {
			return nil, errArgs
		}
		return &MessageCommand{
			ChannelId: match[1],
			Content:   match[2],
		}, nil
	},
	"clear_status": func(args string) (Command, error) {
		if args != "" {
			return nil, errArgs
		}
		return &ClearStatusCommand{}, nil
	},
	"playing":      makeStatusParser(discordgo.ActivityTypeGame),
	"listening_to": makeStatusParser(discordgo.ActivityTypeListening),
	"watching":     makeStatusParser(discordgo.ActivityTypeWatching),
	"competing_in": makeStatusParser(discordgo.ActivityTypeCompeting),
}
