<?php

namespace App\Console\Commands;

use App\Models\Plan;
use App\Utils\Helper;
use Illuminate\Console\Command;
use App\Models\User;
use Illuminate\Support\Facades\DB;

class ResetUser extends Command
{
    protected $builder;
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'reset:user';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = '重置所有用户信息';

    /**
     * Create a new command instance.
     *
     * @return void
     */
    public function __construct()
    {
        parent::__construct();
    }

    /**
     * Execute the console command.
     *
     * @return mixed
     */
    public function handle()
    {
        if (!$this->confirm("确定要重置所有用户安全信息吗？")) {
            return;
        }
        ini_set('memory_limit', (string) env('APP_MEMORY_LIMIT', '512M'));
        $users = User::all();
        foreach ($users as $user)
        {
            $user->token = Helper::guid();
            $user->uuid = Helper::guid(true);
            $user->subscribe_path = Helper::randomLetters(10);
            $user->subscribe_key = Helper::randomLetters(8);
            $user->subscribe_salt = Helper::randomLetters(6);
            if ($user->subscribe_salt === $user->subscribe_key) {
                $user->subscribe_salt = Helper::randomLetters(6);
            }
            $user->save();
            $this->info("已重置用户{$user->email}的安全信息");
        }
    }
}
