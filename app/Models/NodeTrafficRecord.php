<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Carbon\Carbon;

/**
 * App\Models\NodeTrafficRecord
 *
 * @property int $id
 * @property int $user_id 用户 ID
 * @property int $node_id 节点 ID
 * @property int $upload_traffic 上传流量 (KB)
 * @property int $download_traffic 下载流量 (KB)
 * @property \Illuminate\Support\Carbon $record_date 记录日期
 * @property \Illuminate\Support\Carbon|null $created_at
 * @property \Illuminate\Support\Carbon|null $updated_at
 *
 * @property-read User $user 用户
 * @property-read ServerNode $node 节点
 */
class NodeTrafficRecord extends Model
{
    protected $fillable = [
        'user_id',
        'node_id',
        'upload_traffic',
        'download_traffic',
        'record_date',
    ];

    protected $casts = [
        'user_id' => 'integer',
        'node_id' => 'integer',
        'upload_traffic' => 'integer',
        'download_traffic' => 'integer',
        'record_date' => 'date',
    ];

    /**
     * 关联用户
     */
    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    /**
     * 关联节点
     */
    public function node(): BelongsTo
    {
        return $this->belongsTo(ServerNode::class, 'node_id', 'id');
    }

    /**
     * 获取总流量
     */
    public function getTotalTraffic(): int
    {
        return $this->upload_traffic + $this->download_traffic;
    }

    /**
     * 记录用户在节点的流量使用
     */
    public static function recordTraffic(int $userId, int $nodeId, int $upload, int $download, ?Carbon $date = null): self
    {
        $date = $date ?? now()->toDateString();
        
        return self::updateOrCreate(
            [
                'user_id' => $userId,
                'node_id' => $nodeId,
                'record_date' => $date,
            ],
            [
                'upload_traffic' => \DB::raw("upload_traffic + {$upload}"),
                'download_traffic' => \DB::raw("download_traffic + {$download}"),
            ]
        );
    }

    /**
     * 获取用户在指定时间范围内的流量统计
     */
    public static function getUserTrafficStats(int $userId, Carbon $startDate, Carbon $endDate): array
    {
        $records = self::where('user_id', $userId)
                      ->whereBetween('record_date', [$startDate->toDateString(), $endDate->toDateString()])
                      ->with('node')
                      ->get();

        $totalUpload = $records->sum('upload_traffic');
        $totalDownload = $records->sum('download_traffic');
        
        $nodeStats = $records->groupBy('node_id')->map(function ($nodeRecords) {
            $node = $nodeRecords->first()->node;
            return [
                'node_name' => $node->name,
                'upload_traffic' => $nodeRecords->sum('upload_traffic'),
                'download_traffic' => $nodeRecords->sum('download_traffic'),
                'total_traffic' => $nodeRecords->sum(function ($record) {
                    return $record->upload_traffic + $record->download_traffic;
                }),
                'days_used' => $nodeRecords->count(),
            ];
        });

        return [
            'total_upload' => $totalUpload,
            'total_download' => $totalDownload,
            'total_traffic' => $totalUpload + $totalDownload,
            'node_stats' => $nodeStats->toArray(),
            'days_count' => $records->pluck('record_date')->unique()->count(),
        ];
    }

    /**
     * 获取节点在指定时间范围内的流量统计
     */
    public static function getNodeTrafficStats(int $nodeId, Carbon $startDate, Carbon $endDate): array
    {
        $records = self::where('node_id', $nodeId)
                      ->whereBetween('record_date', [$startDate->toDateString(), $endDate->toDateString()])
                      ->with('user')
                      ->get();

        $totalUpload = $records->sum('upload_traffic');
        $totalDownload = $records->sum('download_traffic');
        
        $userStats = $records->groupBy('user_id')->map(function ($userRecords) {
            $user = $userRecords->first()->user;
            return [
                'user_email' => $user->email,
                'user_name' => $user->linux_do_name ?? $user->email,
                'upload_traffic' => $userRecords->sum('upload_traffic'),
                'download_traffic' => $userRecords->sum('download_traffic'),
                'total_traffic' => $userRecords->sum(function ($record) {
                    return $record->upload_traffic + $record->download_traffic;
                }),
                'days_used' => $userRecords->count(),
            ];
        });

        $dailyStats = $records->groupBy('record_date')->map(function ($dayRecords, $date) {
            return [
                'date' => $date,
                'upload_traffic' => $dayRecords->sum('upload_traffic'),
                'download_traffic' => $dayRecords->sum('download_traffic'),
                'total_traffic' => $dayRecords->sum(function ($record) {
                    return $record->upload_traffic + $record->download_traffic;
                }),
                'unique_users' => $dayRecords->pluck('user_id')->unique()->count(),
            ];
        });

        return [
            'total_upload' => $totalUpload,
            'total_download' => $totalDownload,
            'total_traffic' => $totalUpload + $totalDownload,
            'user_stats' => $userStats->toArray(),
            'daily_stats' => $dailyStats->toArray(),
            'unique_users' => $records->pluck('user_id')->unique()->count(),
            'days_count' => $records->pluck('record_date')->unique()->count(),
        ];
    }

