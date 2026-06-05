<?php

namespace App\Http\Controllers\V2\Admin;

use App\Exceptions\ApiException;
use App\Http\Controllers\Controller;
use App\Http\Requests\Admin\NoticeSave;
use App\Models\Plan;
use App\Models\Notice;
use App\Services\Plugin\HookManager;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

class NoticeController extends Controller
{
    public function fetch(Request $request)
    {
        return $this->success(
            Notice::orderBy('sort', 'ASC')
                ->orderBy('id', 'DESC')
                ->get()
        );
    }

    public function save(NoticeSave $request)
    {
        $data = $request->only([
            'title',
            'content',
            'img_url',
            'tags',
            'show',
            'popup',
            'scope_type',
            'target_plan_ids',
        ]);

        $scopeType = (string) ($data['scope_type'] ?? Notice::SCOPE_GLOBAL);
        if ($scopeType === Notice::SCOPE_PLAN_SUBSCRIBERS) {
            $planIds = collect($data['target_plan_ids'] ?? [])
                ->map(fn ($value) => (int) $value)
                ->filter(fn ($value) => $value > 0)
                ->unique()
                ->values()
                ->all();
            if (empty($planIds)) {
                return $this->fail([422, '请选择至少一个目标套餐']);
            }

            $existingCount = Plan::query()->whereIn('id', $planIds)->count();
            if ($existingCount !== count($planIds)) {
                return $this->fail([422, '目标套餐不存在']);
            }
            $data['target_plan_ids'] = $planIds;
        } else {
            $data['scope_type'] = Notice::SCOPE_GLOBAL;
            $data['target_plan_ids'] = [];
        }

        $isNew = !$request->input('id');
        $publishedNotice = null;

        if ($isNew) {
            $notice = Notice::create($data);
            if (!$notice) {
                return $this->fail([500, '保存失败']);
            }
            $publishedNotice = $notice;
        } else {
            try {
                $notice = Notice::find($request->input('id'));
                if (!$notice) {
                    return $this->fail([400202, '公告不存在']);
                }
                $wasVisible = (bool) ($notice?->show ?? false);
                $notice->update($data);
                $publishedNotice = $notice->fresh();
                if ($wasVisible) {
                    $publishedNotice = null;
                }
            } catch (\Exception $e) {
                return $this->fail([500, '保存失败']);
            }
        }

        if ($publishedNotice && (bool) $publishedNotice->show) {
            HookManager::call('notice.published', [
                'notice' => $publishedNotice,
                'source' => 'admin',
                'author_user_id' => $request->user()?->id,
            ]);
        }
        return $this->success(true);
    }



    public function show(Request $request)
    {
        if (empty($request->input('id'))) {
            return $this->fail([500, '公告ID不能为空']);
        }
        $notice = Notice::find($request->input('id'));
        if (!$notice) {
            return $this->fail([400202, '公告不存在']);
        }
        $notice->show = $notice->show ? 0 : 1;
        if (!$notice->save()) {
            return $this->fail([500, '保存失败']);
        }

        if ((bool) $notice->show) {
            HookManager::call('notice.published', [
                'notice' => $notice->fresh(),
                'source' => 'admin',
                'author_user_id' => $request->user()?->id,
            ]);
        }

        return $this->success(true);
    }

    public function drop(Request $request)
    {
        if (empty($request->input('id'))) {
            return $this->fail([422, '公告ID不能为空']);
        }
        $notice = Notice::find($request->input('id'));
        if (!$notice) {
            return $this->fail([400202, '公告不存在']);
        }
        if (!$notice->delete()) {
            return $this->fail([500, '删除失败']);
        }
        return $this->success(true);
    }

    public function sort(Request $request)
    {
        $params = $request->validate([
            'ids' => 'required|array'
        ]);

        try {
            DB::beginTransaction();
            foreach ($params['ids'] as $k => $v) {
                $notice = Notice::findOrFail($v);
                $notice->update(['sort' => $k + 1]);
            }
            DB::commit();
            return $this->success(true);
        } catch (\Exception $e) {
            DB::rollBack();
            \Log::error($e);
            return $this->fail([500, '排序保存失败']);
        }
    }
}
