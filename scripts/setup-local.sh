#!/usr/bin/env bash
# setup-local.sh — 로컬 머신 환경 설정 스크립트
#
# 이 스크립트는 각 개발 머신에서 1회 실행합니다.
# Cargo 빌드 캐시는 추적되는 .cargo/config.toml에 따라 프로젝트 target/에 둡니다.
# 외장 exFAT/NTFS 환경이 필요하면 호출자가 --target-dir로 명시적으로 선택합니다.
#
# 사용법:
#   chmod +x scripts/setup-local.sh
#   ./scripts/setup-local.sh
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== KISAutoTrade 로컬 환경 설정 ==="
echo "프로젝트 경로: $PROJECT_ROOT"

echo "Cargo 빌드 캐시: $PROJECT_ROOT/target"
echo "로컬 검증/연구 임시 파일: $PROJECT_ROOT/.cache (Git 제외)"

# ── Node.js 확인 ────────────────────────────────────────────────
echo ""
echo "[Node.js] 버전 확인..."
if command -v node &>/dev/null; then
    echo "  ✅ Node.js $(node --version) / npm $(npm --version)"
else
    echo "  ⚠️  Node.js가 설치되지 않았습니다."
    if [[ "$(uname)" == "Darwin" ]]; then
        echo "     설치: brew install node"
    fi
fi

# ── Rust/Cargo 확인 ──────────────────────────────────────────────
echo ""
echo "[Rust] 버전 확인..."
if command -v rustc &>/dev/null; then
    echo "  ✅ $(rustc --version) / $(cargo --version)"
else
    echo "  ⚠️  Rust가 설치되지 않았습니다."
    echo "     설치: https://rustup.rs"
fi

# ── npm 의존성 설치 ──────────────────────────────────────────────
echo ""
echo "[npm] 의존성 설치..."
cd "$PROJECT_ROOT"
if [[ -f "package.json" ]]; then
    if [[ -d "node_modules" ]]; then
        echo "  ✅ 기존 node_modules 사용 (재설치 생략)"
    else
        npm ci --ignore-scripts
        echo "  ✅ lockfile 기준 npm ci 완료"
    fi
fi

echo ""
echo "=== 설정 완료! ==="
echo "이제 다음 명령으로 개발을 시작할 수 있습니다:"
echo "  cargo check --manifest-path src-tauri/Cargo.toml"
echo "  npm run dev"
