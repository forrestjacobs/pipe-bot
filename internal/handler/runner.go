package handler

import "github.com/bwmarrin/discordgo"

var clearStatusData = discordgo.UpdateStatusData{Status: "online"}

type Command interface {
	run(session *discordgo.Session) error
}

type MessageCommand struct {
	ChannelId string
	Content   string
}

func (c *MessageCommand) run(session *discordgo.Session) error {
	_, err := session.ChannelMessageSend(c.ChannelId, c.Content)
	return err
}

type StatusCommand struct {
	Type discordgo.ActivityType
	Name string
}

func (c *StatusCommand) run(session *discordgo.Session) error {
	return session.UpdateStatusComplex(discordgo.UpdateStatusData{
		Status: "online",
		Activities: []*discordgo.Activity{{
			Type: c.Type,
			Name: c.Name,
		}},
	})
}

type ClearStatusCommand struct{}

func (c *ClearStatusCommand) run(session *discordgo.Session) error {
	return session.UpdateStatusComplex(clearStatusData)
}
