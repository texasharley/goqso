#!/usr/bin/env python3
"""
Validate prefix rules in prefixes.rs against authoritative dxcc_entities.json.
Identifies mismatches between entity_id in code vs JSON.
"""

import json
import re
from pathlib import Path

def main():
    base = Path(__file__).parent.parent
    json_path = base / "src-tauri" / "resources" / "dxcc_entities.json"
    rs_path = base / "src-tauri" / "src" / "reference" / "prefixes.rs"
    
    # Load authoritative JSON
    with open(json_path, 'r') as f:
        entities = json.load(f)
    
    # Build prefix -> entity_id map from JSON (authoritative)
    json_prefix_map = {}
    entity_name_map = {}
    
    for e in entities:
        entity_id = int(e['EntityId'])
        name = e['Name']
        deleted = e.get('Deleted', False)
        entity_name_map[entity_id] = name
        
        prefixes = e.get('Prefixes')
        if prefixes is None:
            continue
        if isinstance(prefixes, str):
            prefixes = [prefixes]
        
        for p in prefixes:
            # Skip ranges for now
            if '-' in p and len(p) > 4:
                continue
            # Store with deleted flag
            if p not in json_prefix_map:
                json_prefix_map[p] = []
            json_prefix_map[p].append({
                'entity_id': entity_id,
                'name': name,
                'deleted': deleted
            })
    
    # Parse prefixes.rs for PrefixRule entries
    with open(rs_path, 'r') as f:
        rs_content = f.read()
    
    # Match: PrefixRule { prefix: "XX", entity_id: NNN, ...
    pattern = r'PrefixRule\s*\{\s*prefix:\s*"([^"]+)",\s*entity_id:\s*(\d+)'
    rs_rules = re.findall(pattern, rs_content)
    
    print(f"=== DXCC PREFIX VALIDATION ===")
    print(f"JSON entities: {len(entities)}")
    print(f"JSON unique prefixes: {len(json_prefix_map)}")
    print(f"Rust prefix rules: {len(rs_rules)}")
    print()
    
    # Validate each Rust rule
    errors = []
    warnings = []
    validated = 0
    disambiguated = 0
    
    for prefix, entity_id in rs_rules:
        entity_id = int(entity_id)
        
        if prefix not in json_prefix_map:
            # Check if it's a disambiguation suffix (e.g., HK0M, VK9X)
            # Find base prefix
            base_found = False
            for base_len in range(len(prefix)-1, 0, -1):
                base = prefix[:base_len]
                if base in json_prefix_map:
                    # This is a disambiguation rule
                    entries = json_prefix_map[base]
                    valid_ids = [e['entity_id'] for e in entries if not e['deleted']]
                    if entity_id in valid_ids:
                        disambiguated += 1
                        base_found = True
                        break
                    else:
                        # Check if it matches any entity with this base prefix
                        all_ids = [e['entity_id'] for e in entries]
                        if entity_id in all_ids:
                            warnings.append(f"  {prefix} -> {entity_id}: Maps to DELETED entity")
                            base_found = True
                            break
            
            if not base_found:
                # Truly unknown prefix - might be valid ITU allocation not in JSON
                # Check if entity_id is valid
                if entity_id in entity_name_map:
                    warnings.append(f"  {prefix} -> {entity_id} ({entity_name_map[entity_id]}): Prefix not in JSON (may be valid ITU block)")
                else:
                    errors.append(f"  {prefix} -> {entity_id}: INVALID entity_id!")
        else:
            # Prefix exists in JSON - verify entity_id matches
            entries = json_prefix_map[prefix]
            active_entries = [e for e in entries if not e['deleted']]
            
            if len(active_entries) == 1:
                # Unambiguous - must match exactly
                expected = active_entries[0]['entity_id']
                if entity_id != expected:
                    errors.append(f"  {prefix} -> {entity_id} ({entity_name_map.get(entity_id, '?')}): MISMATCH! Expected {expected} ({active_entries[0]['name']})")
                else:
                    validated += 1
            elif len(active_entries) > 1:
                # Ambiguous - entity_id must be one of the valid options
                valid_ids = [e['entity_id'] for e in active_entries]
                if entity_id in valid_ids:
                    disambiguated += 1
                else:
                    names = ', '.join([f"{e['entity_id']}={e['name']}" for e in active_entries])
                    errors.append(f"  {prefix} -> {entity_id}: Not in valid set [{names}]")
            else:
                # All entries are deleted
                deleted_ids = [e['entity_id'] for e in entries]
                if entity_id in deleted_ids:
                    warnings.append(f"  {prefix} -> {entity_id}: Maps to DELETED entity")
                else:
                    errors.append(f"  {prefix} -> {entity_id}: No active entity for this prefix")
    
    # Summary
    print(f"=== VALIDATION RESULTS ===")
    print(f"✅ Validated (exact match): {validated}")
    print(f"✅ Disambiguated rules: {disambiguated}")
    print(f"⚠️  Warnings: {len(warnings)}")
    print(f"❌ Errors: {len(errors)}")
    print()
    
    if errors:
        print("=== ERRORS (must fix) ===")
        for e in errors:
            print(e)
        print()
    
    if warnings:
        print("=== WARNINGS (review) ===")
        for w in warnings[:20]:  # Limit output
            print(w)
        if len(warnings) > 20:
            print(f"  ... and {len(warnings) - 20} more warnings")
        print()
    
    # Calculate accuracy
    total = validated + disambiguated + len(errors)
    if total > 0:
        accuracy = (validated + disambiguated) / total * 100
        print(f"=== ACCURACY ===")
        print(f"Correct rules: {validated + disambiguated}/{total} ({accuracy:.1f}%)")
    
    # Return exit code
    return 1 if errors else 0

if __name__ == "__main__":
    exit(main())
