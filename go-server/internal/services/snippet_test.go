package services

import (
	"testing"
)

func TestExtractContextSnippet_BlockCapture(t *testing.T) {
	tests := []struct {
		name     string
		rawMd    string
		target   string
		want     string
		contains string // check substring instead of exact match
	}{
		{
			name: "simple paragraph",
			rawMd: "This is a paragraph about something.\n\nAnother paragraph with [[TestCard]] inside.\n\nThird paragraph.",
			target: "TestCard",
			contains: "Another paragraph with TestCard inside",
		},
		{
			name: "heading with syntax leak (the bug case)",
			rawMd: "```rust\nlet result } ```\n#### 同日记忆\nThis links to [[Target]] in context.\nMore text here.",
			target: "Target",
			contains: "This links to Target in context",
		},
		{
			name: "bold and code markers stripped",
			rawMd: "Some text with **bold** and `code` and [[LinkTarget]] here.",
			target: "LinkTarget",
			contains: "Some text with bold and code and LinkTarget here",
		},
		{
			name: "heading markers stripped",
			rawMd: "## Section Title\n\n### Subsection\n\nHere is the link to [[MyCard]] in a paragraph.\n\nMore stuff.",
			target: "MyCard",
			contains: "Here is the link to MyCard in a paragraph",
		},
		{
			name: "no wikilink found",
			rawMd: "Some text without any links.",
			target: "NonExistent",
			want: "",
		},
		{
			name: "list item context",
			rawMd: "- First item\n- Second item with [[ListItem]] reference\n- Third item",
			target: "ListItem",
			contains: "Second item with ListItem reference",
		},
		{
			name: "long block gets truncated",
			rawMd: "This is a very long paragraph that contains a reference to [[LongTarget]] somewhere in the middle and it continues with lots of text that goes on and on and on and on about various topics including the weather and the state of the world and everything else you can imagine in this extremely verbose paragraph.",
			target: "LongTarget",
			contains: "This is a very long paragraph that contains a reference to LongTarget",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := extractContextSnippet(tt.rawMd, tt.target)
			if tt.want != "" && got != tt.want {
				t.Errorf("got %q, want %q", got, tt.want)
			}
			if tt.contains != "" && !contains(got, tt.contains) {
				t.Errorf("got %q, want to contain %q", got, tt.contains)
			}
			// Verify no markdown syntax leaks
			if contains(got, "```") {
				t.Errorf("snippet contains code fence: %q", got)
			}
			if contains(got, "####") {
				t.Errorf("snippet contains heading markers: %q", got)
			}
			if contains(got, "**") {
				t.Errorf("snippet contains bold markers: %q", got)
			}
		})
	}
}

func contains(s, substr string) bool {
	for i := 0; i+len(substr) <= len(s); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
