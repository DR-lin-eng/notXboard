<?php

namespace Database\Seeders;
use Illuminate\Database\Seeder;

class DatabaseSeeder extends Seeder
{
    /**
     * Seed the application's database.
     *
     * @return void
     */
    public function run()
    {
        // $this->call(UsersTableSeeder::class)

        // Linux DO OAuth 相关种子文件
        $this->call([
            UserGroupLimitSeeder::class,
            LinuxDoOAuthSeeder::class,
        ]);
    }
}
