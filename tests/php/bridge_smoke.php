<?php

declare(strict_types=1);

/*
 * Runs the published PHP native bridge against the C library built here.
 *
 *     php -d ffi.enable=1 tests/php/bridge_smoke.php <vendor/autoload.php> <shared library>
 *
 * The bridge must select the library when it is healthy, answer the same as the pure-PHP
 * engine, and fall back to that engine when the library path is unusable.
 */

if ($argc !== 3) {
    fwrite(STDERR, "usage: php bridge_smoke.php <autoload.php> <shared library>\n");
    exit(2);
}
require $argv[1];
$library = $argv[2];

if (!extension_loaded('ffi')) {
    throw new RuntimeException('ext-ffi is not loaded, so the library cannot be exercised');
}

$request = json_decode(
    (string) file_get_contents(dirname(__DIR__) . '/fixtures/five-items.json'),
    true,
    512,
    JSON_THROW_ON_ERROR
);

function packed(array $result): int
{
    return (int) ($result['summary']['packed_item_count'] ?? -1);
}

$native = new Packvium\Native\NativePacker($library);
if ($native->backend() !== 'rust') {
    throw new RuntimeException("the bridge did not select the healthy library at $library");
}
$nativeResult = $native->pack($request);

$fallback = new Packvium\Native\NativePacker(__DIR__ . '/no-such-library');
if ($fallback->backend() !== 'php') {
    throw new RuntimeException('the bridge selected a library path that does not exist');
}
$fallbackResult = $fallback->pack($request);

if (($nativeResult['complete'] ?? null) !== ($fallbackResult['complete'] ?? null)) {
    throw new RuntimeException('native and pure-PHP answers disagree on complete');
}
if (packed($nativeResult) !== 5 || packed($fallbackResult) !== 5) {
    throw new RuntimeException(
        'expected 5 packed items, native packed ' . packed($nativeResult)
        . ', pure PHP packed ' . packed($fallbackResult)
    );
}

echo 'PHP ' . PHP_VERSION . ": bridge selected the library and agrees with pure PHP\n";
