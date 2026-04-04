#!/usr/bin/env bash
# setup-local.sh — 로컬 머신 환경 설정 스크립트
#
# 이 스크립트는 각 개발 머신에서 1회 실행합니다.
# 외장 드라이브(exFAT)에서 macOS Apple Double(._*) 파일로 인한
# Tauri 빌드 패닉을 방지하기 위해 target 디렉터리를 내장 드라이브로 이동합니다.
#
# 사용법:
#   chmod +x scripts/setup-local.sh
#   ./scripts/setup-local.sh
#
# - macOS  : target 디렉터리를 ~/Library/Caches/KISAutoTrade-target 으로 이동
# - Windows: 해당 없음 (._* 파일 문제 없음, 스크립트 실행 불필요)
# - Linux  : 해당 없음 (exFAT 외장 드라이브가 아닌 경우 생략 가능)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CARGO_CONFIG_DIR="$PROJECT_ROOT/.cargo"
CARGO_CONFIG_FILE="$CARGO_CONFIG_DIR/config.toml"

echo "=== KISAutoTrade 로컬 환경 설정 ==="
echo "프로젝트 경로: $PROJECT_ROOT"

# ── macOS 전용 처리 ──────────────────────────────────────────────
if [[ "$(uname)" == "Darwin" ]]; then
    echo ""
    echo "[macOS] Cargo target 디렉터리를 내장 드라이브로 이동합니다..."

    TARGET_CACHE_DIR="$HOME/Library/Caches/KISAutoTrade-target"
    mkdir -p "$TARGET_CACHE_DIR"
    mkdir -p "$CARGO_CONFIG_DIR"

    cat > "$CARGO_CONFIG_FILE" <<EOF
# Cargo 로컬 설정 (이 파일은 .gitignore에 포함됨 — 커밋하지 마세요)
#
# macOS 외장 드라이브(exFAT/NTFS)에서 발생하는 Apple Double(._*) 파일 문제를
# 방지하기 위해 target 디렉터리를 내장 드라이브로 이동합니다.
# 자동 생성됨: $(date '+%Y-%m-%d %H:%M:%S') / User: $(whoami)
[build]
target-dir = "$TARGET_CACHE_DIR"
EOF

    echo "  ✅ .cargo/config.toml 생성 완료"
    echo "  📁 target 경로: $TARGET_CACHE_DIR"
else
    echo "[$(uname)] Apple Double 파일 문제 없음 — .cargo/config.toml 생성 생략"
fi

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
    npm install
    echo "  ✅ npm install 완료"
fi

echo ""
echo "=== 설정 완료! ==="
echo "이제 다음 명령으로 개발을 시작할 수 있습니다:"
echo "  cargo check --manifest-path src-tauri/Cargo.toml"
echo "  npm run dev"
