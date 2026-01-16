#!/usr/bin/env python3
"""
Convert prefix_rules.json entity_id values from integers to 3-digit strings.

ARRL's official DXCC entity list uses 3-digit zero-padded strings (e.g., "001" for Canada).
This script converts our prefix_rules.json to match that format.
"""

import json
from pathlib import Path
from datetime import datetime, timezone

def main():
    json_path = Path(__file__).parent.parent / "src-tauri" / "resources" / "prefix_rules.json"
    
    print(f"Reading: {json_path}")
    with open(json_path, 'r') as f:
        data = json.load(f)
    
    # Check if already converted
    sample = data['rules'][0]['entity_id']
    if isinstance(sample, str):
        print(f"Already converted! Sample entity_id: {sample}")
        return
    
    print(f"Converting {len(data['rules'])} rules...")
    
    # Convert all entity_id values to 3-digit strings
    for rule in data['rules']:
        rule['entity_id'] = f"{rule['entity_id']:03d}"
    
    # Update metadata
    data['version'] = '2.0.0'
    data['generated'] = datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')
    data['source'] = 'Converted from v1.x - entity_id now uses ARRL 3-digit string format'
    
    # Write back
    with open(json_path, 'w') as f:
        json.dump(data, f, indent=2)
    
    # Verify
    with open(json_path, 'r') as f:
        verify = json.load(f)
    
    print("\n=== CONVERSION COMPLETE ===")
    print(f"Version: {verify['version']}")
    print(f"Total rules: {len(verify['rules'])}")
    print(f"\nSample rules:")
    for rule in verify['rules'][:5]:
        print(f"  {rule['prefix']}: entity_id=\"{rule['entity_id']}\" ({rule['comment']})")

if __name__ == "__main__":
    main()
