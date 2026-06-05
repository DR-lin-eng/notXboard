<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        if (!Schema::hasTable('server_nodes')) {
            return;
        }

        if (!Schema::hasColumn('server_nodes', 'protocol')
            || !Schema::hasColumn('server_nodes', 'port')
            || !Schema::hasColumn('server_nodes', 'tcping_port')) {
            return;
        }

        // 历史版本会在“未填写 TCPing 探测端口”时默认写入访问端口。
        // 但对 UDP 协议节点（hysteria/hysteria2/tuic）这会导致误下发探测目标。
        $udpProtocols = ['hysteria', 'hysteria2', 'tuic'];

        $nodeIds = DB::table('server_nodes')
            ->whereIn('protocol', $udpProtocols)
            ->whereNotNull('tcping_port')
            ->whereColumn('tcping_port', 'port')
            ->pluck('id')
            ->map(fn ($id) => (int) $id)
            ->all();

        if (empty($nodeIds)) {
            return;
        }

        $update = [
            'tcping_port' => null,
        ];

        $nullableNodeColumns = [
            'tcping_last_status',
            'tcping_last_latency_ms',
            'tcping_last_error',
            'tcping_last_sampled_at',
            'tcping_outage_since',
            'tcping_recovered_since',
        ];
        foreach ($nullableNodeColumns as $column) {
            if (Schema::hasColumn('server_nodes', $column)) {
                $update[$column] = null;
            }
        }
        if (Schema::hasColumn('server_nodes', 'updated_at')) {
            $update['updated_at'] = now();
        }

        DB::table('server_nodes')
            ->whereIn('id', $nodeIds)
            ->update($update);

        // 关闭遗留告警 / 采样，避免“UDP 节点未配置探测端口”仍展示异常。
        $now = time();
        if (Schema::hasTable('tcping_alerts')
            && Schema::hasColumn('tcping_alerts', 'node_id')
            && Schema::hasColumn('tcping_alerts', 'status')) {
            $alertUpdate = [
                'status' => 'resolved',
            ];
            if (Schema::hasColumn('tcping_alerts', 'recovered_at')) {
                $alertUpdate['recovered_at'] = $now;
            }
            if (Schema::hasColumn('tcping_alerts', 'updated_at')) {
                $alertUpdate['updated_at'] = $now;
            }

            DB::table('tcping_alerts')
                ->whereIn('node_id', $nodeIds)
                ->where('status', 'active')
                ->update($alertUpdate);
        }

        if (Schema::hasTable('tcping_samples') && Schema::hasColumn('tcping_samples', 'node_id')) {
            DB::table('tcping_samples')
                ->whereIn('node_id', $nodeIds)
                ->delete();
        }
    }

    public function down(): void
    {
        // no-op: 无法可靠恢复历史默认值
    }
};

