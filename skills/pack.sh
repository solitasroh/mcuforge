#!/bin/bash
# Pack Claude skills into a distributable archive
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION=$(python3 -c "import json; print(json.load(open('$SCRIPT_DIR/manifest.json'))['version'])")

echo "Packing Claude skills v${VERSION}..."

cd "$SCRIPT_DIR"
tar -czf "../claude-skills-v${VERSION}.tar.gz" \
    manifest.json \
    universal/ \
    embedded-c/ \
    templates/ \
    agents/ \
    hooks/

echo "Created: claude-skills-v${VERSION}.tar.gz"
echo "  Universal:   $(ls universal/ | wc -l) skills"
echo "  Embedded-C:  $(ls embedded-c/ | wc -l) skills"
echo "  Templates:   $(ls templates/ | wc -l) skills"
echo "  Agents:      $(ls agents/ 2>/dev/null | wc -l) files"
echo "  Hooks:       $(ls hooks/ 2>/dev/null | wc -l) files"
