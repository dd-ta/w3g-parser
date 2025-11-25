#!/bin/bash

# Autonomous W3G Parser Development Loop
#
# Usage: ./scripts/autonomous-loop.sh [max_iterations]
#
# This runs Claude Code with /lead-auto repeatedly until complete or blocked.
# Uses --dangerously-skip-permissions to bypass ALL approval prompts.

set -e

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_DIR"

mkdir -p logs

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_status() { echo -e "${BLUE}[STATUS]${NC} $1"; }

check_complete() {
    if grep -q "Overall Status: COMPLETE" PROGRESS.md 2>/dev/null; then
        return 0
    fi
    return 1
}

check_blocked() {
    if grep -q "HARD-BLOCKED" PROGRESS.md 2>/dev/null; then
        return 0
    fi
    return 1
}

iteration=0
max_iterations=${1:-20}

log_info "=== Autonomous W3G Parser Development ==="
log_info "Project: $PROJECT_DIR"
log_info "Max iterations: $max_iterations"
log_warn "Running with --dangerously-skip-permissions (no approval prompts)"
echo ""

while [ $iteration -lt $max_iterations ]; do
    iteration=$((iteration + 1))
    timestamp=$(date +%Y%m%d_%H%M%S)
    logfile="logs/iteration-${iteration}-${timestamp}.log"

    log_status "=== Iteration $iteration / $max_iterations ==="

    # Run Claude Code with:
    # --print: non-interactive output
    # --dangerously-skip-permissions: bypass ALL permission prompts
    # -p: pass the prompt directly
    log_info "Running /lead-auto..."

    if command -v claude &> /dev/null; then
        claude --dangerously-skip-permissions -p "/lead-auto" 2>&1 | tee "$logfile"
    else
        log_error "Claude CLI not found!"
        log_info "Install with: npm install -g @anthropic-ai/claude-code"
        exit 1
    fi

    # Check for completion
    if check_complete; then
        log_info "PROJECT COMPLETE!"
        exit 0
    fi

    # Check for hard blocks
    if check_blocked; then
        log_error "PROJECT HARD-BLOCKED"
        log_info "Check PROGRESS.md for details"
        exit 1
    fi

    log_info "Iteration $iteration complete. Continuing..."
    sleep 3
    echo ""
done

log_warn "Reached max iterations ($max_iterations)"
log_info "Project may not be complete. Check PROGRESS.md"
exit 0
