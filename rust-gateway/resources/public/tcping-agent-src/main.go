package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"log"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"path"
	"strings"
	"sync"
	"syscall"
	"time"
)

var agentVersion = "dev"

type configResponse struct {
	Success bool        `json:"success"`
	Data    agentConfig `json:"data"`
	Error   string      `json:"error"`
}

type boolResponse struct {
	Success bool   `json:"success"`
	Error   string `json:"error"`
}

type agentConfig struct {
	Agent   agentMeta `json:"agent"`
	Targets []target  `json:"targets"`
}

type agentMeta struct {
	ID                  int64 `json:"id"`
	PullIntervalSeconds int   `json:"pull_interval_seconds"`
}

type target struct {
	NodeID              int64  `json:"node_id"`
	Name                string `json:"name"`
	Host                string `json:"host"`
	Port                int    `json:"port"`
	IntervalSeconds     int    `json:"interval_seconds"`
	TimeoutMS           int    `json:"timeout_ms"`
	AlertAfterSeconds   int    `json:"alert_after_seconds"`
	RecoverAfterSeconds int    `json:"recover_after_seconds"`
}

type sample struct {
	NodeID      int64  `json:"node_id"`
	IsReachable bool   `json:"is_reachable"`
	LatencyMS   int64  `json:"latency_ms,omitempty"`
	IsTimeout   bool   `json:"is_timeout"`
	Error       string `json:"error_message,omitempty"`
	SampledAt   int64  `json:"sampled_at"`
}

type sampleEnvelope struct {
	Samples []sample `json:"samples"`
}

type probeResult struct {
	TargetID int64
	Sample   sample
}

type apiClient struct {
	baseURL string
	token   string
	client  *http.Client
}

func newAPIClient(baseURL, token string) (*apiClient, error) {
	baseURL = strings.TrimSpace(baseURL)
	token = strings.TrimSpace(token)
	if baseURL == "" {
		return nil, errors.New("empty panel url")
	}
	if token == "" {
		return nil, errors.New("empty agent token")
	}

	parsed, err := url.Parse(baseURL)
	if err != nil {
		return nil, fmt.Errorf("invalid panel url: %w", err)
	}
	parsed.Path = strings.TrimRight(parsed.Path, "/")

	return &apiClient{
		baseURL: parsed.String(),
		token:   token,
		client: &http.Client{
			Timeout: 20 * time.Second,
		},
	}, nil
}

func (c *apiClient) endpoint(relPath string) string {
	base, _ := url.Parse(c.baseURL)
	base.Path = path.Join(base.Path, relPath)
	return base.String()
}