    /**
     * 获取用户今日在指定节点的流量使用
     */
    public static function getTodayUserNodeTraffic(int $userId, int $nodeId): ?self
    {
        return self::where('user_id', $userId)
                   ->where('node_id', $nodeId)
                   ->where('record_date', now()->toDateString())
                   ->first();
    }

    /**
     * 获取用户在所有节点的今日流量
     */
    public static function getTodayUserTraffic(int $userId): \Illuminate\Database\Eloquent\Collection
    {
        return self::where('user_id', $userId)
                   ->where('record_date', now()->toDateString())
                   ->with('node')
                   ->get();
    }

    /**
     * 获取节点今日流量统计
     */
    public static function getTodayNodeTraffic(int $nodeId): array
    {
        $records = self::where('node_id', $nodeId)
                      ->where('record_date', now()->toDateString())
                      ->get();

        return [
            'upload_traffic' => $records->sum('upload_traffic'),
            'download_traffic' => $records->sum('download_traffic'),
            'total_traffic' => $records->sum(function ($record) {
                return $record->upload_traffic + $record->download_traffic;
            }),
            'unique_users' => $records->pluck('user_id')->unique()->count(),
        ];
    }

    /**
     * 清理旧的流量记录
     */
    public static function cleanupOldRecords(int $daysToKeep = 90): int
    {
        return self::where('record_date', '<', now()->subDays($daysToKeep)->toDateString())->delete();
    }

    /**
     * 获取流量趋势数据
     */
    public static function getTrafficTrend(int $nodeId, int $days = 30): array
    {
        $startDate = now()->subDays($days - 1)->toDateString();
        $endDate = now()->toDateString();
        
        $records = self::where('node_id', $nodeId)
                      ->whereBetween('record_date', [$startDate, $endDate])
                      ->selectRaw('record_date, SUM(upload_traffic) as total_upload, SUM(download_traffic) as total_download, COUNT(DISTINCT user_id) as unique_users')
                      ->groupBy('record_date')
                      ->orderBy('record_date')
                      ->get();

        // 填充缺失的日期
        $trend = [];
        for ($i = 0; $i < $days; $i++) {
            $date = now()->subDays($days - 1 - $i)->toDateString();
            $record = $records->firstWhere('record_date', $date);
            
            $trend[] = [
                'date' => $date,
                'upload_traffic' => $record?->total_upload ?? 0,
                'download_traffic' => $record?->total_download ?? 0,
                'total_traffic' => ($record?->total_upload ?? 0) + ($record?->total_download ?? 0),
                'unique_users' => $record?->unique_users ?? 0,
            ];
        }

        return $trend;
    }

    /**
     * 格式化流量大小
     */
    public function getFormattedUploadTraffic(): string
    {
        return $this->formatTraffic($this->upload_traffic);
    }

    /**
     * 格式化流量大小
     */
    public function getFormattedDownloadTraffic(): string
    {
        return $this->formatTraffic($this->download_traffic);
    }

    /**
     * 格式化流量大小
     */
    public function getFormattedTotalTraffic(): string
    {
        return $this->formatTraffic($this->getTotalTraffic());
    }

    /**
     * 格式化流量大小
     */
    private function formatTraffic(int $bytes): string
    {
        $units = ['KB', 'MB', 'GB', 'TB'];
        $bytes = max($bytes, 0);
        $pow = floor(($bytes ? log($bytes) : 0) / log(1024));
        $pow = min($pow, count($units) - 1);
        
        $bytes /= pow(1024, $pow);
        
        return round($bytes, 2) . ' ' . $units[$pow];
    }
}