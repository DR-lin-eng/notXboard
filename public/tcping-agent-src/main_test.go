package main

import (
	"context"
	"net"
	"strings"
	"testing"
)

func TestProbeAddressPolicy(t *testing.T) {
	for _, raw := range []string{
		"127.0.0.1",
		"10.0.0.1",
		"100.64.0.1",
		"169.254.169.254",
		"192.0.2.1",
		"198.18.0.1",
		"224.0.0.1",
		"::1",
		"fc00::1",
		"fe80::1",
		"2001:db8::1",
	} {
		if isPublicProbeIP(net.ParseIP(raw)) {
			t.Fatalf("expected %s to be blocked", raw)
		}
	}

	for _, raw := range []string{"1.1.1.1", "8.8.8.8", "2606:4700:4700::1111"} {
		if !isPublicProbeIP(net.ParseIP(raw)) {
			t.Fatalf("expected %s to be public", raw)
		}
	}
}

func TestLiteralProbeUsesResolvedIPAddress(t *testing.T) {
	address, err := resolvePublicProbeAddress(context.Background(), "1.1.1.1", 443)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.HasPrefix(address, "1.1.1.1:") {
		t.Fatalf("unexpected resolved address: %s", address)
	}
	if _, err := resolvePublicProbeAddress(context.Background(), "127.0.0.1", 80); err == nil {
		t.Fatal("loopback target should be rejected")
	}
}
