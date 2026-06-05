package panel

import (
	"encoding/json"
	"fmt"
	"io"
	"strings"

	"github.com/vmihailenco/msgpack/v5"
)

type OnlineUser struct {
	UID int
	IP  string
}

type UserInfo struct {
	Id              int    `json:"id" msgpack:"id"`
	Uuid            string `json:"uuid" msgpack:"uuid"`
	SpeedLimit      int    `json:"speed_limit" msgpack:"speed_limit"`
	DeviceLimit     int    `json:"device_limit" msgpack:"device_limit"`
	ConnectionLimit int    `json:"connection_limit" msgpack:"connection_limit"`
	TrustLevel      int    `json:"trust_level" msgpack:"trust_level"`
	IsSilenced      bool   `json:"is_silenced" msgpack:"is_silenced"`
}

type UserListBody struct {
	Users []UserInfo `json:"users" msgpack:"users"`
}

type UserDeltaBody struct {
	Mode       string     `json:"mode" msgpack:"mode"`
	Version    string     `json:"version" msgpack:"version"`
	Upserts    []UserInfo `json:"upserts" msgpack:"upserts"`
	RemovedIDs []int      `json:"removed_ids" msgpack:"removed_ids"`
}

type UserSyncResult struct {
	Mode        string
	Version     string
	Full        []UserInfo
	Upserts     []UserInfo
	RemovedIDs  []int
	NotModified bool
}

type AliveMap struct {
	Alive map[int]int `json:"alive"`
}

// GetUserList will pull user from v2board
func (c *Client) GetUserList() (*UserSyncResult, error) {
	const path = "/api/v1/server/UniProxy/user"
	r, err := c.client.R().
		SetHeader("If-None-Match", c.userEtag).
		SetHeader("X-User-Sync-Mode", "delta").
		SetHeader("X-User-Snapshot-Version", c.userSnapshotVersion).
		SetHeader("X-Response-Format", "msgpack").
		SetDoNotParseResponse(true).
		Get(path)
	if r == nil || r.RawResponse == nil {
		return nil, fmt.Errorf("received nil response or raw response")
	}
	defer r.RawResponse.Body.Close()

	if r.StatusCode() == 304 {
		return &UserSyncResult{
			Mode:        "not_modified",
			Version:     c.userSnapshotVersion,
			NotModified: true,
		}, nil
	}

	if err = c.checkResponse(r, path, err); err != nil {
		return nil, err
	}
	version := r.Header().Get("X-User-Snapshot-Version")
	if version == "" {
		version = c.userSnapshotVersion
	}
	if strings.Contains(r.Header().Get("Content-Type"), "application/x-msgpack") {
		decoder := msgpack.NewDecoder(r.RawResponse.Body)
		var body map[string]any
		if err := decoder.Decode(&body); err != nil {
			return nil, fmt.Errorf("decode user list error: %w", err)
		}
		result, err := decodeUserSyncResultFromMap(body, version)
		if err != nil {
			return nil, err
		}
		c.userEtag = r.Header().Get("ETag")
		c.userSnapshotVersion = result.Version
		return result, nil
	} else {
		payload, err := io.ReadAll(r.RawResponse.Body)
		if err != nil {
			return nil, fmt.Errorf("decode user list error: %w", err)
		}
		var body map[string]any
		if err := json.Unmarshal(payload, &body); err != nil {
			return nil, fmt.Errorf("decode user list error: %w", err)
		}
		result, err := decodeUserSyncResultFromMap(body, version)
		if err != nil {
			return nil, err
		}
		c.userEtag = r.Header().Get("ETag")
		c.userSnapshotVersion = result.Version
		return result, nil
	}
}

// GetUserAlive will fetch the alive_ip count for users
func (c *Client) GetUserAlive() (map[int]int, error) {
	c.AliveMap = &AliveMap{}
	const path = "/api/v1/server/UniProxy/alivelist"
	r, err := c.client.R().
		ForceContentType("application/json").
		Get(path)
	if err != nil || r.StatusCode() >= 399 {
		c.AliveMap.Alive = make(map[int]int)
		return c.AliveMap.Alive, nil
	}
	if r == nil || r.RawResponse == nil {
		fmt.Printf("received nil response or raw response")
		c.AliveMap.Alive = make(map[int]int)
		return c.AliveMap.Alive, nil
	}
	defer r.RawResponse.Body.Close()
	if err := json.Unmarshal(r.Body(), c.AliveMap); err != nil {
		fmt.Printf("unmarshal user alive list error: %s", err)
		c.AliveMap.Alive = make(map[int]int)
	}

	return c.AliveMap.Alive, nil
}

func decodeUserSyncResultFromMap(body map[string]any, fallbackVersion string) (*UserSyncResult, error) {
	mode, _ := body["mode"].(string)
	if mode == "delta" {
		raw, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("decode user delta error: %w", err)
		}
		var delta UserDeltaBody
		if err := json.Unmarshal(raw, &delta); err != nil {
			return nil, fmt.Errorf("decode user delta error: %w", err)
		}
		if delta.Version == "" {
			delta.Version = fallbackVersion
		}
		return &UserSyncResult{
			Mode:       "delta",
			Version:    delta.Version,
			Upserts:    delta.Upserts,
			RemovedIDs: delta.RemovedIDs,
		}, nil
	}

	raw, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("decode user list error: %w", err)
	}
	var full UserListBody
	if err := json.Unmarshal(raw, &full); err != nil {
		return nil, fmt.Errorf("decode user list error: %w", err)
	}
	return &UserSyncResult{
		Mode:    "full",
		Version: fallbackVersion,
		Full:    full.Users,
	}, nil
}

type UserTraffic struct {
	UID      int
	Upload   int64
	Download int64
}

// ReportUserTraffic reports the user traffic
func (c *Client) ReportUserTraffic(userTraffic []UserTraffic) error {
	data := make(map[int][]int64, len(userTraffic))
	for i := range userTraffic {
		data[userTraffic[i].UID] = []int64{userTraffic[i].Upload, userTraffic[i].Download}
	}
	const path = "/api/v1/server/UniProxy/push"
	r, err := c.client.R().
		SetBody(data).
		ForceContentType("application/json").
		Post(path)
	err = c.checkResponse(r, path, err)
	if err != nil {
		return err
	}
	return nil
}

func (c *Client) ReportNodeOnlineUsers(data *map[int][]string) error {
	const path = "/api/v1/server/UniProxy/alive"
	r, err := c.client.R().
		SetBody(data).
		ForceContentType("application/json").
		Post(path)
	err = c.checkResponse(r, path, err)

	if err != nil {
		return nil
	}

	return nil
}
