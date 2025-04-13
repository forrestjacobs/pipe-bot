package handler

import (
	"bufio"
	"errors"
	"io"
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

func readLine(reader *bufio.Reader) (string, error) {
	// Ignore EOFs if there's no pending text
	line, err := "", io.EOF
	for line == "" && err == io.EOF {
		line, err = reader.ReadString('\n')
	}
	return line, err
}

func run(session *discordgo.Session, line string) error {
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

func Handle(session *discordgo.Session, reader *bufio.Reader) error {
	line, err := readLine(reader)
	if err != nil {
		return err
	}
	return run(session, line)
}
