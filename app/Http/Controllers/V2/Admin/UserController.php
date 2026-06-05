<?php

namespace App\Http\Controllers\V2\Admin;

use App\Http\Controllers\Controller;
use App\Http\Requests\Admin\UserGenerate;
use App\Http\Requests\Admin\UserSendMail;
use App\Http\Requests\Admin\UserUpdate;
use App\Models\Plan;
use App\Models\UserBanRecord;
use App\Models\User;
use App\Services\UserBanService;
use App\Services\UserService;
use App\Traits\QueryOperators;
use App\Utils\Helper;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;

use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Illuminate\Validation\ValidationException;

class UserController extends Controller
{
    use QueryOperators;

    private const FILTERABLE_FIELDS = [
        'id',
        'invite_user_id',
        'telegram_id',
        'email',
        'balance',
        'discount',
        'commission_type',
        'commission_rate',
        'commission_balance',
        't',
        'u',
        'd',
        'transfer_enable',
        'banned',
        'is_admin',
        'is_staff',
        'is_super_admin',
        'last_login_at',
        'uuid',
        'group_id',
        'group_ids',
        'plan_id',
        'speed_limit',
        'remind_expire',
        'remind_traffic',
        'token',
        'subscribe_path',
        'subscribe_key',
        'subscribe_salt',
        'expired_at',
        'remarks',
        'linux_do_id',
        'linux_do_username',
        'linux_do_name',
        'trust_level',
        'is_silenced',
        'api_key',
        'device_limit',
        'concurrent_ip_limit',
        'created_at',
        'updated_at',
        'total_used',
    ];

    private const SORTABLE_FIELDS = [
        'id',
        'invite_user_id',
        'telegram_id',
        'email',
        'balance',
        'discount',
        'commission_type',
        'commission_rate',
        'commission_balance',
        't',
        'u',
        'd',
        'transfer_enable',
        'banned',
        'is_admin',
        'is_staff',
        'is_super_admin',
        'last_login_at',
        'group_id',
        'plan_id',
        'speed_limit',
        'remind_expire',
        'remind_traffic',
        'expired_at',
        'trust_level',
        'is_silenced',
        'device_limit',
        'concurrent_ip_limit',
        'created_at',
        'updated_at',
        'total_used',
    ];

    private const RELATION_FILTERABLE_FIELDS = [
        'plan' => ['id', 'name'],
        'invite_user' => ['id', 'email', 'linux_do_username', 'linux_do_name'],
        'group' => ['id', 'name'],
    ];

    private function normalizeBaseFilterField(mixed $field): string
    {
        if (!is_string($field) || !in_array($field, self::FILTERABLE_FIELDS, true)) {
            throw ValidationException::withMessages([
                'filter' => ['包含非法用户筛选字段'],
            ]);
        }

        return $field === 'group_ids' ? 'group_id' : $field;
    }

    private function normalizeSortField(mixed $field): string
    {
        if (!is_string($field) || !in_array($field, self::SORTABLE_FIELDS, true)) {
            throw ValidationException::withMessages([
                'sort' => ['包含非法用户排序字段'],
            ]);
        }

        return $field;
    }

    private function parseRelationFilterField(string $field): ?array
    {
        if (!str_contains($field, '.')) {
            return null;
        }

        [$relation, $relationField] = explode('.', $field, 2);
        $allowedFields = self::RELATION_FILTERABLE_FIELDS[$relation] ?? null;

        if (!$allowedFields || !in_array($relationField, $allowedFields, true)) {
            throw ValidationException::withMessages([
                'filter' => ['包含非法用户关联筛选字段'],
            ]);
        }

        return [$relation, $relationField];
    }

    private function applySafeSort(Builder $builder, string $field, string $direction): void
    {
        if ($field === 'total_used') {
            $builder->orderByRaw('(u + d) ' . $direction);
            return;
        }

        $builder->orderBy($field, $direction);
    }

    public function resetSecret(Request $request)
    {
        $user = User::find($request->input('id'));
        if (!$user)
            return $this->fail([400202, '用户不存在']);
        $user->token = Helper::guid();
        $user->uuid = Helper::guid(true);
        $user->subscribe_path = Helper::randomLetters(10);
        $user->subscribe_key = Helper::randomLetters(8);
        $user->subscribe_salt = Helper::randomLetters(6);
        if ($user->subscribe_salt === $user->subscribe_key) {
            $user->subscribe_salt = Helper::randomLetters(6);
        }
        return $this->success($user->save());
    }

