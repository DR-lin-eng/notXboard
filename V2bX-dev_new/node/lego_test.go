package node

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/InazumaV/V2bX/conf"
)

func newLiveLego(t *testing.T) *Lego {
	t.Helper()
	if os.Getenv("V2BX_LIVE_ACME_TEST") != "1" {
		t.Skip("set V2BX_LIVE_ACME_TEST=1 to run the live ACME integration test")
	}
	email := os.Getenv("V2BX_ACME_EMAIL")
	domain := os.Getenv("V2BX_ACME_DOMAIN")
	token := os.Getenv("CF_DNS_API_TOKEN")
	if email == "" || domain == "" || token == "" {
		t.Fatal("V2BX_ACME_EMAIL, V2BX_ACME_DOMAIN, and CF_DNS_API_TOKEN are required")
	}
	tempDir := t.TempDir()
	l, err := NewLego(&conf.CertConfig{
		CertMode:   "dns",
		Email:      email,
		CertDomain: domain,
		Provider:   "cloudflare",
		DNSEnv: map[string]string{
			"CF_DNS_API_TOKEN": token,
		},
		CertFile: filepath.Join(tempDir, "cert.pem"),
		KeyFile:  filepath.Join(tempDir, "cert.key"),
	})
	if err != nil {
		t.Fatal(err)
	}
	return l
}

func TestLego_CreateCertByDns(t *testing.T) {
	l := newLiveLego(t)
	err := l.CreateCert()
	if err != nil {
		t.Error(err)
	}
}

func TestLego_RenewCert(t *testing.T) {
	l := newLiveLego(t)
	if err := l.RenewCert(); err != nil {
		t.Error(err)
	}
}
