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

func read(reader *bufio.Reader) (string, error) {
	// Ignore EOFs if there's no pending text
	str, err := "", io.EOF
	for str == "" && err == io.EOF {
		str, err = reader.ReadString('\n')
	}
	return str, err
}

func parse(str string) (Command, error) {
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
