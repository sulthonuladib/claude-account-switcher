#!/bin/bash

# Semantic Release Script for Claude Account Switcher
# Usage: ./release.sh [patch|minor|major]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [patch|minor|major]"
    echo ""
    echo "Semantic version bumps:"
    echo "  patch  - Bug fixes (0.1.0 -> 0.1.1)"
    echo "  minor  - New features (0.1.0 -> 0.2.0)"
    echo "  major  - Breaking changes (0.1.0 -> 1.0.0)"
    echo ""
    echo "Example:"
    echo "  $0 patch   # For bug fixes"
    echo "  $0 minor   # For new features"
    echo "  $0 major   # For breaking changes"
}

# Function to get current version from Cargo.toml
get_current_version() {
    grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
}

# Function to bump version
bump_version() {
    local current_version=$1
    local bump_type=$2
    
    # Split version into parts
    IFS='.' read -ra VERSION_PARTS <<< "$current_version"
    local major=${VERSION_PARTS[0]}
    local minor=${VERSION_PARTS[1]}
    local patch=${VERSION_PARTS[2]}
    
    case $bump_type in
        patch)
            patch=$((patch + 1))
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        *)
            print_error "Invalid bump type: $bump_type"
            show_usage
            exit 1
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Function to update version in Cargo.toml
update_cargo_version() {
    local new_version=$1
    local os_type=$(uname -s)
    
    if [[ "$os_type" == "Darwin" ]]; then
        # macOS
        sed -i '' "s/^version = .*/version = \"$new_version\"/" Cargo.toml
    else
        # Linux
        sed -i "s/^version = .*/version = \"$new_version\"/" Cargo.toml
    fi
}

# Function to check if git working directory is clean
check_git_status() {
    if [[ -n $(git status --porcelain) ]]; then
        print_error "Working directory is not clean. Please commit or stash your changes first."
        exit 1
    fi
}

# Function to check if we're on main/master branch
check_branch() {
    local current_branch=$(git branch --show-current)
    if [[ "$current_branch" != "main" && "$current_branch" != "master" ]]; then
        print_warning "You're on branch '$current_branch', not main/master."
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Aborted."
            exit 1
        fi
    fi
}

# Main script
main() {
    print_info "Claude Account Switcher - Semantic Release Script"
    echo ""
    
    # Check if argument is provided
    if [[ $# -eq 0 ]]; then
        show_usage
        exit 1
    fi
    
    local bump_type=$1
    
    # Validate bump type
    if [[ ! "$bump_type" =~ ^(patch|minor|major)$ ]]; then
        print_error "Invalid version bump type: $bump_type"
        show_usage
        exit 1
    fi
    
    # Check if we're in a git repository
    if [[ ! -d .git ]]; then
        print_error "Not in a git repository!"
        exit 1
    fi
    
    # Check if Cargo.toml exists
    if [[ ! -f Cargo.toml ]]; then
        print_error "Cargo.toml not found!"
        exit 1
    fi
    
    # Pre-flight checks
    print_info "Running pre-flight checks..."
    check_git_status
    check_branch
    
    # Get current version
    local current_version=$(get_current_version)
    print_info "Current version: $current_version"
    
    # Calculate new version
    local new_version=$(bump_version "$current_version" "$bump_type")
    print_info "New version: $new_version (${bump_type} bump)"
    
    # Confirm release
    echo ""
    print_warning "This will:"
    echo "  1. Update Cargo.toml version to $new_version"
    echo "  2. Run tests and build"
    echo "  3. Commit the version change"
    echo "  4. Create and push git tag v$new_version"
    echo "  5. Trigger GitHub Actions release workflow"
    echo ""
    read -p "Continue with release? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Release aborted."
        exit 1
    fi
    
    print_info "Starting release process..."
    
    # Update version in Cargo.toml
    print_info "Updating Cargo.toml version..."
    update_cargo_version "$new_version"
    print_success "Version updated to $new_version"
    
    # Run tests
    print_info "Running tests..."
    if cargo test; then
        print_success "All tests passed"
    else
        print_error "Tests failed! Reverting changes..."
        git checkout -- Cargo.toml
        exit 1
    fi
    
    # Build release
    print_info "Building release..."
    if cargo build --release; then
        print_success "Release build successful"
    else
        print_error "Build failed! Reverting changes..."
        git checkout -- Cargo.toml
        exit 1
    fi
    
    # Update Cargo.lock
    print_info "Updating Cargo.lock..."
    cargo update
    
    # Commit changes
    print_info "Committing version bump..."
    git add Cargo.toml Cargo.lock
    git commit -m "chore: bump version to $new_version"
    print_success "Changes committed"
    
    # Create and push tag
    local tag_name="v$new_version"
    print_info "Creating git tag: $tag_name"
    git tag -a "$tag_name" -m "Release $tag_name"
    
    print_info "Pushing changes and tag to remote..."
    git push origin HEAD
    git push origin "$tag_name"
    
    print_success "Tag $tag_name pushed successfully!"
    echo ""
    print_success "🚀 Release $new_version initiated!"
    print_info "GitHub Actions will now build and publish the release."
    print_info "Check the progress at: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\([^.]*\).*/\1/')/actions"
}

# Run main function with all arguments
main "$@"