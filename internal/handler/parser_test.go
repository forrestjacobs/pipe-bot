package handler

import (
	"reflect"
	"testing"

	"github.com/bwmarrin/discordgo"
)

func TestParseBadString(t *testing.T) {
	runner, err := parse("Missing LF")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse input" {
		t.Error("Unexpected error")
	}
}

func TestUnrecognizedCommand(t *testing.T) {
	runner, err := parse("nonce\n")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "unrecognized command nonce" {
		t.Error("Unexpected error")
	}
}

func TestParseMessage(t *testing.T) {
	runner, err := parse("message 12345 message content\n")
	if !reflect.DeepEqual(runner, &MessageCommand{
		ChannelId: "12345",
		Content:   "message content",
	}) {
		t.Error("Not equal")
	}
	if err != nil {
		t.Error("Unexpected error")
	}
}

func TestParseMessageWithMissingChannel(t *testing.T) {
	runner, err := parse("message content\n")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}

func TestParseMessageWithMissingContent(t *testing.T) {
	runner, err := parse("message 12345\n")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}

func TestParseMessageWithNoArgs(t *testing.T) {
	runner, err := parse("message\n")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}

func TestParseClearStatus(t *testing.T) {
	runner, err := parse("clear_status\n")
	if !reflect.DeepEqual(runner, &ClearStatusCommand{}) {
		t.Error("Not equal")
	}
	if err != nil {
		t.Error("Unexpected error")
	}
}

func TestParseClearStatusWithArgs(t *testing.T) {
	runner, err := parse("clear_status some args\n")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}

func TestParsePlayingStatus(t *testing.T) {
	runner, err := parse("playing a guitar\n")
	if !reflect.DeepEqual(runner, &StatusCommand{
		Name: "a guitar",
		Type: discordgo.ActivityTypeGame,
	}) {
		t.Error("Not equal")
	}
	if err != nil {
		t.Error("Unexpected error")
	}
}

func TestParsePlayingEmptyStatus(t *testing.T) {
	runner, err := parse("playing\n")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}
