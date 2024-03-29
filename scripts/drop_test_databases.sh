#!/usr/bin/env bash

set -x
set -eo pipefail

# .envファイルに記録されている環境変数を設定
source .env

DB_CONTAINER="${POSTGRES_CONTAINER:=actix_web_example_postgres}"
DB_USER="${POSTGRES_DATABASE__USER:=postgres}"
DB_PASSWORD="${POSTGRES_DATABASE__PASSWORD:=password}"
DB_PORT="${POSTGRES_DATABASE__PORT:=5432}"
DB_HOST="${POSTGRES_DATABASE__HOST:=localhost}"

TEST_DB_PREFIX="awe_test_"

# 統合テスト用のデータベースを取得
export PGPASSWORD="${DB_PASSWORD}"
TEST_DBS=$(psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -c '\l' | grep "${TEST_DB_PREFIX}" | cut -d "|" -f 1 | sed "s/^ *\| *$//")

# 統合テスト用のデータベースを削除
for TEST_DB in ${TEST_DBS[@]}; do
    psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -c "DROP DATABASE \"${TEST_DB}\""
done
