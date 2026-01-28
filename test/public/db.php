<?php
// Test MySQL connection using PDO
$host = 'my-app-mysql';
$port = 3306;
$dbname = 'app';
$user = 'root';
$pass = 'secret';

try {
    $dsn = "mysql:host=$host;port=$port;dbname=$dbname";
    $pdo = new PDO($dsn, $user, $pass);
    $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

    echo "Connected to MySQL via PDO!\n";

    // Get MySQL version
    $stmt = $pdo->query('SELECT VERSION()');
    $version = $stmt->fetchColumn();
    echo "MySQL version: $version\n";

    // Show available PDO drivers
    echo "\nAvailable PDO drivers:\n";
    print_r(PDO::getAvailableDrivers());

} catch (PDOException $e) {
    echo "Connection failed: " . $e->getMessage() . "\n";
}