    /**
     * Apply filters and sorts to the query builder
     *
     * @param Request $request
     * @param Builder $builder
     * @return void
     */
    private function applyFiltersAndSorts(Request $request, Builder $builder): void
    {
        $this->applyFilters($request, $builder);
        $this->applySorting($request, $builder);
    }

    /**
     * Apply filters to the query builder
     *
     * @param Request $request
     * @param Builder $builder
     * @return void
     */
    private function applyFilters(Request $request, Builder $builder): void
    {
        if (!$request->has('filter')) {
            return;
        }

        collect($request->input('filter'))->each(function ($filter) use ($builder) {
            $field = $filter['id'] ?? null;
            if (!is_string($field)) {
                throw ValidationException::withMessages([
                    'filter' => ['包含非法用户筛选字段'],
                ]);
            }

            $value = $filter['value'];

            $builder->where(function ($query) use ($field, $value) {
                $this->buildFilterQuery($query, $field, $value);
            });
        });
    }

    /**
     * Build the filter query based on field and value
     *
     * @param Builder $query
     * @param string $field
     * @param mixed $value
     * @return void
     */
    private function buildFilterQuery(Builder $query, string $field, mixed $value): void
    {
        // 处理关联查询
        if ($relationField = $this->parseRelationFilterField($field)) {
            [$relation, $relationField] = $relationField;
            $query->whereHas($relation, function ($q) use ($relationField, $value) {
                if (is_array($value)) {
                    $q->whereIn($relationField, $value);
                } else if (is_string($value) && str_contains($value, ':')) {
                    [$operator, $filterValue] = explode(':', $value, 2);
                    $this->applyQueryCondition($q, $relationField, $operator, $filterValue);
                } else {
                    $q->where($relationField, 'like', "%{$value}%");
                }
            });
            return;
        }

        $field = $this->normalizeBaseFilterField($field);

        // 处理数组值的 'in' 操作
        if (is_array($value)) {
            $query->whereIn($field, $value);
            return;
        }

        // 处理基于运算符的过滤
        if (!is_string($value) || !str_contains($value, ':')) {
            if ($field === 'total_used' && is_numeric($value)) {
                $query->where(DB::raw('(u + d)'), '=', (float) $value);
                return;
            }

            $query->where($field, 'like', "%{$value}%");
            return;
        }

        [$operator, $filterValue] = explode(':', $value, 2);

        // 转换数字字符串为适当的类型
        if (is_numeric($filterValue)) {
            $filterValue = strpos($filterValue, '.') !== false
                ? (float) $filterValue
                : (int) $filterValue;
        }

        // 处理计算字段
        $queryField = match ($field) {
            'total_used' => DB::raw('(u + d)'),
            default => $field
        };

        $this->applyQueryCondition($query, $queryField, $operator, $filterValue);
    }

    /**
     * Apply sorting to the query builder
     *
     * @param Request $request
     * @param Builder $builder
     * @return void
     */
    private function applySorting(Request $request, Builder $builder): void
    {
        if (!$request->has('sort')) {
            return;
        }

        collect($request->input('sort'))->each(function ($sort) use ($builder) {
            $field = $this->normalizeSortField($sort['id'] ?? null);
            $direction = $sort['desc'] ? 'DESC' : 'ASC';
            $this->applySafeSort($builder, $field, $direction);
        });
    }

    /**
     * Fetch paginated user list with filters and sorting
     *
     * @param Request $request
     * @return \Illuminate\Http\Response
     */
    public function fetch(Request $request)
    {
        $current = $request->input('current', 1);
        $pageSize = $request->input('pageSize', 10);

        $userModel = User::with(['plan:id,name', 'invite_user:id,email', 'group:id,name'])
            ->select(DB::raw('*, (u+d) as total_used'));

        $this->applyFiltersAndSorts($request, $userModel);

        $users = $userModel->orderBy('id', 'desc')
            ->paginate($pageSize, ['*'], 'page', $current);

        $users->getCollection()->transform(function ($user): array {
            return self::transformUserData($user);
        });

        return $this->paginate($users);
    }

    /**
     * Transform user data for response
     *
     * @param User $user
     * @return array<string, mixed>
     */
    public static function transformUserData(User $user): array
    {
        $user = $user->toArray();
        $user['balance'] = $user['balance'] / 100;
        $user['commission_balance'] = $user['commission_balance'] / 100;
        $user['subscribe_url'] = Helper::getSubscribeUrl($user);
        return $user;
    }

