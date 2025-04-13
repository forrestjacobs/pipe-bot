package handler

import "github.com/bwmarrin/discordgo"

type command interface {
	run(session *discordgo.Session) error
}

type messageCommand struct {
	channelId string
	content   string
}

func (c *messageCommand) run(session *discordgo.Session) error {
	_, err := session.ChannelMessageSend(c.channelId, c.content)
	return err
}

type StatusCommand struct {
	data discordgo.UpdateStatusData
}

func (c *StatusCommand) run(session *discordgo.Session) error {
	return session.UpdateStatusComplex(c.data)
}
