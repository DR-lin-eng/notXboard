package node

import (
	"crypto/tls"
	"os"
	"path/filepath"
	"testing"
)

func Test_generateSelfSslCertificate(t *testing.T) {
	tempDir := t.TempDir()
	certPath := filepath.Join(tempDir, "cert.pem")
	keyPath := filepath.Join(tempDir, "cert.key")
	if err := generateSelfSslCertificate("domain.com", certPath, keyPath); err != nil {
		t.Fatal(err)
	}
	if _, err := tls.LoadX509KeyPair(certPath, keyPath); err != nil {
		t.Fatalf("generated certificate pair is invalid: %v", err)
	}
	keyInfo, err := os.Stat(keyPath)
	if err != nil {
		t.Fatal(err)
	}
	if got := keyInfo.Mode().Perm(); got != 0600 {
		t.Fatalf("private key mode = %o, want 600", got)
	}
}
