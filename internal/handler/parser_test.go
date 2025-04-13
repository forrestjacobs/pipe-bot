package handler

import (
	"reflect"
	"testing"

	"github.com/bwmarrin/discordgo"
)

func TestMessage(t *testing.T) {
	runner, err := parsers["message"]("12345 message content")
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

func TestMessageWithMissingChannel(t *testing.T) {
	runner, err := parsers["message"]("content")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}

func TestMessageWithMissingContent(t *testing.T) {
	runner, err := parsers["message"]("12345")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}

func TestMessageWithNoArgs(t *testing.T) {
	runner, err := parsers["message"]("")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}

func TestClearStatus(t *testing.T) {
	runner, err := parsers["clear_status"]("")
	if !reflect.DeepEqual(runner, &ClearStatusCommand{}) {
		t.Error("Not equal")
	}
	if err != nil {
		t.Error("Unexpected error")
	}
}

func TestClearStatusWithArgs(t *testing.T) {
	runner, err := parsers["clear_status"]("some args")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}

func TestPlayingStatus(t *testing.T) {
	runner, err := parsers["playing"]("a guitar")
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

func TestPlayingEmptyStatus(t *testing.T) {
	runner, err := parsers["playing"]("")
	if runner != nil {
		t.Error("Unexpected runner")
	}
	if err.Error() != "could not parse command" {
		t.Error("Unexpected error")
	}
}
