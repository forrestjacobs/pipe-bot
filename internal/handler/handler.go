package handler

import (
	"bufio"
	"io"

	"github.com/bwmarrin/discordgo"
)

func read(reader *bufio.Reader) (string, error) {
	// Ignore EOFs if there's no pending text
	str, err := "", io.EOF
	for str == "" && err == io.EOF {
		str, err = reader.ReadString('\n')
	}
	return str, err
}

func Handle(session *discordgo.Session, reader *bufio.Reader) error {
	str, err := read(reader)
	if err != nil {
		return err
	}
	command, err := parse(str)
	if err != nil {
		return err
	}
	return command.run(session)
}
