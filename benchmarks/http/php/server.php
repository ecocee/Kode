<?php
// HTTP benchmark server: PHP + Swoole. Workers = THREADS.

$port    = (int) ltrim($argv[1] ?? '8080', ':');
$workers = max(1, (int) (getenv('THREADS') ?: 1));

$server = new Swoole\Http\Server('0.0.0.0', $port);
$server->set([
    'worker_num'       => $workers,
    'log_level'        => SWOOLE_LOG_ERROR,
    'http_compression' => false,
    'max_conn'         => 10000,
    'backlog'          => 4096,
]);

const HELLO = 'hello';
const JSON_DOC = '{"message":"hello","ok":true,"n":1}';

$server->on('request', function ($req, $res) {
    if ($req->server['request_uri'] === '/json') {
        $res->header('Content-Type', 'application/json');
        $res->end(JSON_DOC);
        return;
    }
    $res->header('Content-Type', 'text/plain');
    $res->end(HELLO);
});

$server->on('start', function () use ($port, $workers) {
    echo "listening on $port workers=$workers\n";
});

$server->start();