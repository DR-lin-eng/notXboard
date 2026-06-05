<?php

namespace Database\Seeders;

use App\Models\UserGroupLimit;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\Log;

class UserGroupLimitSeeder extends Seeder
{
    /**
     * 运行数据库种子
     */
    public function run(): void
    {
        Log::info('Starting UserGroupLimit seeding');
        
        $groupLimits = [
            [
                'trust_level' => 0,
                'speed_limit_up' => 50,      // 50 Mbps 上传
                'speed_limit_down' => 100,   // 100 Mbps 下载
                'device_limit' => 2,         // 2 个设备
                'connection_limit' => 5,     // 5 个连接
            ],
            [
                'trust_level' => 1,
                'speed_limit_up' => 100,     // 100 Mbps 上传
                'speed_limit_down' => 200,   // 200 Mbps 下载
                'device_limit' => 3,         // 3 个设备
                'connection_limit' => 10,    // 10 个连接
            ],
            [
                'trust_level' => 2,
                'speed_limit_up' => 200,     // 200 Mbps 上传
                'speed_limit_down' => 500,   // 500 Mbps 下载
                'device_limit' => 5,         // 5 个设备
                'connection_limit' => 20,    // 20 个连接
            ],
            [
                'trust_level' => 3,
                'speed_limit_up' => 500,     // 500 Mbps 上传
                'speed_limit_down' => 1000,  // 1000 Mbps 下载
                'device_limit' => 8,         // 8 个设备
                'connection_limit' => 50,    // 50 个连接
            ],
            [
                'trust_level' => 4,
                'speed_limit_up' => 1000,    // 1000 Mbps 上传
                'speed_limit_down' => 2000,  // 2000 Mbps 下载
                'device_limit' => 15,        // 15 个设备
                'connection_limit' => 100,   // 100 个连接
            ],
        ];
        
        foreach ($groupLimits as $limit) {
            UserGroupLimit::updateOrCreate(
                ['trust_level' => $limit['trust_level']],
                $limit
            );
            
            Log::info('Created/updated group limit', [
                'trust_level' => $limit['trust_level'],
                'limits' => $limit
            ]);
        }
        
        Log::info('UserGroupLimit seeding completed', [
            'total_levels' => count($groupLimits)
        ]);
    }
}