    public function getUserInfoById(Request $request)
    {
        $request->validate([
            'id' => 'required|numeric'
        ], [
            'id.required' => '用户ID不能为空'
        ]);
        $user = User::find($request->input('id'));
        if (!$user) {
            return $this->fail([400202, '用户不存在']);
        }

        $user->load('invite_user');
        return $this->success($user);
    }

    public function update(UserUpdate $request)
    {
        $params = $request->validated();

        $user = User::find($request->input('id'));
        if (!$user) {
            return $this->fail([400202, '用户不存在']);
        }
        if (isset($params['email'])) {
            if (User::where('email', $params['email'])->first() && $user->email !== $params['email']) {
                return $this->fail([400201, '邮箱已被使用']);
            }
        }
        // 处理密码
        if (isset($params['password'])) {
            $params['password'] = password_hash($params['password'], PASSWORD_DEFAULT);
            $params['password_algo'] = NULL;
        } else {
            unset($params['password']);
        }
        // 处理订阅计划
        if (isset($params['plan_id'])) {
            $plan = Plan::find($params['plan_id']);
            if (!$plan) {
                return $this->fail([400202, '订阅计划不存在']);
            }
            $params['group_id'] = $plan->group_id;
        }
        // 处理邀请用户
        if ($request->input('invite_user_email') && $inviteUser = User::where('email', $request->input('invite_user_email'))->first()) {
            $params['invite_user_id'] = $inviteUser->id;
        } else {
            $params['invite_user_id'] = null;
        }

        $targetBanState = array_key_exists('banned', $params) ? (bool) $params['banned'] : null;
        $banReason = trim((string) ($params['ban_reason'] ?? ''));
        unset($params['banned'], $params['ban_reason']);

        if (isset($params['balance'])) {
            $params['balance'] = $params['balance'] * 100;
        }
        if (isset($params['commission_balance'])) {
            $params['commission_balance'] = $params['commission_balance'] * 100;
        }

        try {
            if (!empty($params)) {
                $user->update($params);
            }

            $banService = app(UserBanService::class);
            if ($targetBanState === true) {
                $currentBanReason = trim((string) ($user->ban_reason ?? ''));
                $effectiveReason = $banReason !== '' ? $banReason : trim((string) ($user->ban_reason ?? ''));
                if ($effectiveReason === '') {
                    return $this->fail([422, '封禁用户时必须填写封禁原因']);
                }

                if (!$user->banned || $effectiveReason !== $currentBanReason) {
                    $banService->banUser($user, $request->user(), $effectiveReason, [
                        'source' => 'manual',
                    ]);
                }
            } elseif ($targetBanState === false && $user->banned) {
                $banService->unbanUser($user, $request->user(), $banReason !== '' ? $banReason : null, [
                    'source' => 'manual',
                ]);
            }
        } catch (\Exception $e) {
            Log::error($e);
            return $this->fail([500, '保存失败']);
        }
        return $this->success(true);
    }

    /**
     * 导出用户数据为CSV格式
     *
     * @param Request $request
     * @return \Symfony\Component\HttpFoundation\StreamedResponse
     */
    public function dumpCSV(Request $request)
    {
        ini_set('memory_limit', (string) env('APP_MEMORY_LIMIT', '512M'));
        gc_enable(); // 启用垃圾回收

        // 优化查询：使用with预加载plan关系，避免N+1问题
        $query = User::with('plan:id,name')
            ->orderBy('id', 'asc')
            ->select([
                'email',
                'balance',
                'commission_balance',
                'transfer_enable',
                'u',
                'd',
                'expired_at',
                'token',
                'subscribe_path',
                'subscribe_key',
                'subscribe_salt',
                'plan_id'
            ]);

        $this->applyFiltersAndSorts($request, $query);

        $filename = 'users_' . date('Y-m-d_His') . '.csv';

        return response()->streamDownload(function () use ($query) {
            // 打开输出流
            $output = fopen('php://output', 'w');

            // 添加BOM标记，确保Excel正确显示中文
            fprintf($output, chr(0xEF) . chr(0xBB) . chr(0xBF));

            // 写入CSV头部
            fputcsv($output, [
                '邮箱',
                '余额',
                '推广佣金',
                '总流量',
                '剩余流量',
                '套餐到期时间',
                '订阅计划',
                '订阅地址'
            ]);

            // 分批处理数据以减少内存使用
            $query->chunk(500, function ($users) use ($output) {
                foreach ($users as $user) {
                    try {
                        $row = [
                            Helper::sanitizeForCsv($user->email),
                            Helper::sanitizeForCsv(number_format($user->balance / 100, 2)),
                            Helper::sanitizeForCsv(number_format($user->commission_balance / 100, 2)),
                            Helper::sanitizeForCsv(Helper::trafficConvert($user->transfer_enable)),
                            Helper::sanitizeForCsv(Helper::trafficConvert($user->transfer_enable - ($user->u + $user->d))),
                            Helper::sanitizeForCsv($user->expired_at ? date('Y-m-d H:i:s', $user->expired_at) : '长期有效'),
                            Helper::sanitizeForCsv($user->plan ? $user->plan->name : '无订阅'),
                            Helper::sanitizeForCsv(Helper::getSubscribeUrl($user))
                        ];
                        fputcsv($output, $row);
                    } catch (\Exception $e) {
                        Log::error('CSV导出错误: ' . $e->getMessage(), [
                            'user_id' => $user->id,
                            'email' => $user->email
                        ]);
                        continue; // 继续处理下一条记录
                    }
                }

                // 清理内存
                gc_collect_cycles();
            });

            fclose($output);
        }, $filename, [
            'Content-Type' => 'text/csv; charset=UTF-8',
            'Content-Disposition' => 'attachment; filename="' . $filename . '"'
        ]);
    }

