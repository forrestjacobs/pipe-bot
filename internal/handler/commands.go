package handler

import (
	"errors"
	"regexp"
	"strings"

	"github.com/bwmarrin/discordgo"
)

var messageArgsPattern = regexp.MustCompile(`^:(\d+)\s+(.+)$`)

func makeStatusHandler(activityType discordgo.ActivityType) func(session *discordgo.Session, command string, args string) error {
	return func(session *discordgo.Session, command string, args string) error {
		name := strings.TrimSpace(args)
		if name == "" {
			return errors.New("missing argument")
		}
		return session.UpdateStatusComplex(discordgo.UpdateStatusData{
			Status: "online",
			Activities: []*discordgo.Activity{{
				Name: strings.TrimSpace(args),
				Type: activityType,
			}},
		})
	}
}

var commandHandlers = map[string]func(session *discordgo.Session, command string, args string) error{
	"message": func(session *discordgo.Session, command string, args string) error {
		argMatch := messageArgsPattern.FindStringSubmatch(args)
		if argMatch == nil {
			return errors.New("could not parse message")
		}
		channelId, body := argMatch[1], argMatch[2]

		_, err := session.ChannelMessageSend(channelId, body)
		return err
	},
	"clear_status": func(session *discordgo.Session, command string, args string) error {
		if args != "" {
			return errors.New("unexpected argument")
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
