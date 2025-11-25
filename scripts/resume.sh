#!/bin/bash

# Resume from HARD-BLOCKED state
#
# Usage: ./scripts/resume.sh
#
# Shows what's blocked and helps you resume

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== W3G Parser Status ===${NC}"
echo ""

# Check current status
if grep -q "HARD-BLOCKED" PROGRESS.md 2>/dev/null; then
    echo -e "${RED}Status: HARD-BLOCKED${NC}"
    echo ""
    echo "Blocker details:"
    echo "----------------------------------------"
    grep -A 20 "## Blockers" PROGRESS.md | head -25
    echo "----------------------------------------"
    echo ""
    echo -e "${YELLOW}To resume:${NC}"
    echo "1. Resolve the blocker above"
    echo "2. Edit PROGRESS.md - change 'HARD-BLOCKED' to 'Ready to Continue'"
    echo "3. Run: ./scripts/autonomous-loop.sh"
    echo ""
    read -p "Clear HARD-BLOCKED status and resume? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        sed -i '' 's/HARD-BLOCKED/Ready to Continue/g' PROGRESS.md
        echo -e "${GREEN}Status cleared. Starting autonomous loop...${NC}"
        exec ./scripts/autonomous-loop.sh
    fi

elif grep -q "COMPLETE" PROGRESS.md 2>/dev/null; then
    echo -e "${GREEN}Status: COMPLETE${NC}"
    echo "Project is finished!"

else
    echo -e "${GREEN}Status: In Progress${NC}"
    echo ""
    echo "Current phase:"
    grep "Current Phase:" PROGRESS.md
    echo ""
    echo "Recent artifacts:"
    ls -lt thoughts/*/*.md 2>/dev/null | head -5
    echo ""
    echo -e "${YELLOW}To continue:${NC} ./scripts/autonomous-loop.sh"
fi