    public function generate(UserGenerate $request)
    {
        if ($request->input('email_prefix')) {
            $email = $request->input('email_prefix') . '@' . $request->input('email_suffix');

            if (User::where('email', $email)->exists()) {
                return $this->fail([400201, '邮箱已存在于系统中']);
            }

            $userService = app(UserService::class);
            $user = $userService->createUser([
                'email' => $email,
                'password' => $request->input('password') ?? $email,
                'plan_id' => $request->input('plan_id'),
                'expired_at' => $request->input('expired_at'),
            ]);

            if (!$user->save()) {
                return $this->fail([500, '生成失败']);
            }
            return $this->success(true);
        }

        if ($request->input('generate_count')) {
            return $this->multiGenerate($request);
        }
    }

    private function multiGenerate(Request $request)
    {
        $userService = app(UserService::class);
        $usersData = [];

        for ($i = 0; $i < $request->input('generate_count'); $i++) {
            $email = Helper::randomChar(6) . '@' . $request->input('email_suffix');
            $usersData[] = [
                'email' => $email,
                'password' => $request->input('password') ?? $email,
                'plan_id' => $request->input('plan_id'),
                'expired_at' => $request->input('expired_at'),
            ];
        }



        try {
            DB::beginTransaction();
            $users = [];
            foreach ($usersData as $userData) {
                $user = $userService->createUser($userData);
                $user->save();
                $users[] = $user;
            }
            DB::commit();
        } catch (\Exception $e) {
            DB::rollBack();
            return $this->fail([500, '生成失败']);
        }

        // 判断是否导出 CSV
        if ($request->input('download_csv')) {
            $headers = [
                'Content-Type' => 'text/csv',
                'Content-Disposition' => 'attachment; filename="users.csv"',
            ];
            $callback = function () use ($users, $request) {
                $handle = fopen('php://output', 'w');
                fputcsv($handle, ['账号', '密码', '过期时间', 'UUID', '创建时间', '订阅地址']);
                foreach ($users as $user) {
                    $user = $user->refresh();
                    $expireDate = $user['expired_at'] === NULL ? '长期有效' : date('Y-m-d H:i:s', $user['expired_at']);
                    $createDate = date('Y-m-d H:i:s', $user['created_at']);
                    $password = $request->input('password') ?? $user['email'];
                    $subscribeUrl = Helper::getSubscribeUrl($user);
                    fputcsv($handle, [
                        Helper::sanitizeForCsv($user['email']),
                        Helper::sanitizeForCsv($password),
                        Helper::sanitizeForCsv($expireDate),
                        Helper::sanitizeForCsv($user['uuid']),
                        Helper::sanitizeForCsv($createDate),
                        Helper::sanitizeForCsv($subscribeUrl),
                    ]);
                }
                fclose($handle);
            };
            return response()->streamDownload($callback, 'users.csv', $headers);
        }

        // 默认返回 JSON
        $data = collect($users)->map(function ($user) use ($request) {
            return [
                'email' => $user['email'],
                'password' => $request->input('password') ?? $user['email'],
                'expired_at' => $user['expired_at'] === NULL ? '长期有效' : date('Y-m-d H:i:s', $user['expired_at']),
                'uuid' => $user['uuid'],
                'created_at' => date('Y-m-d H:i:s', $user['created_at']),
                'subscribe_url' => Helper::getSubscribeUrl($user),
            ];
        });
        return response()->json([
            'code' => 0,
            'message' => '批量生成成功',
            'data' => $data,
        ]);
    }

