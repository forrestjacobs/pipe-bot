package handler

import (
	"errors"
	"regexp"

	"github.com/bwmarrin/discordgo"
)

var messagePattern = regexp.MustCompile(`^(\d+)\s+(.+)$`)

var errArgs = errors.New("could not parse arguments")

func makeStatusHandler(activityType discordgo.ActivityType) func(session *discordgo.Session, args string) error {
	return func(session *discordgo.Session, args string) error {
		if args == "" {
			return errArgs
		}
		return session.UpdateStatusComplex(discordgo.UpdateStatusData{
			Status: "online",
			Activities: []*discordgo.Activity{{
				Name: args,
				Type: activityType,
			}},
		})
	}
}

var commandHandlers = map[string]func(session *discordgo.Session, args string) error{
	"message": func(session *discordgo.Session, args string) error {
		match := messagePattern.FindStringSubmatch(args)
		if match == nil {
			return errArgs
		}
		channelId, body := match[1], match[2]

		_, err := session.ChannelMessageSend(channelId, body)
		return err
	},
	"clear_status": func(session *discordgo.Session, args string) error {
		if args != "" {
			return errArgs
		}
		return session.UpdateStatusComplex(discordgo.UpdateStatusData{
			Status: "online",
		})
	},
	"playing":      makeStatusHandler(discordgo.ActivityTypeGame),
	"listening_to": makeStatusHandler(discordgo.ActivityTypeListening),
	"watching":     makeStatusHandler(discordgo.ActivityTypeWatching),
	"competing_in": makeStatusHandler(discordgo.ActivityTypeCompeting),
}
