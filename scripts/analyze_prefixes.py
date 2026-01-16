#!/usr/bin/env python3
"""Analyze DXCC prefix coverage and accuracy."""

import json
from pathlib import Path

def main():
    json_path = Path(__file__).parent.parent / "src-tauri" / "resources" / "dxcc_entities.json"
    
    with open(json_path, 'r') as f:
        entities = json.load(f)
    
    active = [e for e in entities if not e.get('Deleted', False)]
    
    # Entities with no prefix data
    no_prefix = [e for e in active if e.get('Prefixes') is None]
    print(f"Active entities with NO prefix data: {len(no_prefix)}")
    for e in no_prefix:
        print(f"  - {e['EntityId']}: {e['Name']}")
    print()
    
    # Count entities that have UNIQUE prefixes (unambiguous)
    prefix_to_entities = {}
    entities_with_unique_prefix = []
    entities_with_shared_prefix = []
    
    for e in active:
        prefixes = e.get('Prefixes')
        if prefixes is None:
            continue
        if isinstance(prefixes, str):
            prefixes = [prefixes]
        
        # Check each prefix
        for p in prefixes:
            if '-' in p and len(p) > 3:  # Skip ranges
                continue
            if p not in prefix_to_entities:
                prefix_to_entities[p] = []
            prefix_to_entities[p].append(e)
    
    # Categorize entities
    for e in active:
        if e.get('Prefixes') is None:
            continue
        prefixes = e.get('Prefixes')
        if isinstance(prefixes, str):
            prefixes = [prefixes]
        
        # Check if ANY of this entity's prefixes are unique
        has_unique = False
        all_shared = True
        for p in prefixes:
            if '-' in p and len(p) > 3:
                continue
            if p in prefix_to_entities and len(prefix_to_entities[p]) == 1:
                has_unique = True
            if p in prefix_to_entities and len(prefix_to_entities[p]) > 1:
                all_shared = True
        
        if has_unique:
            entities_with_unique_prefix.append(e)
        else:
            entities_with_shared_prefix.append(e)
    
    print(f"Entities with at least one unique prefix: {len(entities_with_unique_prefix)}")
    print(f"Entities with ONLY shared prefixes: {len(entities_with_shared_prefix)}")
    print()
    
    print("=== ENTITIES WITH ONLY SHARED PREFIXES (need disambiguation) ===")
    for e in entities_with_shared_prefix:
        prefixes = e.get('Prefixes', [])
        if isinstance(prefixes, str):
            prefixes = [prefixes]
        print(f"  {e['EntityId']}: {e['Name']} - prefixes: {prefixes}")
    
    # Summary
    print()
    print("=== ACCURACY SUMMARY ===")
    total_active = len(active)
    has_prefix = len([e for e in active if e.get('Prefixes') is not None])
    unique_prefix = len(entities_with_unique_prefix)
    
    print(f"Total active entities: {total_active}")
    print(f"Entities with prefix data: {has_prefix} ({100*has_prefix/total_active:.1f}%)")
    print(f"Entities with unique prefix (100% accurate): {unique_prefix} ({100*unique_prefix/total_active:.1f}%)")
    print(f"Entities needing disambiguation: {len(entities_with_shared_prefix)} ({100*len(entities_with_shared_prefix)/total_active:.1f}%)")
    print(f"Entities with no prefix: {len(no_prefix)} ({100*len(no_prefix)/total_active:.1f}%)")

if __name__ == "__main__":
    main()
