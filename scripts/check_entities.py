#!/usr/bin/env python3
"""Check specific entity IDs that have errors."""

import json
from pathlib import Path

base = Path(__file__).parent.parent
json_path = base / "src-tauri" / "resources" / "dxcc_entities.json"

with open(json_path) as f:
    entities = json.load(f)

# Build lookup
by_id = {int(e['EntityId']): e for e in entities}

# Check the error cases - what the code has vs what JSON says
error_cases = [
    # (prefix, code_has, json_should_have_according_to_error)
    ("VP2E", 8, 12),
    ("VP2M", 177, 96),
    ("VP5", 84, 89),
    ("VP9", 51, 64),
    ("ZF", 96, 69),
    ("V2", 12, 94),
    ("C6", 211, 60),
    ("FM", 64, 84),
    ("P4", 9, 91),
    ("GM", 265, 279),
    ("GI", 279, 265),
]

print("=== ENTITY ID VERIFICATION ===")
print()
for prefix, code_id, json_id in error_cases:
    code_name = by_id.get(code_id, {}).get('Name', 'UNKNOWN')
    json_name = by_id.get(json_id, {}).get('Name', 'UNKNOWN')
    print(f"{prefix}:")
    print(f"  Code has:    {code_id} = {code_name}")
    print(f"  JSON wants:  {json_id} = {json_name}")
    
    # Find what JSON actually says for this prefix
    for e in entities:
        prefixes = e.get('Prefixes') or []
        if isinstance(prefixes, str):
            prefixes = [prefixes]
        if prefix in prefixes:
            print(f"  JSON actual: {e['EntityId']} = {e['Name']} (Deleted: {e.get('Deleted', False)})")
    print()