    public function sendMail(UserSendMail $request)
    {
        ini_set('memory_limit', (string) env('APP_MEMORY_LIMIT', '512M'));
        $sortType = in_array($request->input('sort_type'), ['ASC', 'DESC']) ? $request->input('sort_type') : 'DESC';
        $sort = $this->normalizeSortField($request->input('sort') ?: 'created_at');
        $builder = User::query();
        $this->applySafeSort($builder, $sort, $sortType);
        $this->applyFiltersAndSorts($request, $builder);

        $subject = $request->input('subject');
        $content = $request->input('content');
        $templateValue = [
            'name' => admin_setting('app_name', 'Portal'),
            'url' => admin_setting('app_url'),
            'content' => $content
        ];

        $chunkSize = 1000;

        $builder->chunk($chunkSize, function ($users) use ($subject, $templateValue, &$totalProcessed) {
            foreach ($users as $user) {
                \App\Services\MailService::dispatchEmail([
                    'email' => $user->email,
                    'subject' => $subject,
                    'template_name' => 'notify',
                    'template_value' => $templateValue
                ], 'send_email_mass');
            }
        });

        return $this->success(true);
    }

    public function ban(Request $request)
    {
        $request->validate([
            'reason' => 'required|string|max:500',
        ], [
            'reason.required' => '批量封禁时必须填写封禁原因',
            'reason.max' => '封禁原因长度不能超过500个字符',
        ]);

        $builder = User::query();
        $this->applyFilters($request, $builder);
        $builder->where('banned', 0)->orderBy('id');
        $reason = trim((string) $request->input('reason'));
        $banService = app(UserBanService::class);
        $affected = 0;

        try {
            foreach ($builder->cursor() as $user) {
                $banService->banUser($user, $request->user(), $reason, [
                    'source' => 'batch',
                    'notify' => false,
                ]);
                $affected++;
            }
        } catch (\Exception $e) {
            Log::error($e);
            return $this->fail([500, '处理失败']);
        }

        return $this->success([
            'updated' => $affected,
        ]);
    }

    public function banRecords(Request $request)
    {
        $current = max(1, (int) $request->input('current', 1));
        $pageSize = max(1, min(100, (int) $request->input('pageSize', 20)));

        $query = UserBanRecord::query()
            ->with([
                'user:id,email',
                'admin:id,email',
            ])
            ->orderByDesc('id');

        if ($request->filled('user_id')) {
            $query->where('user_id', (int) $request->input('user_id'));
        }

        if ($request->filled('action')) {
            $query->where('action', (string) $request->input('action'));
        }

        $records = $query->paginate($pageSize, ['*'], 'page', $current);

        $records->getCollection()->transform(function (UserBanRecord $record): array {
            return [
                'id' => (int) $record->id,
                'user_id' => (int) $record->user_id,
                'user_email' => (string) ($record->user?->email ?? '-'),
                'admin_id' => $record->admin_id ? (int) $record->admin_id : null,
                'admin_email' => (string) ($record->admin?->email ?? 'system'),
                'action' => (string) $record->action,
                'reason' => (string) $record->reason,
                'source' => (string) $record->source,
                'context' => $record->context,
                'created_at' => optional($record->created_at)->timestamp,
            ];
        });

        return $this->paginate($records);
    }

    /**
     * 删除用户及其关联数据
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function destroy(Request $request)
    {
        $request->validate([
            'id' => 'required|exists:App\Models\User,id'
        ], [
            'id.required' => '用户ID不能为空',
            'id.exists' => '用户不存在'
        ]);
        $user = User::find($request->input('id'));
        try {
            DB::beginTransaction();
            $user->orders()->delete();
            $user->codes()->delete();
            $user->stat()->delete();
            $user->tickets()->delete();
            $user->delete();
            DB::commit();
            return $this->success(true);
        } catch (\Exception $e) {
            DB::rollBack();
            Log::error($e);
            return $this->fail([500, '删除失败']);
        }
    }
}