func (c *apiClient) newRequest(ctx context.Context, method, relPath string, body []byte) (*http.Request, error) {
	req, err := http.NewRequestWithContext(ctx, method, c.endpoint(relPath), bytes.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Authorization", "Bearer "+c.token)
	req.Header.Set("Content-Type", "application/json")
	return req, nil
}

func (c *apiClient) fetchConfig(ctx context.Context) (agentConfig, error) {
	req, err := c.newRequest(ctx, http.MethodGet, "/api/v1/tcping/agent/config", nil)
	if err != nil {
		return agentConfig{}, err
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return agentConfig{}, err
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		return agentConfig{}, fmt.Errorf("config request failed: http %d", resp.StatusCode)
	}

	var parsed configResponse
	if err := json.NewDecoder(resp.Body).Decode(&parsed); err != nil {
		return agentConfig{}, err
	}
	if !parsed.Success {
		if parsed.Error != "" {
			return agentConfig{}, errors.New(parsed.Error)
		}
		return agentConfig{}, errors.New("config request returned unsuccessful response")
	}
	return parsed.Data, nil
}

func (c *apiClient) sendHeartbeat(ctx context.Context) error {
	req, err := c.newRequest(ctx, http.MethodPost, "/api/v1/tcping/agent/heartbeat", []byte(`{}`))
	if err != nil {
		return err
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		return fmt.Errorf("heartbeat failed: http %d", resp.StatusCode)
	}

	var parsed boolResponse
	if err := json.NewDecoder(resp.Body).Decode(&parsed); err != nil {
		return err
	}
	if !parsed.Success {
		if parsed.Error != "" {
			return errors.New(parsed.Error)
		}
		return errors.New("heartbeat returned unsuccessful response")
	}
	return nil
}

func (c *apiClient) sendSamples(ctx context.Context, items []sample) error {
	if len(items) == 0 {
		return nil
	}
	payload, err := json.Marshal(sampleEnvelope{Samples: items})
	if err != nil {
		return err
	}

	req, err := c.newRequest(ctx, http.MethodPost, "/api/v1/tcping/agent/samples", payload)
	if err != nil {
		return err
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		return fmt.Errorf("sample upload failed: http %d", resp.StatusCode)
	}

	var parsed boolResponse
	if err := json.NewDecoder(resp.Body).Decode(&parsed); err != nil {
		return err
	}
	if !parsed.Success {
		if parsed.Error != "" {
			return errors.New(parsed.Error)
		}
		return errors.New("sample upload returned unsuccessful response")
	}
	return nil
}

type runner struct {
	logger       *log.Logger
	api          *apiClient
	flushSize    int
	queueLimit   int
	heartbeatGap time.Duration
	configGap    time.Duration

	mu        sync.Mutex
	queue     []sample
	targets    map[int64]target
	nextProbe  map[int64]time.Time
	isRunning  map[int64]bool
	pullTicker *time.Ticker
}

func newRunner(logger *log.Logger, api *apiClient) *runner {
	return &runner{
		logger:       logger,
		api:          api,
		flushSize:    20,
		queueLimit:   2000,
		heartbeatGap: 30 * time.Second,
		configGap:    60 * time.Second,
		targets:      make(map[int64]target),
		nextProbe:    make(map[int64]time.Time),
		isRunning:    make(map[int64]bool),
	}
}

func (r *runner) run(ctx context.Context) error {
	if err := r.refreshConfig(ctx); err != nil {
		r.logger.Printf("initial config load failed: %v", err)
	}

	flushTicker := time.NewTicker(5 * time.Second)
	defer flushTicker.Stop()

	heartbeatTicker := time.NewTicker(r.heartbeatGap)
	defer heartbeatTicker.Stop()

	probeTicker := time.NewTicker(1 * time.Second)
	defer probeTicker.Stop()

	configTicker := time.NewTicker(60 * time.Second)
	defer configTicker.Stop()

	results := make(chan probeResult, 256)

	for {
		select {
		case <-ctx.Done():
			r.logger.Println("shutting down, flushing pending samples")
			r.flushQueue(context.Background())
			return nil
		case <-configTicker.C:
			if err := r.refreshConfig(ctx); err != nil {
				r.logger.Printf("config refresh failed: %v", err)
			} else {
				configTicker.Reset(r.pullInterval())
			}
		case <-heartbeatTicker.C:
			hCtx, cancel := context.WithTimeout(ctx, 15*time.Second)
			err := r.api.sendHeartbeat(hCtx)
			cancel()
			if err != nil {
				r.logger.Printf("heartbeat failed: %v", err)
			}
		case <-flushTicker.C:
			r.flushQueue(ctx)
		case <-probeTicker.C:
			r.scheduleDueTargets(ctx, results)
		case result := <-results:
			r.finishProbe(result)
		}
	}
}

func (r *runner) pullInterval() time.Duration {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.configGap
}

func (r *runner) refreshConfig(ctx context.Context) error {
	cfgCtx, cancel := context.WithTimeout(ctx, 20*time.Second)
	defer cancel()

	cfg, err := r.api.fetchConfig(cfgCtx)
	if err != nil {
		return err
	}

	now := time.Now()
	r.mu.Lock()
	defer r.mu.Unlock()

	nextTargets := make(map[int64]target, len(cfg.Targets))
	nextProbe := make(map[int64]time.Time, len(cfg.Targets))
	nextRunning := make(map[int64]bool, len(cfg.Targets))

	for _, targetItem := range cfg.Targets {
		targetItem.IntervalSeconds = normalizeSeconds(targetItem.IntervalSeconds, 60, 15, 3600)
		targetItem.TimeoutMS = normalizeInt(targetItem.TimeoutMS, 3000, 500, 60000)
		if targetItem.NodeID <= 0 || strings.TrimSpace(targetItem.Host) == "" || targetItem.Port <= 0 {
			continue
		}

		nextTargets[targetItem.NodeID] = targetItem
		nextRunning[targetItem.NodeID] = false

		if dueAt, ok := r.nextProbe[targetItem.NodeID]; ok {
			nextProbe[targetItem.NodeID] = dueAt
		} else {
			nextProbe[targetItem.NodeID] = now
		}
	}

	r.targets = nextTargets
	r.nextProbe = nextProbe
	r.isRunning = nextRunning
	r.configGap = time.Duration(normalizeSeconds(cfg.Agent.PullIntervalSeconds, 60, 15, 3600)) * time.Second
	r.logger.Printf("config refreshed: %d targets", len(r.targets))

	return nil
}

func (r *runner) scheduleDueTargets(ctx context.Context, out chan<- probeResult) {
	r.mu.Lock()
	defer r.mu.Unlock()

	now := time.Now()
	for nodeID, targetItem := range r.targets {
		if r.isRunning[nodeID] {
			continue
		}
		if dueAt, ok := r.nextProbe[nodeID]; ok && dueAt.After(now) {
			continue
		}

		r.isRunning[nodeID] = true
		go func(item target) {
			out <- probeResult{
				TargetID: item.NodeID,
				Sample:   probeTarget(ctx, item),
			}
		}(targetItem)
	}
}

func (r *runner) finishProbe(result probeResult) {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.isRunning[result.TargetID] = false
	if targetItem, ok := r.targets[result.TargetID]; ok {
		r.nextProbe[result.TargetID] = time.Now().Add(time.Duration(targetItem.IntervalSeconds) * time.Second)
	}

	r.queue = append(r.queue, result.Sample)
	if len(r.queue) > r.queueLimit {
		r.queue = append([]sample(nil), r.queue[len(r.queue)-r.queueLimit:]...)
	}
}

func (r *runner) flushQueue(ctx context.Context) {
	items := r.dequeue()
	if len(items) == 0 {
		return
	}

	flushCtx, cancel := context.WithTimeout(ctx, 20*time.Second)
	defer cancel()

	if err := r.api.sendSamples(flushCtx, items); err != nil {
		r.logger.Printf("sample upload failed, keeping %d samples in memory: %v", len(items), err)
		r.requeue(items)
		return
	}

	r.logger.Printf("uploaded %d samples", len(items))
}

func (r *runner) dequeue() []sample {
	r.mu.Lock()
	defer r.mu.Unlock()

	if len(r.queue) == 0 {
		return nil
	}

	size := len(r.queue)
	if size > r.flushSize {
		size = r.flushSize
	}

	items := append([]sample(nil), r.queue[:size]...)
	r.queue = append([]sample(nil), r.queue[size:]...)
	return items
}

func (r *runner) requeue(items []sample) {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.queue = append(items, r.queue...)
	if len(r.queue) > r.queueLimit {
		r.queue = r.queue[:r.queueLimit]
	}
}

func probeTarget(ctx context.Context, item target) sample {
	timeout := time.Duration(normalizeInt(item.TimeoutMS, 3000, 500, 60000)) * time.Millisecond
	address := net.JoinHostPort(strings.TrimSpace(item.Host), fmt.Sprintf("%d", item.Port))
	started := time.Now()

	dialCtx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	dialer := &net.Dialer{}
	conn, err := dialer.DialContext(dialCtx, "tcp", address)
	if err != nil {
		isTimeout := false
		var netErr net.Error
		if errors.As(err, &netErr) {
			isTimeout = netErr.Timeout()
		}
		return sample{
			NodeID:      item.NodeID,
			IsReachable: false,
			IsTimeout:   isTimeout,
			Error:       truncateError(err),
			SampledAt:   time.Now().Unix(),
		}
	}
	_ = conn.Close()

	latency := time.Since(started).Milliseconds()
	return sample{
		NodeID:      item.NodeID,
		IsReachable: true,
		LatencyMS:   latency,
		SampledAt:   time.Now().Unix(),
	}
}

func truncateError(err error) string {
	if err == nil {
		return ""
	}
	text := err.Error()
	if len(text) > 200 {
		return text[:200]
	}
	return text
}

func normalizeSeconds(value, fallback, minValue, maxValue int) int {
	return normalizeInt(value, fallback, minValue, maxValue)
}

func normalizeInt(value, fallback, minValue, maxValue int) int {
	if value <= 0 {
		value = fallback
	}
	if value < minValue {
		value = minValue
	}
	if value > maxValue {
		value = maxValue
	}
	return value
}

func envOrDefault(value, envKey string) string {
	if strings.TrimSpace(value) != "" {
		return strings.TrimSpace(value)
	}
	return strings.TrimSpace(os.Getenv(envKey))
}

func main() {
	panelFlag := flag.String("panel", "", "Panel base URL")
	tokenFlag := flag.String("token", "", "TCPing agent token")
	versionFlag := flag.Bool("version", false, "Print version")
	flag.Parse()

	if *versionFlag {
		fmt.Println(agentVersion)
		return
	}

	panelURL := envOrDefault(*panelFlag, "PANEL_URL")
	token := envOrDefault(*tokenFlag, "TCPING_AGENT_TOKEN")

	logger := log.New(os.Stdout, "[tcping-agent] ", log.LstdFlags|log.Lmsgprefix)

	api, err := newAPIClient(panelURL, token)
	if err != nil {
		logger.Fatalf("invalid configuration: %v", err)
	}

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	r := newRunner(logger, api)
	if err := r.run(ctx); err != nil {
		logger.Fatalf("agent stopped with error: %v", err)
	}
}
