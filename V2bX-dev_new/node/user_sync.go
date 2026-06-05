package node

import (
	"fmt"

	"github.com/InazumaV/V2bX/api/panel"
)

type UserApplyPlan struct {
	FullReload bool
	NextUsers  []panel.UserInfo
	Added      []panel.UserInfo
	Deleted    []panel.UserInfo
	Changed    []panel.UserInfo
	RemovedIDs []int
}

func buildUserApplyPlan(current []panel.UserInfo, syncResult *panel.UserSyncResult) (*UserApplyPlan, error) {
	if syncResult == nil || syncResult.NotModified {
		return &UserApplyPlan{NextUsers: current}, nil
	}

	switch syncResult.Mode {
	case "", "full":
		deleted, added, changedOld, changedNew := diffUsers(current, syncResult.Full)
		return &UserApplyPlan{
			FullReload: true,
			NextUsers:  cloneUsers(syncResult.Full),
			Added:      append(append([]panel.UserInfo{}, added...), changedNew...),
			Deleted:    append(append([]panel.UserInfo{}, deleted...), changedOld...),
			Changed:    changedNew,
		}, nil
	case "delta":
		nextUsers, deleted, added, changed, err := applyDeltaUsers(current, syncResult.Upserts, syncResult.RemovedIDs)
		if err != nil {
			return nil, err
		}
		return &UserApplyPlan{
			NextUsers:  nextUsers,
			Added:      append([]panel.UserInfo{}, added...),
			Deleted:    append([]panel.UserInfo{}, deleted...),
			Changed:    changed,
			RemovedIDs: append([]int{}, syncResult.RemovedIDs...),
		}, nil
	default:
		return nil, fmt.Errorf("unsupported user sync mode: %s", syncResult.Mode)
	}
}

func applyDeltaUsers(current []panel.UserInfo, upserts []panel.UserInfo, removedIDs []int) ([]panel.UserInfo, []panel.UserInfo, []panel.UserInfo, []panel.UserInfo, error) {
	currentByID := make(map[int]panel.UserInfo, len(current))
	for _, user := range current {
		currentByID[user.Id] = user
	}

	deleted := make([]panel.UserInfo, 0)
	for _, id := range removedIDs {
		if existing, ok := currentByID[id]; ok {
			deleted = append(deleted, existing)
			delete(currentByID, id)
		}
	}

	added := make([]panel.UserInfo, 0)
	changed := make([]panel.UserInfo, 0)
	for _, user := range upserts {
		if existing, ok := currentByID[user.Id]; ok {
			if usersEqual(existing, user) {
				continue
			}
			deleted = append(deleted, existing)
			added = append(added, user)
			changed = append(changed, user)
		} else {
			added = append(added, user)
		}
		currentByID[user.Id] = user
	}

	nextUsers := make([]panel.UserInfo, 0, len(currentByID))
	seen := make(map[int]struct{}, len(currentByID))
	for _, user := range current {
		if updated, ok := currentByID[user.Id]; ok {
			nextUsers = append(nextUsers, updated)
			seen[user.Id] = struct{}{}
		}
	}
	for _, user := range upserts {
		if _, ok := seen[user.Id]; ok {
			continue
		}
		nextUsers = append(nextUsers, user)
		seen[user.Id] = struct{}{}
	}

	return nextUsers, deleted, added, changed, nil
}

func diffUsers(oldUsers, newUsers []panel.UserInfo) ([]panel.UserInfo, []panel.UserInfo, []panel.UserInfo, []panel.UserInfo) {
	oldByID := make(map[int]panel.UserInfo, len(oldUsers))
	for _, user := range oldUsers {
		oldByID[user.Id] = user
	}
	newByID := make(map[int]panel.UserInfo, len(newUsers))
	for _, user := range newUsers {
		newByID[user.Id] = user
	}

	deleted := make([]panel.UserInfo, 0)
	for _, user := range oldUsers {
		if _, ok := newByID[user.Id]; !ok {
			deleted = append(deleted, user)
		}
	}

	added := make([]panel.UserInfo, 0)
	changedOld := make([]panel.UserInfo, 0)
	changedNew := make([]panel.UserInfo, 0)
	for _, user := range newUsers {
		if oldUser, ok := oldByID[user.Id]; ok {
			if !usersEqual(oldUser, user) {
				changedOld = append(changedOld, oldUser)
				changedNew = append(changedNew, user)
			}
			continue
		}
		added = append(added, user)
	}

	return deleted, added, changedOld, changedNew
}

func usersEqual(a, b panel.UserInfo) bool {
	return a.Id == b.Id &&
		a.Uuid == b.Uuid &&
		a.SpeedLimit == b.SpeedLimit &&
		a.DeviceLimit == b.DeviceLimit &&
		a.ConnectionLimit == b.ConnectionLimit &&
		a.TrustLevel == b.TrustLevel &&
		a.IsSilenced == b.IsSilenced
}

func cloneUsers(users []panel.UserInfo) []panel.UserInfo {
	if len(users) == 0 {
		return nil
	}
	out := make([]panel.UserInfo, len(users))
	copy(out, users)
	return out
}
