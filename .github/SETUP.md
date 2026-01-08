# GitHub Repository Setup Guide

## Repository Settings for Automated Releases

### 1. Enable GitHub Actions

1. Go to your repository on GitHub
2. Click **Settings** tab
3. Click **Actions** in the left sidebar
4. Under **Actions permissions**, select:
   - **Allow all actions and reusable workflows**

### 2. Set Repository Permissions (Important!)

1. Go to **Settings** -> **Actions** -> **General**
2. Scroll down to **Workflow permissions**
3. Select: **Read and write permissions**
4. Check: **Allow GitHub Actions to create and approve pull requests**

**This is crucial for releases to work!** The default "Read repository contents and packages permissions" is not enough.

### 3. Repository Settings

1. Go to **Settings** -> **General**
2. Under **Features**, make sure these are enabled:
   - Issues
   - Releases (this should be enabled by default)

### 4. Branch Protection (Optional but Recommended)

1. Go to **Settings** -> **Branches**
2. Add a branch protection rule for `main`/`master`:
   - Require status checks to pass before merging
   - Require branches to be up to date before merging
   - Include administrators

### 5. Test the Setup

Once you've configured the settings:

1. **Push your code** to GitHub (if not already done)
2. **Create a test release**:
   ```bash
   ./release.sh patch
   ```
3. **Check the Actions tab** to see the workflow running

### Troubleshooting

#### If you get "Resource not accessible by integration" error:
- Check that **Workflow permissions** is set to "Read and write permissions"
- Make sure **Actions** are enabled for your repository

#### If CI doesn't run before release:
- Check that both `ci.yml` and `release.yml` are in `.github/workflows/`
- Make sure the `workflow_call` trigger is in `ci.yml`

#### If releases don't appear:
- Check the **Releases** feature is enabled in repository settings
- Verify the tag was pushed: `git tag -l`

### Manual GitHub Token (Alternative)

If you prefer not to use the default token permissions, you can create a Personal Access Token:

1. Go to **GitHub Settings** -> **Developer settings** -> **Personal access tokens** -> **Fine-grained tokens**
2. Create a new token with these permissions:
   - Contents: Read and write
   - Metadata: Read
   - Pull requests: Read and write
   - Actions: Read
3. Add it as a repository secret named `GITHUB_TOKEN`

But the recommended approach is to use the repository workflow permissions setting above.