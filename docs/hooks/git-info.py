import subprocess
import os

def on_config(config):
    """
    Hook to add git commit information. This runs once when MkDocs loads the config.
    """
    try:
        repo_root = os.path.dirname(os.path.abspath(config.config_file_path))
        commit_hash = os.environ.get('DOCS_COMMIT_SHA')
        
        if commit_hash:
            print(f"Using commit from DOCS_COMMIT_SHA env: {commit_hash[:7]}")
            short_hash = commit_hash[:7]
            commit_date = subprocess.check_output(
                ['git', 'log', '-1', '--format=%cd', '--date=short', commit_hash],
                cwd=repo_root,
                stderr=subprocess.STDOUT
            ).decode('utf-8').strip()
            print(f"Git hook: Loaded commit info {short_hash} • {commit_date}")
        else:
            # No commit details in not provided via env
            print("DOCS_COMMIT_SHA not set")
        
        # Add to extra context (available in templates if needed)
        if 'extra' not in config:
            config['extra'] = {}
        
        config['extra']['git'] = {
            'commit': commit_hash,
            'short_commit': short_hash,
            'date': commit_date,
        }
        
    except subprocess.CalledProcessError as e:
        print(f"Warning: Could not get git information: {e}")
    except Exception as e:
        print(f"Warning: Error in git hook: {e}")

    return config
