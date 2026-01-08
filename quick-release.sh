#!/bin/bash

# Quick release aliases for common release types
set -e

case "$1" in
    "fix"|"bugfix"|"hotfix")
        echo "Creating patch release (bug fix)..."
        ./release.sh patch
        ;;
    "feature"|"feat")
        echo "Creating minor release (new feature)..."
        ./release.sh minor
        ;;
    "breaking"|"break"|"major")
        echo "Creating major release (breaking change)..."
        ./release.sh major
        ;;
    *)
        echo "Quick Release Helper"
        echo "Usage: $0 [fix|feature|breaking]"
        echo ""
        echo "Shortcuts:"
        echo "  fix      - Bug fixes (patch release)"
        echo "  feature  - New features (minor release)"
        echo "  breaking - Breaking changes (major release)"
        echo ""
        echo "Or use the full script:"
        echo "  ./release.sh [patch|minor|major]"
        ;;
esac